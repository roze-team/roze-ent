#![allow(dead_code)]

use std::sync::Arc;

use crate::config::Config;

const USES_IDEMPOTENCY: bool = false;

#[derive(Clone, Debug)]
pub struct ServiceContext {
    pub config: Config,
    pub health: roze_health::HealthRegistry,
    pub extensions: roze_service::ApplicationExtensions,
    pub db_connections: Option<roze_db::DatabaseConnections>,
    pub db_shards: Option<roze_db::ShardedDatabase>,
    pub cache: Option<roze_cache::RedisCache>,
    pub mq: Option<Arc<roze_nats::NatsJetStream>>,
    pub storage: Option<Arc<dyn roze_storage::ObjectStorage>>,
    pub report_source: Option<Arc<dyn roze_report::ReportDataSource>>,
    pub outbox: Arc<dyn roze_transaction::OutboxStore>,
    pub sql_outbox: Option<Arc<roze_transaction_sql::SqlOutboxStore>>,
    pub idempotency: Arc<dyn roze_middleware::IdempotencyStore>,
    pub rate_limiter: Arc<roze_rate_limit::RateLimiter>,
}

impl ServiceContext {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let health = roze_health::HealthRegistry::new();
        let database_runtime = roze_db::connect_runtime_optional(config.database.as_ref()).await?;
        let db_connections = database_runtime
            .as_ref()
            .and_then(roze_db::DatabaseRuntime::direct)
            .cloned();
        let db_shards = database_runtime
            .as_ref()
            .and_then(roze_db::DatabaseRuntime::sharded)
            .cloned();
        let cache = match config.cache.as_ref() {
            Some(cache) => Some(
                roze_cache::RedisCache::connect(&roze_cache::CacheConfig {
                    url: cache.url.clone(),
                    cluster_urls: cache.cluster_urls.clone(),
                    namespace: cache.namespace.clone(),
                    default_ttl_secs: cache.default_ttl_secs,
                })
                .await?,
            ),
            None => None,
        };
        let mq = match config.nats.as_ref() {
            Some(nats) => Some(Arc::new(
                roze_nats::NatsJetStream::connect(nats.clone()).await?,
            )),
            None => None,
        };
        let storage = match config.storage.clone() {
            Some(storage) => Some(Arc::from(roze_storage::build_storage(storage)?)),
            None => None,
        };
        if let Some(database_runtime) = database_runtime.clone() {
            health.register_dependency("database", move || {
                let database_runtime = database_runtime.clone();
                async move { database_runtime.health_check().await.map_err(Into::into) }
            });
        }
        if let Some(cache) = cache.clone() {
            health.register_dependency("redis", move || {
                let cache = cache.clone();
                async move { cache.health_check().await }
            });
        }
        if let Some(mq) = mq.clone() {
            health.register_dependency("nats", move || {
                let mq = mq.clone();
                async move { mq.health_check().await }
            });
        }
        let (outbox, sql_outbox): (
            Arc<dyn roze_transaction::OutboxStore>,
            Option<Arc<roze_transaction_sql::SqlOutboxStore>>,
        ) = match config.outbox.as_ref().filter(|settings| settings.enabled) {
            Some(settings) => {
                let use_sql = match settings.store {
                    roze_config::OutboxStoreKind::Auto => db_connections.is_some(),
                    roze_config::OutboxStoreKind::Memory => false,
                    roze_config::OutboxStoreKind::Sql => true,
                };
                if use_sql {
                    let database = db_connections
                        .as_ref()
                        .map(|connections| connections.write().clone())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "SQL outbox requires a direct/proxy database; sharded services \
                                 must install an application-routed OutboxStore explicitly"
                            )
                        })?;
                    let store = Arc::new(roze_transaction_sql::SqlOutboxStore::with_config(
                        database,
                        roze_transaction_sql::SqlOutboxConfig {
                            table: settings.table.clone(),
                            max_attempts: settings.max_attempts,
                        },
                    )?);
                    if settings.migrate {
                        store.migrate().await?;
                    }
                    (store.clone(), Some(store))
                } else {
                    if config.profile.is_production() {
                        anyhow::bail!(
                            "production services with outbox.enabled require a SQL outbox store"
                        );
                    }
                    tracing::warn!(
                        "in-memory outbox is process-local and intended only for development"
                    );
                    (Arc::new(roze_transaction::InMemoryOutbox::new()), None)
                }
            }
            None => (Arc::new(roze_transaction::InMemoryOutbox::new()), None),
        };
        let idempotency: Arc<dyn roze_middleware::IdempotencyStore> = {
            let settings = config.idempotency.clone().unwrap_or_default();
            let use_redis = match settings.store {
                roze_config::IdempotencyStoreKind::Auto => config.cache.is_some(),
                roze_config::IdempotencyStoreKind::Memory => false,
                roze_config::IdempotencyStoreKind::Redis => true,
            };
            if use_redis {
                let cache = config.cache.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("Redis idempotency requires cache Redis topology configuration")
                })?;
                let mut store_config =
                    roze_middleware::RedisIdempotencyConfig::new(cache.url.clone());
                store_config.cluster_urls = cache.cluster_urls.clone();
                store_config.key_prefix = settings.key_prefix;
                store_config.record_ttl_millis = settings.record_ttl_millis;
                let store = roze_middleware::RedisIdempotencyStore::connect(store_config)?;
                if settings.unavailable_policy
                    == roze_config::IdempotencyUnavailablePolicy::FailFast
                {
                    store.health_check().await?;
                }
                let health_store = store.clone();
                health.register_dependency("idempotency:redis", move || {
                    let store = health_store.clone();
                    async move { store.health_check().await }
                });
                Arc::new(store)
            } else {
                if USES_IDEMPOTENCY && config.profile.is_production() {
                    anyhow::bail!(
                        "production services with idempotent routes require Redis idempotency"
                    );
                }
                if USES_IDEMPOTENCY {
                    tracing::warn!(
                        "in-memory idempotency is process-local and intended only for development"
                    );
                }
                Arc::new(roze_middleware::InMemoryIdempotencyStore::default())
            }
        };
        let rate_limiter_config = config.resolved_rate_limiter_config();
        let rate_limiter = Arc::new(roze_rate_limit::RateLimiter::from_config(
            &rate_limiter_config,
        )?);
        if config.profile.is_production()
            && config.governance.uses_rate_limit()
            && rate_limiter.store_kind() == roze_rate_limit::RateLimitStoreKind::Memory
        {
            anyhow::bail!(
                "production services with rate limiting require governance.rate_limiter.redis_url or cache.url"
            );
        }
        if rate_limiter.store_kind() == roze_rate_limit::RateLimitStoreKind::Redis {
            let health_limiter = rate_limiter.clone();
            health.register_dependency("rate-limit:redis", move || {
                let limiter = health_limiter.clone();
                async move { limiter.health_check().await }
            });
        }
        health.mark_ready();
        Ok(Self {
            config,
            health,
            extensions: roze_service::ApplicationExtensions::new(),
            db_connections,
            db_shards,
            cache,
            mq,
            storage,
            report_source: None,
            outbox,
            sql_outbox,
            idempotency,
            rate_limiter,
        })
    }

    pub fn read_db(&self) -> anyhow::Result<&roze_db::DatabaseConnection> {
        self.db_connections
            .as_ref()
            .map(|connections| connections.read())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "direct database connection is not configured; route sharded models explicitly"
                )
            })
    }

    pub fn write_db(&self) -> anyhow::Result<&roze_db::DatabaseConnection> {
        self.db_connections
            .as_ref()
            .map(|connections| connections.write())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "direct database connection is not configured; route sharded models explicitly"
                )
            })
    }

    pub fn sharded_db(&self) -> anyhow::Result<&roze_db::ShardedDatabase> {
        self.db_shards
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("sharded database topology is not configured"))
    }

    pub fn with_outbox_store(mut self, outbox: Arc<dyn roze_transaction::OutboxStore>) -> Self {
        self.outbox = outbox;
        self.sql_outbox = None;
        self
    }

    pub fn sql_outbox(&self) -> anyhow::Result<Arc<roze_transaction_sql::SqlOutboxStore>> {
        self.sql_outbox
            .clone()
            .ok_or_else(|| anyhow::anyhow!("SQL outbox is not configured"))
    }

    pub fn with_idempotency_store(
        mut self,
        idempotency: Arc<dyn roze_middleware::IdempotencyStore>,
    ) -> Self {
        self.idempotency = idempotency;
        self
    }

    pub fn with_rate_limiter(mut self, rate_limiter: Arc<roze_rate_limit::RateLimiter>) -> Self {
        self.rate_limiter = rate_limiter;
        self
    }

    pub fn with_storage(mut self, storage: Arc<dyn roze_storage::ObjectStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn storage(&self) -> anyhow::Result<Arc<dyn roze_storage::ObjectStorage>> {
        self.storage
            .clone()
            .ok_or_else(|| anyhow::anyhow!("object storage is not configured"))
    }

    pub fn with_report_source(
        mut self,
        report_source: Arc<dyn roze_report::ReportDataSource>,
    ) -> Self {
        self.report_source = Some(report_source);
        self
    }

    pub fn report_source(&self) -> anyhow::Result<Arc<dyn roze_report::ReportDataSource>> {
        self.report_source
            .clone()
            .ok_or_else(|| anyhow::anyhow!("report data source is not configured"))
    }

    pub async fn media_url(
        &self,
        key: &str,
        expires: std::time::Duration,
    ) -> anyhow::Result<roze_storage::MediaUrl> {
        roze_storage::resolve_media_url(self.storage()?.as_ref(), key, expires).await
    }

    pub fn jwt_config(&self) -> Option<roze_jwt::JwtConfig> {
        self.config.auth.as_ref().map(Into::into)
    }

    pub fn mq(&self) -> anyhow::Result<Arc<roze_nats::NatsJetStream>> {
        self.mq
            .clone()
            .ok_or_else(|| anyhow::anyhow!("nats jetstream is not configured"))
    }

    pub fn with_extension<T>(self, value: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.extensions.insert(value);
        self
    }

    pub fn insert_extension<T>(&self, value: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.extensions.insert(value)
    }

    pub fn extension<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.extensions.get::<T>()
    }

    pub fn require_extension<T>(&self) -> anyhow::Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.extensions.require::<T>()
    }
}

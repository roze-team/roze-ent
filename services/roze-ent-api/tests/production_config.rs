use std::path::{Path, PathBuf};

use roze_config::{
    load_service_with_secret_provider, SecretProvider, SecretProviderError, ServiceProfile,
};

#[derive(Debug)]
struct ProductionTemplateSecrets;

impl SecretProvider for ProductionTemplateSecrets {
    fn resolve(
        &self,
        reference: &str,
        _base_dir: &Path,
    ) -> Result<Option<String>, SecretProviderError> {
        let value = match reference {
            "env://ROZE_ENT_DATABASE_URL" => {
                "postgres://roze_ent:template-only@postgres.internal/roze_ent"
            }
            "env://ROZE_ENT_REDIS_URL" => "redis://redis.internal/0",
            "env://ROZE_ENT_JWT_SECRET" => "template-only-jwt-secret-at-least-thirty-two-bytes",
            _ => return Ok(None),
        };
        Ok(Some(value.to_string()))
    }
}

fn production_config_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/config/rest.production.yaml")
}

#[test]
fn production_config_is_strict_and_resolves_only_declared_secrets() {
    let config =
        load_service_with_secret_provider(production_config_path(), &ProductionTemplateSecrets)
            .expect(
                "production configuration must parse, resolve secrets, and pass Roze validation",
            );

    assert_eq!(config.profile, ServiceProfile::Production);

    let rest = config
        .rest
        .as_ref()
        .expect("production REST listener must be configured");
    assert_eq!(rest.addr.ip().to_string(), "0.0.0.0");
    assert!(rest.connect_info);
    assert!(!rest.middlewares.cors);
    assert!(!rest.middlewares.trust_forwarded_identity_headers);
    assert_eq!(rest.middlewares.request_body_limit_bytes, Some(1_048_576));

    let database = config
        .database
        .as_ref()
        .expect("production database must be configured");
    assert!(database.url.starts_with("postgres://"));
    assert!(!database.sqlx_logging);

    let cache = config
        .cache
        .as_ref()
        .expect("production Redis must be configured");
    assert!(cache.url.starts_with("redis://"));

    let auth = config
        .auth
        .as_ref()
        .expect("production JWT authentication must be configured");
    assert_eq!(auth.jwt_active_key_id, "production-v1");
    assert_eq!(auth.jwt_keys.len(), 1);
    assert!(auth.jwt_keys[0].secret.len() >= 32);

    let limiter = config.resolved_rate_limiter_config();
    assert_eq!(limiter.namespace.as_deref(), Some("roze-ent:production"));
    assert!(limiter
        .redis_url
        .as_deref()
        .is_some_and(|url| url.starts_with("redis://")));
}

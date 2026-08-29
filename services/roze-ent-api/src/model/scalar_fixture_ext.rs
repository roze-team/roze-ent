#![allow(dead_code)]

// Application-owned extension methods for `scalar_fixtures`.
// This file is created by rozectl but preserved during `--update`.
use super::scalar_fixture::{Model, ScalarFixtureRepository};

impl Model {
    // Add domain helpers for generated SeaORM model rows here.
}

impl<'a> ScalarFixtureRepository<'a> {
    // Add application-owned repository queries here.
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use rust_decimal::Decimal;
    use sea_orm::ConnectionTrait as _;

    async fn sqlite_context() -> anyhow::Result<crate::svc::ServiceContext> {
        let config: crate::config::Config = serde_json::from_value(serde_json::json!({
            "name": "roze-ent-scalar-parity-test",
            "profile": "test",
            "governance": {},
            "database": {
                "mode": "direct",
                "url": "sqlite::memory:",
                "max_connections": 1,
                "min_connections": 1
            }
        }))?;
        let ctx = crate::svc::ServiceContext::new(config).await?;
        let db = ctx.write_db()?;
        for statement in [
            "CREATE TABLE scalar_fixtures (\
                id TEXT PRIMARY KEY, \
                external_id TEXT NOT NULL UNIQUE, \
                small_value INTEGER NOT NULL, \
                int_value INTEGER NOT NULL, \
                big_value INTEGER NOT NULL, \
                float_value REAL NOT NULL, \
                double_value REAL NOT NULL, \
                payload BLOB NULL, \
                metadata TEXT NOT NULL, \
                amount REAL NOT NULL, \
                local_time TEXT NOT NULL, \
                event_time TEXT NULL\
            )",
            "CREATE TABLE locale_settings (\
                tenant_id TEXT NOT NULL, \
                locale TEXT NOT NULL, \
                value TEXT NOT NULL, \
                PRIMARY KEY (tenant_id, locale)\
            )",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(ctx)
    }

    #[tokio::test]
    async fn scalar_and_composite_ids_have_real_sqlite_evidence() -> anyhow::Result<()> {
        use crate::model::{locale_setting, scalar_fixture};

        let ctx = sqlite_context().await?;
        let models = ctx.model();
        let scalars = models.scalar_fixture();
        let locales = models.locale_setting();
        let local_time: NaiveDateTime = "2026-08-29T12:34:56".parse()?;
        let event_time: DateTime<Utc> = "2026-08-29T12:34:56Z".parse()?;
        let amount = Decimal::new(123_456, 3);
        let metadata = serde_json::json!({
            "enabled": true,
            "labels": ["roze", "ent"],
            "nested": {"version": 1}
        });

        let created = scalars
            .create()
            .set_id("fixture-custom-id".to_string())
            .set_external_id("c2ef9f44-3c14-4f77-99c8-75b5f24d11e7".to_string())
            .set_small_value(-123)
            .set_int_value(-123_456)
            .set_big_value(9_876_543_210)
            .set_float_value(1.25)
            .set_double_value(-9_876.5)
            .set_payload(Some(vec![0, 1, 2, 127, 255]))
            .set_metadata(metadata.clone())
            .set_amount(amount)
            .set_local_time(local_time)
            .set_event_time(Some(event_time))
            .save()
            .await?;

        assert_eq!(created.id, "fixture-custom-id");
        assert_eq!(created.external_id, "c2ef9f44-3c14-4f77-99c8-75b5f24d11e7");
        assert_eq!(created.small_value, -123);
        assert_eq!(created.int_value, -123_456);
        assert_eq!(created.big_value, 9_876_543_210);
        assert_eq!(created.float_value, 1.25);
        assert_eq!(created.double_value, -9_876.5);
        assert_eq!(created.payload, Some(vec![0, 1, 2, 127, 255]));
        assert_eq!(created.metadata, metadata);
        assert_eq!(created.amount, amount);
        assert_eq!(created.local_time, local_time);
        assert_eq!(created.event_time, Some(event_time));
        assert_eq!(
            scalars
                .query()
                .where_(scalar_fixture::small_value_eq(-123))
                .where_(scalar_fixture::amount_eq(amount))
                .only_id()
                .await?,
            "fixture-custom-id"
        );

        let updated = scalars
            .update_one("fixture-custom-id".to_string())
            .clear_payload()
            .set_metadata(serde_json::json!({"updated": true}))
            .save()
            .await?;
        assert_eq!(updated.payload, None);
        assert_eq!(updated.metadata, serde_json::json!({"updated": true}));

        let key = locale_setting::LocaleSettingKey {
            tenant_id: "tenant-a".to_string(),
            locale: "zh-CN".to_string(),
        };
        locales
            .create()
            .set_tenant_id(key.tenant_id.clone())
            .set_locale(key.locale.clone())
            .set_value("你好，Roze".to_string())
            .save()
            .await?;
        assert_eq!(
            locales.find_by_key(key.clone()).await?.unwrap().value,
            "你好，Roze"
        );
        let updated_locale = locales
            .update_one(key.clone())
            .set_value("Rust ORM".to_string())
            .save()
            .await?;
        assert_eq!(updated_locale.tenant_id, key.tenant_id);
        assert_eq!(updated_locale.locale, key.locale);
        assert_eq!(updated_locale.value, "Rust ORM");
        assert_eq!(locales.delete_by_key(key).await?.rows_affected, 1);
        assert_eq!(locales.count().await?, 0);
        Ok(())
    }
}

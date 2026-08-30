#![allow(dead_code)]

// Application-owned extension methods for `audit_events`.
// This file is created by rozectl but preserved during `--update`.
use super::audit_event::{AuditEventRepository, Model};

impl Model {
    // Add domain helpers for generated SeaORM model rows here.
}

impl<'a> AuditEventRepository<'a> {
    // Add application-owned repository queries here.
}

#[cfg(test)]
mod tests {
    async fn database_context(url: &str) -> anyhow::Result<crate::svc::ServiceContext> {
        let config: crate::config::Config = serde_json::from_value(serde_json::json!({
            "name": "roze-ent-multi-schema-test",
            "profile": "test",
            "governance": {},
            "database": {
                "mode": "direct",
                "url": url,
                "max_connections": 1,
                "min_connections": 1
            }
        }))?;
        crate::svc::ServiceContext::new(config).await
    }

    #[tokio::test]
    #[ignore = "requires ROZE_ENT_TEST_DATABASE_URL and an applied project schema"]
    async fn generated_multi_schema_model_and_cross_schema_edge_have_real_sql_evidence(
    ) -> anyhow::Result<()> {
        let url = std::env::var("ROZE_ENT_TEST_DATABASE_URL")?;
        let marker = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let ctx = database_context(&url).await?;
        let users = ctx.model().user();
        let events = ctx.model().audit_event();

        let user = users
            .create()
            .set_email(format!("multi-schema-{marker}@example.com"))
            .set_name("multi schema".to_string())
            .set_created_at(1)
            .save()
            .await?;
        let event = events
            .create()
            .set_user(&user)
            .set_action("created".to_string())
            .set_created_at(2)
            .save()
            .await?;

        assert_eq!(
            event.traverse_user(&users).await?.only_name().await?,
            "multi schema"
        );
        assert_eq!(
            events
                .query()
                .where_user_with(&users, [crate::model::user::email_eq(user.email.clone())],)
                .await?
                .only_action()
                .await?,
            "created"
        );

        users.delete_one(user.id).exec().await?;
        assert!(
            !events
                .query()
                .where_(crate::model::audit_event::id_eq(event.id))
                .exists()
                .await?
        );
        Ok(())
    }
}

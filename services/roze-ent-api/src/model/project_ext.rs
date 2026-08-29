#![allow(dead_code)]

// Application-owned extension methods for `projects`.
// This file is created by rozectl but preserved during `--update`.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::project::{Model, ProjectCreate, ProjectMutationHooks, ProjectRepository};

impl Model {
    // Add domain helpers for generated SeaORM model rows here.
}

impl<'a> ProjectRepository<'a> {
    // Add application-owned repository queries here.
}

/// Applies API-owned values to the generated create builder as one reusable mixin.
pub struct ProjectCreateValues {
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
}

impl<'repo, 'ctx> roze_orm::OperationMixin<ProjectCreate<'repo, 'ctx>> for ProjectCreateValues {
    fn apply(&self, mutation: ProjectCreate<'repo, 'ctx>) -> ProjectCreate<'repo, 'ctx> {
        mutation
            .set_tenant_id(self.tenant_id.clone())
            .set_name(self.name.clone())
            .set_description(self.description.clone())
            .set_version(1)
    }
}

/// A small policy adapter that lets the application make tenant authorization explicit.
pub struct TenantWritePolicy {
    allowed: bool,
}

impl TenantWritePolicy {
    pub const fn new(allowed: bool) -> Self {
        Self { allowed }
    }
}

impl<C> roze_orm::Policy<C> for TenantWritePolicy {
    type Error = anyhow::Error;

    fn evaluate(&self, _context: &C) -> Result<roze_orm::PolicyDecision, Self::Error> {
        Ok(if self.allowed {
            roze_orm::PolicyDecision::Allow
        } else {
            roze_orm::PolicyDecision::Deny
        })
    }
}

/// Reusable client-level hook provider. It counts successful mutation execution,
/// which happens before an enclosing transaction commits. Hooks must not emit
/// irreversible side effects; use a transactional outbox for those.
#[derive(Clone, Default)]
pub struct ProjectMutationAudit {
    successful_creates: Arc<AtomicUsize>,
}

impl ProjectMutationAudit {
    pub fn successful_creates(&self) -> usize {
        self.successful_creates.load(Ordering::Relaxed)
    }
}

impl ProjectMutationHooks for ProjectMutationAudit {
    fn create<'repo, 'ctx>(
        &self,
        mutation: ProjectCreate<'repo, 'ctx>,
    ) -> ProjectCreate<'repo, 'ctx> {
        mutation.hook(ProjectCreateAuditHook {
            successful_creates: self.successful_creates.clone(),
        })
    }
}

struct ProjectCreateAuditHook {
    successful_creates: Arc<AtomicUsize>,
}

impl<'repo, 'ctx> roze_orm::OperationMiddleware<ProjectCreate<'repo, 'ctx>, Model, anyhow::Error>
    for ProjectCreateAuditHook
{
    fn call<'a>(
        &'a self,
        mutation: ProjectCreate<'repo, 'ctx>,
        next: roze_orm::OperationNext<'a, ProjectCreate<'repo, 'ctx>, Model, anyhow::Error>,
    ) -> roze_orm::OperationFuture<'a, Model, anyhow::Error>
    where
        ProjectCreate<'repo, 'ctx>: 'a,
        Model: 'a,
        anyhow::Error: 'a,
    {
        Box::pin(async move {
            let model = next.run(mutation).await?;
            self.successful_creates.fetch_add(1, Ordering::Relaxed);
            tracing::debug!(
                model = "Project",
                operation = "create",
                "model mutation succeeded"
            );
            Ok(model)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::ConnectionTrait as _;

    async fn sqlite_context() -> anyhow::Result<crate::svc::ServiceContext> {
        let config: crate::config::Config = serde_json::from_value(serde_json::json!({
            "name": "roze-ent-model-test",
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
        ctx.write_db()?
            .execute_unprepared(
                "CREATE TABLE projects (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT, \
                    tenant_id TEXT NOT NULL, \
                    name TEXT NOT NULL, \
                    description TEXT NULL, \
                    version INTEGER NOT NULL DEFAULT 1, \
                    deleted_at INTEGER NULL, \
                    created_at INTEGER NOT NULL, \
                    updated_at INTEGER NOT NULL, \
                    UNIQUE (tenant_id, name)\
                )",
            )
            .await?;
        Ok(ctx)
    }

    #[tokio::test]
    async fn transaction_hook_policy_and_mixin_have_real_sqlite_evidence() -> anyhow::Result<()> {
        let ctx = sqlite_context().await?;
        let audit = ProjectMutationAudit::default();
        let client = ctx.model().use_project_mutation_hooks(&audit);

        let committed = client
            .transaction(|models| {
                Box::pin(async move {
                    let repository = models.project();
                    let policy: Arc<
                        dyn roze_orm::Policy<ProjectCreate<'_, '_>, Error = anyhow::Error>,
                    > = Arc::new(TenantWritePolicy::new(true));
                    repository
                        .create()
                        .mixin(ProjectCreateValues {
                            tenant_id: "tenant-a".to_string(),
                            name: "committed".to_string(),
                            description: Some("created in a transaction".to_string()),
                        })
                        .policy([policy], || anyhow::anyhow!("tenant write denied"))
                        .save()
                        .await
                })
            })
            .await?;
        assert_eq!(committed.tenant_id, "tenant-a");
        assert_eq!(committed.version, 1);
        assert_eq!(audit.successful_creates(), 1);

        let repository = client.project();
        let denied_policy: Arc<dyn roze_orm::Policy<ProjectCreate<'_, '_>, Error = anyhow::Error>> =
            Arc::new(TenantWritePolicy::new(false));
        let denied = repository
            .create()
            .mixin(ProjectCreateValues {
                tenant_id: "tenant-b".to_string(),
                name: "denied".to_string(),
                description: None,
            })
            .policy([denied_policy], || anyhow::anyhow!("tenant write denied"))
            .save()
            .await;
        assert_eq!(denied.unwrap_err().to_string(), "tenant write denied");
        assert_eq!(audit.successful_creates(), 1);

        let rolled_back: anyhow::Result<()> = client
            .transaction(|models| {
                Box::pin(async move {
                    models
                        .project()
                        .create()
                        .mixin(ProjectCreateValues {
                            tenant_id: "tenant-a".to_string(),
                            name: "rolled-back".to_string(),
                            description: None,
                        })
                        .save()
                        .await?;
                    anyhow::bail!("force rollback")
                })
            })
            .await;
        assert_eq!(rolled_back.unwrap_err().to_string(), "force rollback");

        let projects = client
            .project()
            .query()
            .primary()
            .with_deleted()
            .all()
            .await?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "committed");
        // The second create reached its hook before the outer transaction was
        // forced to roll back, while the database correctly retained one row.
        assert_eq!(audit.successful_creates(), 2);
        Ok(())
    }
}

use roze_migration::{
    migrate_sqlite, plan_apply, rollback_sqlite, sqlite_migration_records, MigrationDirection,
    MigrationPlanError, MigrationRecord, SqlMigration,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

fn project_migrations() -> Vec<SqlMigration> {
    vec![
        SqlMigration::new(
            1,
            "init",
            include_str!("../../../migrations/sqlite/0001_init.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0001_init.sql"
            )),
        ),
        SqlMigration::new(
            2,
            "groups_and_memberships",
            include_str!("../../../migrations/sqlite/0002_groups_and_memberships.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0002_groups_and_memberships.sql"
            )),
        ),
        SqlMigration::new(
            3,
            "tenant_projects",
            include_str!("../../../migrations/sqlite/0003_tenant_projects.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0003_tenant_projects.sql"
            )),
        ),
    ]
}

async fn sqlite_pool() -> anyhow::Result<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?)
}

async fn table_exists(pool: &SqlitePool, name: &str) -> anyhow::Result<bool> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(name)
            .fetch_one(pool)
            .await?;
    Ok(count == 1)
}

#[tokio::test]
async fn project_sqlite_migrations_plan_apply_rollback_and_reject_drift() -> anyhow::Result<()> {
    let pool = sqlite_pool().await?;
    let migrations = project_migrations();

    let dry_run = plan_apply(&sqlite_migration_records(&pool).await?, &migrations)?;
    assert_eq!(
        dry_run
            .steps
            .iter()
            .map(|step| (step.version, step.direction))
            .collect::<Vec<_>>(),
        vec![
            (1, MigrationDirection::Up),
            (2, MigrationDirection::Up),
            (3, MigrationDirection::Up),
        ]
    );

    let applied = migrate_sqlite(&pool, &migrations).await?;
    assert_eq!(applied, dry_run);
    for table in ["users", "pets", "groups", "memberships", "projects"] {
        assert!(table_exists(&pool, table).await?, "missing table {table}");
    }
    assert!(plan_apply(&sqlite_migration_records(&pool).await?, &migrations)?.is_empty());

    let partial = rollback_sqlite(&pool, &migrations, 1).await?;
    assert_eq!(
        partial
            .steps
            .iter()
            .map(|step| (step.version, step.direction))
            .collect::<Vec<_>>(),
        vec![(3, MigrationDirection::Down), (2, MigrationDirection::Down)]
    );
    assert!(table_exists(&pool, "users").await?);
    assert!(table_exists(&pool, "pets").await?);
    assert!(!table_exists(&pool, "groups").await?);
    assert!(!table_exists(&pool, "projects").await?);
    assert_eq!(
        sqlite_migration_records(&pool).await?,
        vec![MigrationRecord {
            version: 1,
            name: "init".to_string(),
        }]
    );

    let final_rollback = rollback_sqlite(&pool, &migrations, 0).await?;
    assert_eq!(final_rollback.steps.len(), 1);
    assert!(!table_exists(&pool, "users").await?);
    assert!(!table_exists(&pool, "pets").await?);
    assert!(sqlite_migration_records(&pool).await?.is_empty());

    migrate_sqlite(&pool, &migrations).await?;
    let mut drifted = migrations.clone();
    drifted[0].name = "renamed_init".to_string();
    assert!(matches!(
        plan_apply(&sqlite_migration_records(&pool).await?, &drifted),
        Err(MigrationPlanError::NameDrift { version: 1, .. })
    ));
    Ok(())
}

#[tokio::test]
async fn project_sqlite_migration_execution_is_atomic() -> anyhow::Result<()> {
    let pool = sqlite_pool().await?;
    let mut migrations = project_migrations();
    migrations.push(SqlMigration::new(
        4,
        "transient_table",
        "CREATE TABLE transient_table (id INTEGER PRIMARY KEY)",
        Some("DROP TABLE transient_table"),
    ));
    migrations.push(SqlMigration::new(
        5,
        "invalid_sql",
        "THIS IS NOT VALID SQL",
        Some("SELECT 1"),
    ));

    assert!(migrate_sqlite(&pool, &migrations).await.is_err());
    assert!(sqlite_migration_records(&pool).await?.is_empty());
    for table in [
        "users",
        "pets",
        "groups",
        "memberships",
        "projects",
        "transient_table",
    ] {
        assert!(
            !table_exists(&pool, table).await?,
            "table {table} escaped the failed migration transaction"
        );
    }
    Ok(())
}

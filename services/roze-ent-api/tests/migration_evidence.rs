use roze_migration::{
    migrate_mysql, migrate_postgres, migrate_sqlite, mysql_migration_records, plan_apply,
    postgres_migration_records, rollback_mysql, rollback_postgres, rollback_sqlite,
    sqlite_migration_records, MigrationDirection, MigrationPlanError, MigrationRecord,
    SqlMigration,
};
use sqlx::{
    mysql::MySqlPoolOptions, postgres::PgPoolOptions, sqlite::SqlitePoolOptions, MySqlPool, PgPool,
    SqlitePool,
};

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

fn postgres_migrations() -> Vec<SqlMigration> {
    [
        statement_migrations(
            1,
            "init",
            include_str!("../../../migrations/0001_init.sql"),
            include_str!("../../../migrations/down/0001_init.sql"),
        ),
        statement_migrations(
            2,
            "groups_and_memberships",
            include_str!("../../../migrations/0002_groups_and_memberships.sql"),
            include_str!("../../../migrations/down/0002_groups_and_memberships.sql"),
        ),
        statement_migrations(
            3,
            "tenant_projects",
            include_str!("../../../migrations/0003_tenant_projects.sql"),
            include_str!("../../../migrations/down/0003_tenant_projects.sql"),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn mysql_migrations() -> Vec<SqlMigration> {
    [
        statement_migrations(
            1,
            "init",
            include_str!("../../../migrations/mysql/0001_init.sql"),
            include_str!("../../../migrations/mysql/down/0001_init.sql"),
        ),
        statement_migrations(
            2,
            "groups_and_memberships",
            include_str!("../../../migrations/mysql/0002_groups_and_memberships.sql"),
            include_str!("../../../migrations/mysql/down/0002_groups_and_memberships.sql"),
        ),
        statement_migrations(
            3,
            "tenant_projects",
            include_str!("../../../migrations/mysql/0003_tenant_projects.sql"),
            include_str!("../../../migrations/mysql/down/0003_tenant_projects.sql"),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn statement_migrations(
    file_version: i64,
    file_name: &str,
    up_sql: &str,
    down_sql: &str,
) -> Vec<SqlMigration> {
    let up = split_statements(up_sql);
    let mut down = split_statements(down_sql);
    down.reverse();
    up.into_iter()
        .enumerate()
        .map(|(index, statement)| {
            let statement_number = index as i64 + 1;
            SqlMigration::new(
                file_version * 100 + statement_number,
                format!("{file_name}_{statement_number:02}"),
                statement,
                Some(
                    down.get(index)
                        .cloned()
                        .unwrap_or_else(|| "SELECT 1".to_string()),
                ),
            )
        })
        .collect()
}

fn split_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(str::to_string)
        .collect()
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

async fn postgres_table_count(pool: &PgPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name IN \
         ('users', 'pets', 'groups', 'memberships', 'projects')",
    )
    .fetch_one(pool)
    .await?)
}

async fn mysql_table_count(pool: &MySqlPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = DATABASE() AND table_name IN \
         ('users', 'pets', 'groups', 'memberships', 'projects')",
    )
    .fetch_one(pool)
    .await?)
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

#[tokio::test]
#[ignore = "requires ROZE_ENT_TEST_POSTGRES_URL"]
async fn project_postgres_migrations_apply_and_rollback() -> anyhow::Result<()> {
    let url = std::env::var("ROZE_ENT_TEST_POSTGRES_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let migrations = postgres_migrations();

    let dry_run = plan_apply(&postgres_migration_records(&pool).await?, &migrations)?;
    assert_eq!(dry_run.steps.len(), 8);
    assert_eq!(migrate_postgres(&pool, &migrations).await?, dry_run);
    assert_eq!(postgres_table_count(&pool).await?, 5);
    assert!(plan_apply(&postgres_migration_records(&pool).await?, &migrations)?.is_empty());

    let rollback = rollback_postgres(&pool, &migrations, 0).await?;
    assert_eq!(rollback.steps.len(), 8);
    assert_eq!(postgres_table_count(&pool).await?, 0);
    assert!(postgres_migration_records(&pool).await?.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "requires ROZE_ENT_TEST_MYSQL_URL"]
async fn project_mysql_migrations_apply_and_rollback() -> anyhow::Result<()> {
    let url = std::env::var("ROZE_ENT_TEST_MYSQL_URL")?;
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let migrations = mysql_migrations();

    let dry_run = plan_apply(&mysql_migration_records(&pool).await?, &migrations)?;
    assert_eq!(dry_run.steps.len(), 8);
    assert_eq!(migrate_mysql(&pool, &migrations).await?, dry_run);
    assert_eq!(mysql_table_count(&pool).await?, 5);
    assert!(plan_apply(&mysql_migration_records(&pool).await?, &migrations)?.is_empty());

    let rollback = rollback_mysql(&pool, &migrations, 0).await?;
    assert_eq!(rollback.steps.len(), 8);
    assert_eq!(mysql_table_count(&pool).await?, 0);
    assert!(mysql_migration_records(&pool).await?.is_empty());
    Ok(())
}

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
use std::{fs, path::Path};

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
        SqlMigration::new(
            4,
            "scalar_and_composite_ids",
            include_str!("../../../migrations/sqlite/0004_scalar_and_composite_ids.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0004_scalar_and_composite_ids.sql"
            )),
        ),
        SqlMigration::new(
            5,
            "self_and_named_edges",
            include_str!("../../../migrations/sqlite/0005_self_and_named_edges.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0005_self_and_named_edges.sql"
            )),
        ),
        SqlMigration::new(
            6,
            "multi_schema_audit",
            include_str!("../../../migrations/sqlite/0006_multi_schema_audit.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0006_multi_schema_audit.sql"
            )),
        ),
        SqlMigration::new(
            7,
            "global_unique_ids",
            include_str!("../../../migrations/sqlite/0007_global_unique_ids.sql"),
            Some(include_str!(
                "../../../migrations/sqlite/down/0007_global_unique_ids.sql"
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
        statement_migrations(
            4,
            "scalar_and_composite_ids",
            include_str!("../../../migrations/0004_scalar_and_composite_ids.sql"),
            include_str!("../../../migrations/down/0004_scalar_and_composite_ids.sql"),
        ),
        statement_migrations(
            5,
            "self_and_named_edges",
            include_str!("../../../migrations/0005_self_and_named_edges.sql"),
            include_str!("../../../migrations/down/0005_self_and_named_edges.sql"),
        ),
        statement_migrations(
            6,
            "multi_schema_audit",
            include_str!("../../../migrations/0006_multi_schema_audit.sql"),
            include_str!("../../../migrations/down/0006_multi_schema_audit.sql"),
        ),
        statement_migrations(
            7,
            "global_unique_ids",
            include_str!("../../../migrations/0007_global_unique_ids.sql"),
            include_str!("../../../migrations/down/0007_global_unique_ids.sql"),
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
        statement_migrations(
            4,
            "scalar_and_composite_ids",
            include_str!("../../../migrations/mysql/0004_scalar_and_composite_ids.sql"),
            include_str!("../../../migrations/mysql/down/0004_scalar_and_composite_ids.sql"),
        ),
        statement_migrations(
            5,
            "self_and_named_edges",
            include_str!("../../../migrations/mysql/0005_self_and_named_edges.sql"),
            include_str!("../../../migrations/mysql/down/0005_self_and_named_edges.sql"),
        ),
        statement_migrations(
            6,
            "multi_schema_audit",
            include_str!("../../../migrations/mysql/0006_multi_schema_audit.sql"),
            include_str!("../../../migrations/mysql/down/0006_multi_schema_audit.sql"),
        ),
        statement_migrations(
            7,
            "global_unique_ids",
            include_str!("../../../migrations/mysql/0007_global_unique_ids.sql"),
            include_str!("../../../migrations/mysql/down/0007_global_unique_ids.sql"),
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
    let step_count = up.len().max(down.len());
    (0..step_count)
        .map(|index| {
            let statement_number = index as i64 + 1;
            SqlMigration::new(
                file_version * 100 + statement_number,
                format!("{file_name}_{statement_number:02}"),
                up.get(index)
                    .cloned()
                    .unwrap_or_else(|| "SELECT 1".to_string()),
                Some(
                    down.get(index)
                        .cloned()
                        .unwrap_or_else(|| "SELECT 1".to_string()),
                ),
            )
        })
        .collect()
}

fn migration_file_names(directory: &Path) -> anyhow::Result<Vec<String>> {
    let mut names = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with('0') && name.ends_with(".sql"))
        .collect::<Vec<_>>();
    names.sort();
    Ok(names)
}

#[test]
fn migration_layout_is_complete_and_consistent_across_dialects() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("migrations");
    let postgres = migration_file_names(&root)?;
    assert!(!postgres.is_empty(), "the migration set must not be empty");
    assert_eq!(postgres, migration_file_names(&root.join("mysql"))?);
    assert_eq!(postgres, migration_file_names(&root.join("sqlite"))?);

    for (index, name) in postgres.iter().enumerate() {
        let expected_prefix = format!("{:04}_", index + 1);
        assert!(
            name.starts_with(&expected_prefix),
            "expected contiguous migration {expected_prefix}, found {name}"
        );
        for dialect in [Path::new(""), Path::new("mysql"), Path::new("sqlite")] {
            let directory = root.join(dialect);
            let up = directory.join(name);
            let down = directory.join("down").join(name);
            assert!(
                down.is_file(),
                "missing rollback migration {}",
                down.display()
            );
            for sql_file in [up, down] {
                let sql = fs::read_to_string(&sql_file)?;
                assert!(
                    !sql.trim().is_empty(),
                    "migration {} is empty",
                    sql_file.display()
                );
            }
        }
    }
    Ok(())
}

#[test]
fn global_id_config_is_stable_and_matches_every_dialect() -> anyhow::Result<()> {
    let config = include_str!("../../../model/globalid.toml");
    let names = config
        .lines()
        .filter_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
        .collect::<Vec<_>>();
    let starts = config
        .lines()
        .filter_map(|line| line.strip_prefix("increment_start = "))
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        names,
        [
            "User",
            "Pet",
            "Group",
            "Membership",
            "Friendship",
            "Project",
            "AuditEvent",
        ]
    );
    assert_eq!(starts[0], 1);
    for (index, start) in starts.iter().copied().enumerate().skip(1) {
        assert_eq!(start, (index as i64) << 32);
    }

    let postgres = include_str!("../../../migrations/0007_global_unique_ids.sql");
    let mysql = include_str!("../../../migrations/mysql/0007_global_unique_ids.sql");
    let sqlite = include_str!("../../../migrations/sqlite/0007_global_unique_ids.sql");
    for start in starts.iter().copied().skip(1) {
        assert!(postgres.contains(&start.to_string()));
        assert!(mysql.contains(&start.to_string()));
        assert!(sqlite.contains(&(start - 1).to_string()));
    }
    Ok(())
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
         ('users', 'pets', 'groups', 'memberships', 'projects', \
          'scalar_fixtures', 'locale_settings', 'friendships')",
    )
    .fetch_one(pool)
    .await?)
}

async fn postgres_audit_table_count(pool: &PgPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'roze_ent' AND table_name = 'audit_events'",
    )
    .fetch_one(pool)
    .await?)
}

async fn mysql_table_count(pool: &MySqlPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = DATABASE() AND table_name IN \
         ('users', 'pets', 'groups', 'memberships', 'projects', \
          'scalar_fixtures', 'locale_settings', 'friendships', 'audit_events')",
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
            (4, MigrationDirection::Up),
            (5, MigrationDirection::Up),
            (6, MigrationDirection::Up),
            (7, MigrationDirection::Up),
        ]
    );

    let applied = migrate_sqlite(&pool, &migrations).await?;
    assert_eq!(applied, dry_run);
    for table in [
        "users",
        "pets",
        "groups",
        "memberships",
        "projects",
        "scalar_fixtures",
        "locale_settings",
        "friendships",
        "audit_events",
    ] {
        assert!(table_exists(&pool, table).await?, "missing table {table}");
    }
    let user_id =
        sqlx::query("INSERT INTO users (email, name, active, created_at) VALUES (?, ?, TRUE, 1)")
            .bind("global-id-sqlite@example.com")
            .bind("global id")
            .execute(&pool)
            .await?
            .last_insert_rowid();
    let pet_id = sqlx::query("INSERT INTO pets (owner_id, name, species) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind("global id pet")
        .bind("other")
        .execute(&pool)
        .await?
        .last_insert_rowid();
    let group_id = sqlx::query("INSERT INTO groups (name, created_at) VALUES (?, 1)")
        .bind("global id group")
        .execute(&pool)
        .await?
        .last_insert_rowid();
    assert_eq!(user_id, 1);
    assert_eq!(pet_id, 1_i64 << 32);
    assert_eq!(group_id, 2_i64 << 32);
    assert_eq!(
        std::collections::HashSet::from([user_id, pet_id, group_id]).len(),
        3
    );
    assert!(plan_apply(&sqlite_migration_records(&pool).await?, &migrations)?.is_empty());

    let partial = rollback_sqlite(&pool, &migrations, 1).await?;
    assert_eq!(
        partial
            .steps
            .iter()
            .map(|step| (step.version, step.direction))
            .collect::<Vec<_>>(),
        vec![
            (7, MigrationDirection::Down),
            (6, MigrationDirection::Down),
            (5, MigrationDirection::Down),
            (4, MigrationDirection::Down),
            (3, MigrationDirection::Down),
            (2, MigrationDirection::Down),
        ]
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
        8,
        "transient_table",
        "CREATE TABLE transient_table (id INTEGER PRIMARY KEY)",
        Some("DROP TABLE transient_table"),
    ));
    migrations.push(SqlMigration::new(
        9,
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
        "scalar_fixtures",
        "locale_settings",
        "friendships",
        "audit_events",
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
    assert_eq!(dry_run.steps.len(), 24);
    assert_eq!(migrate_postgres(&pool, &migrations).await?, dry_run);
    assert_eq!(postgres_table_count(&pool).await?, 8);
    assert_eq!(postgres_audit_table_count(&pool).await?, 1);
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO public.users (email, name, active, created_at) \
         VALUES ($1, $2, TRUE, 1) RETURNING id",
    )
    .bind("multi-schema-postgres@example.com")
    .bind("multi schema")
    .fetch_one(&pool)
    .await?;
    assert_eq!(user_id, 1);
    let group_id: i64 = sqlx::query_scalar(
        "INSERT INTO public.groups (name, created_at) VALUES ($1, 1) RETURNING id",
    )
    .bind("global id postgres")
    .fetch_one(&pool)
    .await?;
    assert_eq!(group_id, 2_i64 << 32);
    sqlx::query(
        "INSERT INTO roze_ent.audit_events (user_id, action, created_at) VALUES ($1, $2, 1)",
    )
    .bind(user_id)
    .bind("created")
    .execute(&pool)
    .await?;
    let action: String = sqlx::query_scalar(
        "SELECT event.action FROM roze_ent.audit_events event \
         JOIN public.users users ON users.id = event.user_id WHERE users.id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(action, "created");
    assert!(plan_apply(&postgres_migration_records(&pool).await?, &migrations)?.is_empty());

    let rollback = rollback_postgres(&pool, &migrations, 0).await?;
    assert_eq!(rollback.steps.len(), 24);
    assert_eq!(postgres_table_count(&pool).await?, 0);
    assert_eq!(postgres_audit_table_count(&pool).await?, 0);
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
    assert_eq!(dry_run.steps.len(), 23);
    assert_eq!(migrate_mysql(&pool, &migrations).await?, dry_run);
    assert_eq!(mysql_table_count(&pool).await?, 9);
    let inserted =
        sqlx::query("INSERT INTO users (email, name, active, created_at) VALUES (?, ?, TRUE, 1)")
            .bind("multi-schema-mysql@example.com")
            .bind("multi schema")
            .execute(&pool)
            .await?;
    let user_id = inserted.last_insert_id();
    assert_eq!(user_id, 1);
    let group_id = sqlx::query("INSERT INTO `groups` (name, created_at) VALUES (?, 1)")
        .bind("global id mysql")
        .execute(&pool)
        .await?
        .last_insert_id();
    assert_eq!(group_id, 2_u64 << 32);
    sqlx::query("INSERT INTO roze_ent.audit_events (user_id, action, created_at) VALUES (?, ?, 1)")
        .bind(user_id)
        .bind("created")
        .execute(&pool)
        .await?;
    let action: String = sqlx::query_scalar(
        "SELECT event.action FROM roze_ent.audit_events event \
         JOIN roze_ent.users users ON users.id = event.user_id WHERE users.id = ?",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(action, "created");
    assert!(plan_apply(&mysql_migration_records(&pool).await?, &migrations)?.is_empty());

    let rollback = rollback_mysql(&pool, &migrations, 0).await?;
    assert_eq!(rollback.steps.len(), 23);
    assert_eq!(mysql_table_count(&pool).await?, 0);
    assert!(mysql_migration_records(&pool).await?.is_empty());
    Ok(())
}

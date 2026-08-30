#![allow(dead_code)]

// Application-owned extension methods for `users`.
// This file is created by rozectl but preserved during `--update`.
use super::user::{Model, UserRepository};

impl Model {
    // Add domain helpers for generated SeaORM model rows here.
}

impl<'a> UserRepository<'a> {
    // Add application-owned repository queries here.
}

#[cfg(test)]
mod tests {
    use sea_orm::ConnectionTrait as _;

    async fn database_context(url: &str) -> anyhow::Result<crate::svc::ServiceContext> {
        let config: crate::config::Config = serde_json::from_value(serde_json::json!({
            "name": "roze-ent-query-parity-test",
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

    async fn sqlite_context() -> anyhow::Result<crate::svc::ServiceContext> {
        let ctx = database_context("sqlite::memory:").await?;
        let db = ctx.write_db()?;
        for statement in [
            "CREATE TABLE users (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                email TEXT NOT NULL UNIQUE, \
                name TEXT NOT NULL, \
                active BOOLEAN NOT NULL DEFAULT TRUE, \
                created_at INTEGER NOT NULL, \
                manager_id INTEGER NULL REFERENCES users(id) ON DELETE SET NULL\
            )",
            "CREATE TABLE groups (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                name TEXT NOT NULL UNIQUE, \
                description TEXT NULL, \
                created_at INTEGER NOT NULL\
            )",
            "CREATE TABLE memberships (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                user_id INTEGER NOT NULL, \
                group_id INTEGER NOT NULL, \
                role TEXT NOT NULL DEFAULT 'member', \
                joined_at INTEGER NOT NULL, \
                UNIQUE (user_id, group_id)\
            )",
            "CREATE TABLE friendships (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, \
                friend_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, \
                created_at INTEGER NOT NULL, \
                UNIQUE (user_id, friend_id), \
                CHECK (user_id <> friend_id)\
            )",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(ctx)
    }

    async fn assert_string_predicate_semantics(
        ctx: &crate::svc::ServiceContext,
        marker: &str,
    ) -> anyhow::Result<()> {
        use crate::model::user;

        let users = ctx.model().user();
        let email_prefix = format!("roze-ent-like-{marker}-");
        let rows = [
            ("alpha", format!("{marker}-AlphaBeta"), 100),
            ("omega", format!("{marker}-BetaOmega"), 200),
            ("mixed", format!("{marker}-MiXeDCase"), 300),
            ("literal", format!(r"{marker}-A%_\\Z"), 400),
        ];
        let mut ids = Vec::with_capacity(rows.len());
        for (email_suffix, name, created_at) in rows {
            let model = users
                .create()
                .set_email(format!("{email_prefix}{email_suffix}@example.com"))
                .set_name(name)
                .set_created_at(created_at)
                .save()
                .await?;
            ids.push(model.id);
        }

        let scoped = |predicate| {
            user::and(vec![
                user::email_starts_with(email_prefix.clone()),
                predicate,
            ])
        };
        assert_eq!(
            users
                .query()
                .where_(scoped(user::name_starts_with(format!("{marker}-Alpha"))))
                .only_name()
                .await?,
            format!("{marker}-AlphaBeta")
        );
        assert_eq!(
            users
                .query()
                .where_(scoped(user::name_ends_with("Omega")))
                .only_name()
                .await?,
            format!("{marker}-BetaOmega")
        );
        assert_eq!(
            users
                .query()
                .where_(scoped(user::name_equal_fold(format!(
                    "{}-mixedcase",
                    marker.to_lowercase()
                ))))
                .only_name()
                .await?,
            format!("{marker}-MiXeDCase")
        );
        assert_eq!(
            users
                .query()
                .where_(scoped(user::name_icontains("XeDc")))
                .only_name()
                .await?,
            format!("{marker}-MiXeDCase")
        );
        assert_eq!(
            users
                .query()
                .where_(scoped(user::name_contains(r"%_\\")))
                .only_name()
                .await?,
            format!(r"{marker}-A%_\\Z")
        );
        for predicate in [
            user::name_not_starts_with(format!("{marker}-Alpha")),
            user::name_not_ends_with("Omega"),
            user::name_not_equal_fold(format!("{}-mixedcase", marker.to_lowercase())),
            user::name_not_icontains("XeDc"),
            user::name_not_contains(r"%_\\"),
        ] {
            assert_eq!(users.query().where_(scoped(predicate)).count().await?, 3);
        }

        users.delete_many_by_ids(ids).await?;
        Ok(())
    }

    #[tokio::test]
    async fn ent_style_query_surface_has_real_sqlite_evidence() -> anyhow::Result<()> {
        use crate::model::{group, user};

        let ctx = sqlite_context().await?;
        let models = ctx.model();
        let users = models.user();
        let groups = models.group();
        let memberships = models.membership();

        let alice = users
            .create()
            .set_email("alice@example.com".to_string())
            .set_name("Alice".to_string())
            .set_active(true)
            .set_created_at(100)
            .save()
            .await?;
        let alicia = users
            .create()
            .set_email("alicia@example.com".to_string())
            .set_name("ALICIA".to_string())
            .set_active(false)
            .set_created_at(200)
            .save()
            .await?;
        let bob = users
            .create()
            .set_email("bob@example.com".to_string())
            .set_name("Bob".to_string())
            .set_active(true)
            .set_created_at(300)
            .save()
            .await?;

        let matched = users
            .query()
            .where_(user::and(vec![
                user::name_contains("ali"),
                user::created_at_between(50, 250),
            ]))
            .where_(user::not(user::active_eq(false)))
            .only()
            .await?;
        assert_eq!(matched.id, alice.id);
        assert_eq!(
            users
                .query()
                .where_(user::name_icontains("LiCe"))
                .only_id()
                .await?,
            alice.id
        );
        assert_eq!(
            users
                .query()
                .where_(user::name_equal_fold("aLiCiA"))
                .only_id()
                .await?,
            alicia.id
        );

        let selected_ids = users
            .query()
            .where_(user::or(vec![
                user::email_in(vec!["alicia@example.com".to_string()]),
                user::name_eq("Bob".to_string()),
            ]))
            .order_by_created_at_asc()
            .ids()
            .await?;
        assert_eq!(selected_ids, vec![alicia.id, bob.id]);

        let page = users
            .query()
            .order_by_created_at_asc()
            .paginate(2, 1)
            .page()
            .await?;
        assert_eq!(page.total, 3);
        assert_eq!((page.page, page.page_size), (2, 1));
        assert_eq!(page.items[0].id, alicia.id);
        assert_eq!(
            users
                .query()
                .order_by_created_at_desc()
                .offset(1)
                .limit(1)
                .first_id()
                .await?,
            Some(alicia.id)
        );

        assert_eq!(
            users
                .query()
                .where_(user::email_eq("bob@example.com".to_string()))
                .only_name()
                .await?,
            "Bob"
        );
        assert_eq!(
            users.query().where_(user::active_eq(true)).count().await?,
            2
        );
        let mut counts = users.query().count_by_active().await?;
        counts.sort_by_key(|(active, _)| *active);
        assert_eq!(counts, vec![(false, 1), (true, 2)]);
        assert_eq!(users.query().sum_created_at().await?, 600);
        assert_eq!(users.query().avg_created_at().await?, Some(200.0));
        assert_eq!(users.query().min_created_at().await?, Some(100));
        assert_eq!(users.query().max_created_at().await?, Some(300));

        let admins = groups
            .create()
            .set_name("Admins".to_string())
            .set_description(Some("Production access".to_string()))
            .set_created_at(400)
            .save()
            .await?;
        let engineering = groups
            .create()
            .set_name("Engineering".to_string())
            .clear_description()
            .set_created_at(500)
            .save()
            .await?;
        alice.add_groups(&admins, &memberships).await?;
        alice.add_groups(&engineering, &memberships).await?;
        alicia.add_groups(&engineering, &memberships).await?;

        assert_eq!(
            groups
                .query()
                .where_(group::description_is_null())
                .only_description()
                .await?,
            None
        );
        assert_eq!(
            groups
                .query()
                .order_by_name_asc()
                .pluck_description()
                .await?,
            vec![Some("Production access".to_string()), None]
        );

        let alice_groups = alice
            .traverse_groups(&memberships, &groups)
            .await?
            .order_by_name_asc()
            .pluck_name()
            .await?;
        assert_eq!(alice_groups, vec!["Admins", "Engineering"]);
        let engineering_users = engineering
            .traverse_users(&memberships, &users)
            .await?
            .order_by_name_asc()
            .pluck_name()
            .await?;
        assert_eq!(engineering_users, vec!["ALICIA", "Alice"]);

        let admin_users = users
            .query()
            .where_groups_with(
                &groups,
                &memberships,
                [group::name_eq("Admins".to_string())],
            )
            .await?
            .only_id()
            .await?;
        assert_eq!(admin_users, alice.id);

        let loaded = users
            .query()
            .order_by_id_asc()
            .all_with_groups(&memberships, &groups)
            .await?;
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].edge.len(), 2);
        assert_eq!(loaded[1].edge.len(), 1);
        assert!(loaded[2].edge.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn string_predicates_preserve_literal_wildcards_on_sqlite() -> anyhow::Result<()> {
        let ctx = sqlite_context().await?;
        assert_string_predicate_semantics(&ctx, "sqlite").await
    }

    #[tokio::test]
    #[ignore = "requires ROZE_ENT_TEST_DATABASE_URL and an applied project schema"]
    async fn string_predicates_have_real_external_sql_evidence() -> anyhow::Result<()> {
        let url = std::env::var("ROZE_ENT_TEST_DATABASE_URL")?;
        let marker = format!(
            "external-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        );
        let ctx = database_context(&url).await?;
        assert_string_predicate_semantics(&ctx, &marker).await
    }

    #[tokio::test]
    async fn ent_style_mutation_surface_has_real_sqlite_evidence() -> anyhow::Result<()> {
        use crate::model::user;

        let ctx = sqlite_context().await?;
        let models = ctx.model();
        let users = models.user();

        let missing_required = users
            .create()
            .set_name("Missing email".to_string())
            .save()
            .await
            .unwrap_err();
        assert!(missing_required
            .to_string()
            .contains("missing required field `email`"));
        let invalid = users
            .create()
            .set_email(String::new())
            .set_name("Invalid".to_string())
            .save()
            .await
            .unwrap_err();
        assert!(invalid.to_string().contains("email validation failed"));
        assert_eq!(users.count().await?, 0);

        let alice = users
            .create()
            .set_email("alice@example.com".to_string())
            .set_name("Alice".to_string())
            .set_active(false)
            .set_created_at(100)
            .save()
            .await?;
        let bob = users
            .create()
            .set_email("bob@example.com".to_string())
            .set_name("Bob".to_string())
            .set_created_at(200)
            .save()
            .await?;

        let duplicate = users
            .create()
            .set_email("alice@example.com".to_string())
            .set_name("Duplicate".to_string())
            .save()
            .await;
        assert!(duplicate.is_err());
        assert_eq!(users.count().await?, 2);

        let alice = users
            .update_one(alice.id)
            .set_name("Alice Updated".to_string())
            .save()
            .await?;
        assert_eq!(alice.name, "Alice Updated");
        assert_eq!(alice.email, "alice@example.com");

        let activated = users
            .update_many()
            .where_(user::active_eq(false))
            .set_active(true)
            .save()
            .await?;
        assert_eq!(activated.len(), 1);
        assert_eq!(activated[0].id, alice.id);
        assert!(activated[0].active);

        let deactivated = users
            .update_where()
            .where_(user::email_eq("bob@example.com".to_string()))
            .set_active(false)
            .execute()
            .await?;
        assert_eq!(deactivated.rows_affected, 1);
        assert!(users
            .update_where()
            .where_(user::id_eq(alice.id))
            .execute()
            .await
            .unwrap_err()
            .to_string()
            .contains("requires at least one field assignment"));

        users
            .insert_many(vec![
                user::Model {
                    id: 100,
                    email: "batch-100@example.com".to_string(),
                    name: "Batch 100".to_string(),
                    active: true,
                    created_at: 300,
                    manager_id: None,
                },
                user::Model {
                    id: 101,
                    email: "batch-101@example.com".to_string(),
                    name: "Batch 101".to_string(),
                    active: true,
                    created_at: 400,
                    manager_id: None,
                },
            ])
            .await?;
        assert_eq!(users.count().await?, 4);

        let upserted = users
            .upsert(user::Model {
                id: 102,
                email: "batch-102@example.com".to_string(),
                name: "Batch 102 Upserted".to_string(),
                active: false,
                created_at: 450,
                manager_id: None,
            })
            .await?;
        assert_eq!(upserted.id, 102);
        assert_eq!(upserted.name, "Batch 102 Upserted");
        assert!(!upserted.active);

        let conflict_upserted = users
            .upsert(user::Model {
                id: 102,
                email: "batch-102-updated@example.com".to_string(),
                name: "Batch 102 Conflict Updated".to_string(),
                active: true,
                created_at: 999,
                manager_id: None,
            })
            .await?;
        assert_eq!(conflict_upserted.id, 102);
        assert_eq!(conflict_upserted.email, "batch-102-updated@example.com");
        assert_eq!(conflict_upserted.name, "Batch 102 Conflict Updated");
        assert!(conflict_upserted.active);
        assert_eq!(conflict_upserted.created_at, 450);
        assert_eq!(
            users.query().where_(user::id_eq(102)).only().await?,
            conflict_upserted
        );

        let deleted_one = users.delete_one(bob.id).exec().await?;
        assert_eq!(deleted_one.rows_affected, 1);
        assert_eq!(
            users
                .delete_many()
                .where_(user::name_contains("Batch"))
                .exec()
                .await?,
            3
        );

        users
            .insert_many(vec![
                user::Model {
                    id: 200,
                    email: "bulk-200@example.com".to_string(),
                    name: "Bulk 200".to_string(),
                    active: true,
                    created_at: 500,
                    manager_id: None,
                },
                user::Model {
                    id: 201,
                    email: "bulk-201@example.com".to_string(),
                    name: "Bulk 201".to_string(),
                    active: true,
                    created_at: 600,
                    manager_id: None,
                },
            ])
            .await?;
        let deleted_many = users.delete_many_by_ids(vec![200, 201]).await?;
        assert_eq!(deleted_many.rows_affected, 2);
        assert_eq!(users.query().only_id().await?, alice.id);
        Ok(())
    }

    #[tokio::test]
    async fn self_bidirectional_and_named_edges_have_real_sqlite_evidence() -> anyhow::Result<()> {
        use crate::model::user;

        let ctx = sqlite_context().await?;
        let models = ctx.model();
        let users = models.user();
        let friendships = models.friendship();

        let manager = users
            .create()
            .set_email("manager@example.com".to_string())
            .set_name("Manager".to_string())
            .set_created_at(100)
            .save()
            .await?;
        let alice = users
            .create()
            .set_email("alice@example.com".to_string())
            .set_name("Alice".to_string())
            .set_created_at(200)
            .set_manager(&manager)
            .save()
            .await?;
        let bob = users
            .create()
            .set_email("bob@example.com".to_string())
            .set_name("Bob".to_string())
            .set_created_at(300)
            .set_manager(&manager)
            .save()
            .await?;
        let carol = users
            .create()
            .set_email("carol@example.com".to_string())
            .set_name("Carol".to_string())
            .set_created_at(400)
            .save()
            .await?;

        assert_eq!(alice.query_manager(&users).await?.unwrap().id, manager.id);
        assert_eq!(
            manager
                .traverse_reports(&users)
                .await?
                .order_by_name_asc()
                .pluck_name()
                .await?,
            vec!["Alice", "Bob"]
        );

        let nested = users
            .query()
            .where_(user::id_eq(alice.id))
            .all_with_manager_then_reports(&users, &users)
            .await?;
        let manager_with_reports = nested[0].edge.as_ref().unwrap();
        assert_eq!(manager_with_reports.node.id, manager.id);
        assert_eq!(manager_with_reports.edge.len(), 2);

        alice.add_friends(&bob, &friendships).await?;
        alice.add_friends(&carol, &friendships).await?;
        assert!(alice.add_friends(&alice, &friendships).await.is_err());
        assert_eq!(
            alice
                .traverse_friends(&friendships, &users)
                .await?
                .order_by_name_asc()
                .pluck_name()
                .await?,
            vec!["Bob", "Carol"]
        );
        assert_eq!(
            bob.query_friended_by(&friendships, &users).await?[0].id,
            alice.id
        );
        assert_eq!(
            users
                .query()
                .where_friends_with(&users, &friendships, [user::name_eq("Carol".to_string())],)
                .await?
                .only_id()
                .await?,
            alice.id
        );

        let named = friendships
            .query()
            .order_by_friend_id_asc()
            .all_with_user_and_friend(&users, &users)
            .await?;
        assert_eq!(named.len(), 2);
        assert_eq!(named[0].user.as_ref().unwrap().id, alice.id);
        assert_eq!(named[0].friend.as_ref().unwrap().id, bob.id);

        assert_eq!(alice.remove_friends(&bob, &friendships).await?, 1);
        assert_eq!(alice.clear_friends(&friendships).await?, 1);
        assert!(alice.query_friends(&friendships, &users).await?.is_empty());

        let alice = users.update_one(alice.id).clear_manager().save().await?;
        assert_eq!(alice.manager_id, None);
        assert!(alice.query_manager(&users).await?.is_none());
        Ok(())
    }
}

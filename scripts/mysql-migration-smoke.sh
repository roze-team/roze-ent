#!/usr/bin/env bash
set -euo pipefail

cleanup() {
  if [[ "${ROZE_ENT_REMOVE_VOLUMES:-0}" = "1" ]]; then
    docker compose down --volumes >/dev/null 2>&1 || true
  else
    docker compose down >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

docker compose up -d mysql

for _ in $(seq 1 90); do
  if docker compose exec -T mysql mysqladmin ping -h 127.0.0.1 -uroot -proot --silent \
    >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
docker compose exec -T mysql mysqladmin ping -h 127.0.0.1 -uroot -proot --silent \
  >/dev/null

export ROZE_ENT_TEST_MYSQL_URL="mysql://roze:roze@127.0.0.1:${ROZE_ENT_MYSQL_PORT:-3306}/roze_ent"
cargo test -p roze-ent-api --test migration_evidence \
  project_mysql_migrations_apply_and_rollback -- --ignored

for migration in migrations/mysql/0*.sql; do
  docker compose exec -T mysql mysql -uroze -proze roze_ent <"${migration}" >/dev/null
done

export ROZE_ENT_TEST_DATABASE_URL="${ROZE_ENT_TEST_MYSQL_URL}"
cargo test -p roze-ent-api \
  model::user_ext::tests::generated_creates_use_ent_compatible_global_id_ranges -- --ignored --exact
cargo test -p roze-ent-api \
  model::user_ext::tests::string_predicates_have_real_external_sql_evidence -- --ignored --exact
cargo test -p roze-ent-api \
  model::user_ext::tests::upsert_and_pessimistic_locks_have_real_external_sql_evidence -- --ignored --exact
cargo test -p roze-ent-api \
  model::audit_event_ext::tests::generated_multi_schema_model_and_cross_schema_edge_have_real_sql_evidence -- --ignored --exact

export DATABASE_URL="${ROZE_ENT_TEST_MYSQL_URL}"
bash scripts/service-api-smoke.sh mysql

echo "mysql smoke passed"

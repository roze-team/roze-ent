#!/usr/bin/env bash
set -euo pipefail

migration_db="roze_ent_migration_test"

cleanup() {
  docker compose exec -T postgres psql -U roze -d postgres \
    -c "DROP DATABASE IF EXISTS ${migration_db} WITH (FORCE)" >/dev/null 2>&1 || true
  if [[ "${ROZE_ENT_REMOVE_VOLUMES:-0}" = "1" ]]; then
    docker compose down --volumes >/dev/null 2>&1 || true
  else
    docker compose down >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

docker compose up -d postgres

ready_checks=0
for _ in $(seq 1 60); do
  if docker compose exec -T postgres pg_isready -U roze -d roze_ent >/dev/null 2>&1; then
    ready_checks=$((ready_checks + 1))
    if [[ "${ready_checks}" -ge 2 ]]; then
      break
    fi
  else
    ready_checks=0
  fi
  sleep 1
done
test "${ready_checks}" -ge 2
docker compose exec -T postgres pg_isready -U roze -d roze_ent >/dev/null

docker compose exec -T postgres psql -v ON_ERROR_STOP=1 -U roze -d postgres \
  -c "DROP DATABASE IF EXISTS ${migration_db} WITH (FORCE)" >/dev/null
docker compose exec -T postgres psql -v ON_ERROR_STOP=1 -U roze -d postgres \
  -c "CREATE DATABASE ${migration_db}" >/dev/null

export ROZE_ENT_TEST_POSTGRES_URL="postgres://roze:roze@127.0.0.1:${ROZE_ENT_POSTGRES_PORT:-5432}/${migration_db}"
cargo test -p roze-ent-api --test migration_evidence \
  project_postgres_migrations_apply_and_rollback -- --ignored
docker compose exec -T postgres psql -v ON_ERROR_STOP=1 -U roze -d postgres \
  -c "DROP DATABASE IF EXISTS ${migration_db} WITH (FORCE)" >/dev/null

for migration in migrations/0*.sql; do
  docker compose exec -T postgres psql -v ON_ERROR_STOP=1 -U roze -d roze_ent <"${migration}" >/dev/null
done

export ROZE_ENT_TEST_DATABASE_URL="postgres://roze:roze@127.0.0.1:${ROZE_ENT_POSTGRES_PORT:-5432}/roze_ent"
cargo test -p roze-ent-api \
  application::user_activity_view::tests::generated_view_is_query_only_and_has_real_sql_evidence -- --ignored --exact
cargo test -p roze-ent-api \
  model::user_ext::tests::generated_creates_use_ent_compatible_global_id_ranges -- --ignored --exact
cargo test -p roze-ent-api \
  model::user_ext::tests::string_predicates_have_real_external_sql_evidence -- --ignored --exact
cargo test -p roze-ent-api \
  model::user_ext::tests::upsert_and_pessimistic_locks_have_real_external_sql_evidence -- --ignored --exact
cargo test -p roze-ent-api \
  model::audit_event_ext::tests::generated_multi_schema_model_and_cross_schema_edge_have_real_sql_evidence -- --ignored --exact

export DATABASE_URL="postgres://roze:roze@127.0.0.1:${ROZE_ENT_POSTGRES_PORT:-5432}/roze_ent"
bash scripts/service-api-smoke.sh postgres

echo "postgres smoke passed"

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

echo "mysql migration smoke passed"

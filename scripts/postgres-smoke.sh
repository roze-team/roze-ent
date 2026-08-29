#!/usr/bin/env bash
set -euo pipefail

service_pid=""

cleanup() {
  if [[ -n "${service_pid}" ]]; then
    kill "${service_pid}" 2>/dev/null || true
    wait "${service_pid}" 2>/dev/null || true
  fi
  docker compose down >/dev/null 2>&1 || true
}
trap cleanup EXIT

docker compose up -d postgres

for _ in $(seq 1 60); do
  if docker compose exec -T postgres pg_isready -U roze -d roze_ent >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
docker compose exec -T postgres pg_isready -U roze -d roze_ent >/dev/null

for migration in migrations/0*.sql; do
  docker compose exec -T postgres psql -v ON_ERROR_STOP=1 -U roze -d roze_ent <"${migration}" >/dev/null
done

mkdir -p target
export DATABASE_URL="postgres://roze:roze@127.0.0.1:5432/roze_ent"
export ROZE_CONFIG_PATH="services/roze-ent-api/config.yaml"
cargo run -p roze-ent-api >target/postgres-smoke-service.log 2>&1 &
service_pid=$!

for _ in $(seq 1 180); do
  if curl --fail --silent http://127.0.0.1:3000/healthz >/dev/null; then
    break
  fi
  if ! kill -0 "${service_pid}" 2>/dev/null; then
    cat target/postgres-smoke-service.log
    exit 1
  fi
  sleep 1
done
curl --fail --silent http://127.0.0.1:3000/healthz >/dev/null

project_name="smoke-$(date +%s)-${RANDOM}"
created=$(curl --fail --silent \
  -X POST http://127.0.0.1:3000/api/v1/projects \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"name\":\"${project_name}\",\"description\":\"smoke\"}")
project_id=$(printf '%s' "${created}" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
if [[ -z "${project_id}" ]]; then
  printf '%s\n' "${created}"
  exit 1
fi

wrong_tenant_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H 'x-tenant-id: tenant-b' \
  "http://127.0.0.1:3000/api/v1/projects/${project_id}")
test "${wrong_tenant_status}" = "404"

curl --fail --silent \
  -X PATCH "http://127.0.0.1:3000/api/v1/projects/${project_id}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"expected_version\":1,\"name\":\"${project_name}-updated\",\"description\":null}" \
  >/dev/null

stale_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -X PATCH "http://127.0.0.1:3000/api/v1/projects/${project_id}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"expected_version\":1,\"name\":\"${project_name}-stale\",\"description\":null}")
test "${stale_status}" = "412"

curl --fail --silent \
  -X DELETE \
  -H 'x-tenant-id: tenant-a' \
  "http://127.0.0.1:3000/api/v1/projects/${project_id}" \
  >/dev/null

deleted_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H 'x-tenant-id: tenant-a' \
  "http://127.0.0.1:3000/api/v1/projects/${project_id}")
test "${deleted_status}" = "404"

echo "postgres smoke passed"

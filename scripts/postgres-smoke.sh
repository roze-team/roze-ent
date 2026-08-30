#!/usr/bin/env bash
set -euo pipefail

service_pid=""
migration_db="roze_ent_migration_test"

cleanup() {
  if [[ -n "${service_pid}" ]]; then
    kill "${service_pid}" 2>/dev/null || true
    wait "${service_pid}" 2>/dev/null || true
  fi
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
  model::user_ext::tests::string_predicates_have_real_external_sql_evidence -- --ignored --exact

mkdir -p target
export DATABASE_URL="postgres://roze:roze@127.0.0.1:${ROZE_ENT_POSTGRES_PORT:-5432}/roze_ent"
export ROZE_CONFIG_PATH="services/roze-ent-api/config.yaml"
export ROZE_JWT_SECRET="${ROZE_ENT_SMOKE_JWT_SECRET:-roze-ent-smoke-secret-at-least-32-bytes}"
cargo run -p roze-ent-api >target/postgres-smoke-service.log 2>&1 &
service_pid=$!
readiness_url="http://127.0.0.1:3000/api/v1/readyz"

for _ in $(seq 1 180); do
  if curl --fail --silent "${readiness_url}" >/dev/null; then
    break
  fi
  if ! kill -0 "${service_pid}" 2>/dev/null; then
    cat target/postgres-smoke-service.log
    exit 1
  fi
  sleep 1
done
if ! curl --fail --silent "${readiness_url}" >/dev/null; then
  cat target/postgres-smoke-service.log
  exit 1
fi

jwt_iat=$(date +%s)
jwt_exp=$((jwt_iat + 600))
base64url() {
  openssl base64 -A | tr '+/' '-_' | tr -d '='
}
jwt_header=$(printf '%s' '{"alg":"HS256","typ":"JWT","kid":"development-v1"}' | base64url)
jwt_payload=$(printf '{"sub":"postgres-smoke","roles":[],"tenant":"tenant-a","permissions":["projects:read","projects:write"],"scopes":[],"iss":"roze-ent","aud":"roze-ent","jti":"postgres-smoke-%s","iat":%s,"exp":%s}' \
  "${jwt_iat}" "${jwt_iat}" "${jwt_exp}" | base64url)
jwt_signing_input="${jwt_header}.${jwt_payload}"
jwt_signature=$(printf '%s' "${jwt_signing_input}" \
  | openssl dgst -sha256 -hmac "${ROZE_JWT_SECRET}" -binary \
  | base64url)
auth_header="authorization: Bearer ${jwt_signing_input}.${jwt_signature}"

unauthenticated_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H 'x-tenant-id: tenant-a' \
  http://127.0.0.1:3000/api/v1/projects)
test "${unauthenticated_status}" = "401"

project_name="smoke-$(date +%s)-${RANDOM}"
created=$(curl --fail --silent \
  -X POST http://127.0.0.1:3000/api/v1/projects \
  -H "${auth_header}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"name\":\"${project_name}\",\"description\":\"smoke\"}")
project_id=$(printf '%s' "${created}" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
if [[ -z "${project_id}" ]]; then
  printf '%s\n' "${created}"
  exit 1
fi

wrong_tenant_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H "${auth_header}" \
  -H 'x-tenant-id: tenant-b' \
  "http://127.0.0.1:3000/api/v1/projects/${project_id}")
test "${wrong_tenant_status}" = "403"

curl --fail --silent \
  -X PATCH "http://127.0.0.1:3000/api/v1/projects/${project_id}" \
  -H "${auth_header}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"expected_version\":1,\"name\":\"${project_name}-updated\",\"description\":null}" \
  >/dev/null

stale_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -X PATCH "http://127.0.0.1:3000/api/v1/projects/${project_id}" \
  -H "${auth_header}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"expected_version\":1,\"name\":\"${project_name}-stale\",\"description\":null}")
test "${stale_status}" = "412"

curl --fail --silent \
  -X DELETE \
  -H "${auth_header}" \
  -H 'x-tenant-id: tenant-a' \
  "http://127.0.0.1:3000/api/v1/projects/${project_id}" \
  >/dev/null

deleted_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H "${auth_header}" \
  -H 'x-tenant-id: tenant-a' \
  "http://127.0.0.1:3000/api/v1/projects/${project_id}")
test "${deleted_status}" = "404"

echo "postgres smoke passed"

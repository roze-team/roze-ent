#!/usr/bin/env bash
set -euo pipefail

database_name="${1:?usage: service-api-smoke.sh <database-name>}"
service_pid=""
log_file="target/${database_name}-smoke-service.log"
base_url="${ROZE_ENT_SMOKE_BASE_URL:-http://127.0.0.1:3000}"

cleanup() {
  if [[ -n "${service_pid}" ]]; then
    kill "${service_pid}" 2>/dev/null || true
    wait "${service_pid}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

: "${DATABASE_URL:?DATABASE_URL must point at the smoke-test database}"
export ROZE_CONFIG_PATH="${ROZE_CONFIG_PATH:-services/roze-ent-api/config.yaml}"
export ROZE_JWT_SECRET="${ROZE_ENT_SMOKE_JWT_SECRET:-roze-ent-smoke-secret-at-least-32-bytes}"

mkdir -p target
cargo run -p roze-ent-api >"${log_file}" 2>&1 &
service_pid=$!
readiness_url="${base_url}/api/v1/readyz"

for _ in $(seq 1 180); do
  if curl --fail --silent "${readiness_url}" >/dev/null; then
    break
  fi
  if ! kill -0 "${service_pid}" 2>/dev/null; then
    cat "${log_file}"
    exit 1
  fi
  sleep 1
done
if ! curl --fail --silent "${readiness_url}" >/dev/null; then
  cat "${log_file}"
  exit 1
fi

base64url() {
  openssl base64 -A | tr '+/' '-_' | tr -d '='
}

jwt_iat=$(date +%s)
jwt_exp=$((jwt_iat + 600))
jwt_header=$(printf '%s' '{"alg":"HS256","typ":"JWT","kid":"development-v1"}' | base64url)
jwt_payload=$(printf '{"sub":"%s-smoke","roles":[],"tenant":"tenant-a","permissions":["projects:read","projects:write"],"scopes":[],"iss":"roze-ent","aud":"roze-ent","jti":"%s-smoke-%s","iat":%s,"exp":%s}' \
  "${database_name}" "${database_name}" "${jwt_iat}" "${jwt_iat}" "${jwt_exp}" | base64url)
jwt_signing_input="${jwt_header}.${jwt_payload}"
jwt_signature=$(printf '%s' "${jwt_signing_input}" \
  | openssl dgst -sha256 -hmac "${ROZE_JWT_SECRET}" -binary \
  | base64url)
auth_header="authorization: Bearer ${jwt_signing_input}.${jwt_signature}"

unauthenticated_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H 'x-tenant-id: tenant-a' \
  "${base_url}/api/v1/projects")
test "${unauthenticated_status}" = "401"

project_name="${database_name}-smoke-$(date +%s)-${RANDOM}"
created=$(curl --fail --silent \
  -X POST "${base_url}/api/v1/projects" \
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
  "${base_url}/api/v1/projects/${project_id}")
test "${wrong_tenant_status}" = "403"

curl --fail --silent \
  -X PATCH "${base_url}/api/v1/projects/${project_id}" \
  -H "${auth_header}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"expected_version\":1,\"name\":\"${project_name}-updated\",\"description\":null}" \
  >/dev/null

stale_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -X PATCH "${base_url}/api/v1/projects/${project_id}" \
  -H "${auth_header}" \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-a' \
  -d "{\"expected_version\":1,\"name\":\"${project_name}-stale\",\"description\":null}")
test "${stale_status}" = "412"

curl --fail --silent \
  -X DELETE \
  -H "${auth_header}" \
  -H 'x-tenant-id: tenant-a' \
  "${base_url}/api/v1/projects/${project_id}" \
  >/dev/null

deleted_status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
  -H "${auth_header}" \
  -H 'x-tenant-id: tenant-a' \
  "${base_url}/api/v1/projects/${project_id}")
test "${deleted_status}" = "404"

echo "${database_name} service API smoke passed"

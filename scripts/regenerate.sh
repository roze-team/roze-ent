#!/usr/bin/env bash
set -euo pipefail

rozectl_bin="${ROZECTL_BIN:-rozectl}"

"${rozectl_bin}" api validate roze-ent.api
"${rozectl_bin}" api format roze-ent.api --check
"${rozectl_bin}" api generate roze-ent.api --out services/roze-ent-api --update --roze-source git
"${rozectl_bin}" model generate model/schema.ent --out services/roze-ent-api --format ent --update --roze-source git
extension_host="${ROZE_ENT_CODEGEN_BIN:-$(dirname "${rozectl_bin}")/roze-ent-codegen}"
if [[ ! -x "${extension_host}" ]]; then
  echo "Roze Ent extension host not found or not executable: ${extension_host}" >&2
  exit 2
fi
"${extension_host}" model/schema.ent services/roze-ent-api
"${rozectl_bin}" openapi generate roze-ent.api --out docs/openapi.json
cargo fmt --all

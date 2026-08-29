#!/usr/bin/env bash
set -euo pipefail

rozectl_bin="${ROZECTL_BIN:-rozectl}"

"${rozectl_bin}" api validate roze-ent.api
"${rozectl_bin}" api format roze-ent.api --check
"${rozectl_bin}" api generate roze-ent.api --out services/roze-ent-api --update --roze-source git
"${rozectl_bin}" model generate model/schema.ent --out services/roze-ent-api --format ent --update --roze-source git
"${rozectl_bin}" openapi generate roze-ent.api --out docs/openapi.json
cargo fmt --all

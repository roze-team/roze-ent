#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <empty-source-directory>" >&2
  exit 2
fi

source_dir="$1"
roze_revision="e4bf750dfa630ca4224318d1e7c72a818598a2d2"
workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -e "${source_dir}" ]]; then
  echo "source directory already exists: ${source_dir}" >&2
  exit 2
fi

git clone https://github.com/roze-team/roze.git "${source_dir}"
git -C "${source_dir}" checkout "${roze_revision}"
cp "${workspace_dir}/integration/rozectl-model-adapter.rs" \
  "${source_dir}/apps/rozectl/src/generator/model.rs"
cargo add --manifest-path "${source_dir}/apps/rozectl/Cargo.toml" \
  roze-ent --path "${workspace_dir}/crates/roze-ent"
mkdir -p "${source_dir}/apps/rozectl/src/bin"
cp "${workspace_dir}/extensions/roze-ent-codegen.rs" \
  "${source_dir}/apps/rozectl/src/bin/roze-ent-codegen.rs"
cargo build --locked --manifest-path "${source_dir}/Cargo.toml" --target-dir "${source_dir}/target" \
  -p rozectl --bin rozectl --bin roze-ent-codegen

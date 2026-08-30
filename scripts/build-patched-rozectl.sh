#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <empty-source-directory>" >&2
  exit 2
fi

source_dir="$1"
roze_revision="1945a037558717ae9253fa61060fe900567e52de"
workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -e "${source_dir}" ]]; then
  echo "source directory already exists: ${source_dir}" >&2
  exit 2
fi

git clone https://github.com/roze-team/roze.git "${source_dir}"
git -C "${source_dir}" checkout "${roze_revision}"
git -C "${source_dir}" apply "${workspace_dir}/patches/roze/0001-fix-sea-orm-case-insensitive-predicates.patch"
git -C "${source_dir}" apply "${workspace_dir}/patches/roze/0002-fix-sea-orm-sqlite-upsert-returning.patch"
git -C "${source_dir}" apply "${workspace_dir}/patches/roze/0003-fix-sea-orm-custom-id-insert-returning.patch"
git -C "${source_dir}" apply "${workspace_dir}/patches/roze/0004-fix-sea-orm-scalar-clippy-output.patch"
git -C "${source_dir}" apply "${workspace_dir}/patches/roze/0005-fix-sea-orm-like-escape.patch"
cargo build --locked --manifest-path "${source_dir}/Cargo.toml" --target-dir "${source_dir}/target" -p rozectl

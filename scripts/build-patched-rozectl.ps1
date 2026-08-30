param(
    [Parameter(Mandatory = $true)]
    [string]$SourceDir
)

$ErrorActionPreference = "Stop"
$RozeRevision = "1945a037558717ae9253fa61060fe900567e52de"
$WorkspaceDir = Split-Path -Parent $PSScriptRoot
$SourceDir = [IO.Path]::GetFullPath($SourceDir)

if (Test-Path -LiteralPath $SourceDir) {
    throw "Source directory already exists: $SourceDir"
}

git clone https://github.com/roze-team/roze.git $SourceDir
git -C $SourceDir checkout $RozeRevision
git -C $SourceDir apply (Join-Path $WorkspaceDir "patches/roze/0001-fix-sea-orm-case-insensitive-predicates.patch")
git -C $SourceDir apply (Join-Path $WorkspaceDir "patches/roze/0002-fix-sea-orm-sqlite-upsert-returning.patch")
git -C $SourceDir apply (Join-Path $WorkspaceDir "patches/roze/0003-fix-sea-orm-custom-id-insert-returning.patch")
git -C $SourceDir apply (Join-Path $WorkspaceDir "patches/roze/0004-fix-sea-orm-scalar-clippy-output.patch")
git -C $SourceDir apply (Join-Path $WorkspaceDir "patches/roze/0005-fix-sea-orm-like-escape.patch")
git -C $SourceDir apply (Join-Path $WorkspaceDir "patches/roze/0006-add-sea-orm-pessimistic-locks.patch")
$BinDir = Join-Path $SourceDir "apps/rozectl/src/bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item -LiteralPath (Join-Path $WorkspaceDir "extensions/roze-ent-codegen.rs") `
    -Destination (Join-Path $BinDir "roze-ent-codegen.rs")
cargo build --locked --manifest-path (Join-Path $SourceDir "Cargo.toml") --target-dir (Join-Path $SourceDir "target") -p rozectl --bin rozectl --bin roze-ent-codegen

Write-Output (Join-Path $SourceDir "target/debug/rozectl.exe")

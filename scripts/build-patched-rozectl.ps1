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
cargo build --locked --manifest-path (Join-Path $SourceDir "Cargo.toml") --target-dir (Join-Path $SourceDir "target") -p rozectl

Write-Output (Join-Path $SourceDir "target/debug/rozectl.exe")

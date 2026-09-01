param(
    [Parameter(Mandatory = $true)]
    [string]$SourceDir
)

$ErrorActionPreference = "Stop"
$RozeRevision = "e4bf750dfa630ca4224318d1e7c72a818598a2d2"
$WorkspaceDir = Split-Path -Parent $PSScriptRoot
$SourceDir = [IO.Path]::GetFullPath($SourceDir)

if (Test-Path -LiteralPath $SourceDir) {
    throw "Source directory already exists: $SourceDir"
}

git clone https://github.com/roze-team/roze.git $SourceDir
git -C $SourceDir checkout $RozeRevision
$RozectlManifest = Join-Path $SourceDir "apps/rozectl/Cargo.toml"
Copy-Item -LiteralPath (Join-Path $WorkspaceDir "integration/rozectl-model-adapter.rs") `
    -Destination (Join-Path $SourceDir "apps/rozectl/src/generator/model.rs") -Force
cargo add --manifest-path $RozectlManifest roze-ent --path (Join-Path $WorkspaceDir "crates/roze-ent")
$BinDir = Join-Path $SourceDir "apps/rozectl/src/bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item -LiteralPath (Join-Path $WorkspaceDir "extensions/roze-ent-codegen.rs") `
    -Destination (Join-Path $BinDir "roze-ent-codegen.rs")
cargo build --locked --manifest-path (Join-Path $SourceDir "Cargo.toml") --target-dir (Join-Path $SourceDir "target") -p rozectl --bin rozectl --bin roze-ent-codegen

Write-Output (Join-Path $SourceDir "target/debug/rozectl.exe")

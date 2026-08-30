param(
    [string]$Rozectl = $(if ($env:ROZECTL_BIN) { $env:ROZECTL_BIN } else { "rozectl" })
)

$ErrorActionPreference = "Stop"

& $Rozectl api validate roze-ent.api
& $Rozectl api format roze-ent.api --check
& $Rozectl api generate roze-ent.api --out services/roze-ent-api --update --roze-source git
& $Rozectl model generate model/schema.ent --out services/roze-ent-api --format ent --update --roze-source git
$ExtensionHost = if ($env:ROZE_ENT_CODEGEN_BIN) {
    $env:ROZE_ENT_CODEGEN_BIN
} else {
    Join-Path (Split-Path -Parent $Rozectl) "roze-ent-codegen.exe"
}
if (-not (Test-Path -LiteralPath $ExtensionHost)) {
    throw "Roze Ent extension host not found: $ExtensionHost"
}
& $ExtensionHost model/schema.ent services/roze-ent-api
& $Rozectl openapi generate roze-ent.api --out docs/openapi.json
cargo fmt --all

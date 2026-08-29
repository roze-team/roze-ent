param(
    [string]$Rozectl = $(if ($env:ROZECTL_BIN) { $env:ROZECTL_BIN } else { "rozectl" })
)

$ErrorActionPreference = "Stop"

& $Rozectl api validate roze-ent.api
& $Rozectl api format roze-ent.api --check
& $Rozectl api generate roze-ent.api --out services/roze-ent-api --update --roze-source git
& $Rozectl model generate model/schema.ent --out services/roze-ent-api --format ent --update --roze-source git
& $Rozectl openapi generate roze-ent.api --out docs/openapi.json
cargo fmt --all

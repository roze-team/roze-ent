$ErrorActionPreference = "Stop"

rozectl api validate roze-ent.api
rozectl api format roze-ent.api --check
rozectl api generate roze-ent.api --out services/roze-ent-api --update --roze-source git
rozectl model generate model/schema.ent --out services/roze-ent-api --format ent --update --roze-source git
cargo fmt --all


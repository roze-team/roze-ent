# rozectl integration

Replace `apps/rozectl/src/generator/model.rs` with
`rozectl-model-adapter.rs`, then add `roze-ent` to `apps/rozectl/Cargo.toml`
using the reviewed commit of this repository:

```toml
roze-ent = { git = "https://github.com/roze-team/roze-ent.git", rev = "<reviewed-commit>" }
```

The adapter preserves the public `rozectl::generator::model` names and all CLI
arguments. Its `RozectlHost` injects the same `ROZE_GIT_REV` used by generated
`roze-*` dependencies and delegates service-manifest/ServiceContext sync back to
`rozectl`. The file intentionally contains no parser, inspector, or renderer.
Mongo wiring is selected from `ModelProjectRequirements`, including the
`roze-mongo` dependency and connection, health-registration, and model-context
capabilities. It does not inspect generated Rust source or backend markers.

For later REST/RPC updates, the adapter persists that decision in the normal
host-owned Cargo dependency merge. Cross-generator reconciliation therefore
reads `Cargo.toml`, not generated model source, and remains intact regardless of
whether model generation or REST/RPC generation runs first.

For a coordinated local checkout, a temporary path dependency may be used only
for validation. Release and CI integration must use the Git dependency above.

Verified integration commands:

```bash
cargo check -p rozectl
cargo test -p rozectl -- --skip postgres --skip mysql --skip mongo
```

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

For a coordinated local checkout, a temporary path dependency may be used only
for validation. Release and CI integration must use the Git dependency above.

Verified integration commands:

```bash
cargo check -p rozectl
cargo test -p rozectl -- --skip postgres --skip mysql --skip mongo
```

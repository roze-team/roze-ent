# roze-ent

`roze-ent` 是基于 [Roze](https://github.com/roze-team/roze) 的 Rust-native
实体与图数据服务。它把上游 [ent](https://github.com/ent/ent) 的核心理念——schema
as code、静态类型查询、关系遍历和代码生成——映射到 Roze 的 `.ent`、SeaORM 与
原生 HTTP 运行时。

当前仓库是第一阶段可运行迁移，不宣称已经逐文件复刻上游 2,318 个 Go 源文件。
已落地的切片包括：

- Rust 1.98 Cargo workspace；
- Roze 1.0 生成的 REST、DTO、校验、OpenAPI 与运行时边界；
- `.ent` 驱动的 User/Pet 与 User/Group/Membership 图模型；
- tenant-scoped、soft-delete、optimistic-lock Project 模型；
- 生成的 typed predicate/query/create/update/delete、边遍历、事务与缓存接线；
- SQLite 真实事务回滚及 Project hook/policy/mixin 兼容测试；
- `roze-migration` 驱动的 SQLite dry-run、apply、rollback、drift 与原子性证据；
- User/Pet CRUD API 与 PostgreSQL 初始化；
- 可重复的 API/model 再生成脚本与 CI 门禁。

完整迁移边界和未完成能力见 [迁移说明](docs/MIGRATION.md)。

## 快速开始

要求 Docker、Rust 1.98 和与本项目固定 revision 一致的 `rozectl`：

```powershell
cargo install --git https://github.com/roze-team/roze.git --rev 39bb1afc8aaf759bf130c5008a61f092e7acbc46 rozectl
docker compose up -d postgres
$env:DATABASE_URL = "postgres://roze:roze@127.0.0.1:5432/roze_ent"
$env:ROZE_CONFIG_PATH = "services/roze-ent-api/config.yaml"
cargo run -p roze-ent-api
```

服务默认监听 `127.0.0.1:3000`。主要接口：

- `POST /api/v1/users`
- `GET /api/v1/users`
- `GET /api/v1/users/:id`
- `DELETE /api/v1/users/:id`
- `POST /api/v1/pets`
- `GET /api/v1/pets/:id`
- `GET /api/v1/users/:id/pets`
- `DELETE /api/v1/pets/:id`
- `POST /api/v1/groups`
- `GET /api/v1/groups`
- `GET /api/v1/groups/:id`
- `POST /api/v1/groups/:group_id/members/:user_id`
- `PATCH /api/v1/groups/:group_id/members/:user_id`
- `DELETE /api/v1/groups/:group_id/members/:user_id`
- `GET /api/v1/groups/:id/users`
- `GET /api/v1/users/:id/groups`
- `POST /api/v1/projects`
- `GET /api/v1/projects`
- `GET /api/v1/projects/:id`
- `PATCH /api/v1/projects/:id`
- `DELETE /api/v1/projects/:id`

Roze 自带的 `/healthz`、`/readyz`、`/startupz`、`/metrics` 和
`/openapi.json` 也由服务暴露。

Membership 使用显式 Through edge。角色更新要求同时提交 `expected_role` 和新
`role`；并发修改时返回 `412 Failed Precondition`，避免静默覆盖。

Project 接口要求 `x-tenant-id` 请求头。查询默认排除已软删除记录，更新要求提交
`expected_version`，删除只写入 `deleted_at` 而不物理删除数据。

示例请求：

```bash
curl -X POST http://127.0.0.1:3000/api/v1/users \
  -H "content-type: application/json" \
  -d '{"email":"alice@example.com","name":"Alice"}'
```

## 事实源与再生成

- HTTP/DTO：[`roze-ent.api`](roze-ent.api)
- OpenAPI：[`docs/openapi.json`](docs/openapi.json)
- 数据模型：[`model/schema.ent`](model/schema.ent)
- 应用逻辑：`services/roze-ent-api/src/logic/**`
- 生成模型：`services/roze-ent-api/src/model/**`

Windows：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/regenerate.ps1
```

Linux/macOS：

```bash
bash scripts/regenerate.sh
```

顺序必须是先 API、后 model；第二步会把生成模型重新接入 Roze
`ServiceContext`。日常更新使用 `--update`，不要手改 handler、route、DTO 或生成
repository。

## 验证

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

工作区测试包含无需 Docker 的 SQLite 内存集成场景，验证生成模型事务的提交/回滚、
Project create mixin、tenant policy 和 mutation hook。mutation hook 先于外层事务提交
完成；不可逆副作用应进入事务型 outbox，而不是直接从 hook 发出。

SQLite 方言迁移保存在 `migrations/sqlite/`，workspace 测试会直接加载这些 SQL，
验证迁移计划、账本一致性、部分/全量回滚、名称漂移拒绝和失败批次原子性。

具备 Docker 的 Linux/CI 环境还可以运行真实 PostgreSQL smoke：

```bash
bash scripts/postgres-smoke.sh
```

该流程验证迁移、服务健康、tenant 隔离、乐观版本冲突和 soft-delete。脚本结束时
停止服务与 Compose 容器，但保留数据库 volume 以便诊断。

许可证：Apache-2.0。上游 Go 项目保留为 Git remote `upstream`，便于持续做行为对照。

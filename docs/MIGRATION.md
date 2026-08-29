# 从 ent 到 Roze/Rust

## 上游集成边界

本项目的最终消费者是 `roze-team/roze`。这里先用独立 workspace 固化 ent 行为、生成
稳定性和真实数据库证据，成熟后按职责合入 Roze，而不是让 Roze 长期依赖本仓库的示例
服务：

- `.ent` schema、解析、规范化和代码生成能力归入 `rozectl`；
- typed query/mutation、edge、Hook、Policy、Mixin 与事务语义归入 Roze 数据层 crate；
- migration plan、ledger、apply/rollback 与方言证据归入 `roze-migration`；
- `services/roze-ent-api` 保留为契约和端到端验收样例，不进入框架核心依赖图；
- 业务 schema、路由和 logic 不向 Roze 核心反向渗透。

孵化阶段固定 Roze Git revision，以便验证真实集成；正式迁入时使用 Roze workspace path
依赖并通过 `rozectl --update` 再生成，以生成 diff、兼容矩阵和数据库 smoke 作为合并门禁。

## Roze 全量兼容门禁

“本项目功能全部兼容 Roze”按以下可执行证据判定，而不是仅以接口名称或生成成功判定：

| 门禁 | 通过条件 | 当前状态 |
| --- | --- | --- |
| 依赖 | 所有 Roze crate 与 `rozectl` 固定到同一上游 revision | 已通过：`39bb1af`，与当前 Roze `main` 一致 |
| 生成 | `rozectl api/model/openapi --update` 可重复执行，应用自有文件保留且生成结果无漂移 | 持续门禁 |
| 编译与质量 | Rust 1.98 下 fmt/check/test/clippy 全部通过 | CI 门禁 |
| 数据行为 | SQLite/PostgreSQL/MySQL 的查询、变更、事务与迁移语义有真实数据库证据 | 持续扩展；三方言 migration 已覆盖 |
| 运行时 | HTTP、配置、健康检查、中间件与数据库只使用 Roze 公共接口 | 已纳入服务 smoke |
| 上游归属 | 框架能力进入 `rozectl`、Roze 数据层 crate 或 `roze-migration`，生成文件不接受手工补丁 | 强制边界 |

任何一项失败都视为 Roze 兼容阻断。目前已知阻断项是生成的 `IContains/EqualFold` 使用 PostgreSQL
`ILIKE`，在 SQLite SeaQuery builder 上会 panic；该项必须在 Roze 生成器中完成方言化并重新生成后，才能标记为全量通过。

## 基线

迁移分析使用上游 `ent/ent` 提交 `69d5d4deb`。该基线约有 2,318 个 Go 文件，主要
能力分布在 `entc`、`dialect`、`schema`、`entql`、`privacy` 与生成模板。逐文件
翻译会复制 Roze 已有的 ORM 与代码生成基础设施，因此本项目采用行为映射：由 Roze
负责框架级生成和运行时，仓库只维护领域 schema、API 与应用逻辑。

## 能力映射

| ent 概念 | Roze/Rust 落点 | 当前状态 |
| --- | --- | --- |
| Go schema as code | `model/schema.ent` | 已落地 |
| `ent generate` / `entc` | `rozectl model generate --format ent` | 已落地 |
| typed Client/Query/Create/Update/Delete | 生成的 SeaORM repository 与 builder | 已落地 |
| predicates/order/pagination/projection/aggregate | 生成的 ent-style query API | 已落地；SQLite 真实语义矩阵覆盖复合/IN/range/contains predicate、typed order、offset/limit/page、nullable projection、grouped count 与数值聚合；`IContains/EqualFold` 的 SQLite 方言化仍需修复 Roze 生成器 |
| edge traversal | `.ent` edge 与生成的 relation query | 已落地；SQLite 真实语义矩阵覆盖 User/Group/Membership Through M2M、`HasXWith`、双向 traversal 与 eager loading，User/Pet 保留 ordinary edge 示例 |
| optimistic update | `update_where().execute()` + `FailedPrecondition` | 已落地：Membership role |
| tenant scope / soft delete | tenant predicate、live scope、`soft_delete_by_id` | 已落地：Project |
| PostgreSQL smoke | migration + HTTP tenant/version/delete 流程 | 已接入 CI；本地需要 Docker |
| transaction/hooks/privacy/mixins | `ModelClient::transaction`、operation chain 与 `*_ext.rs` | SQLite 真实事务提交/回滚及 Project hook/policy/mixin 已覆盖 |
| SQL migrations | `roze-migration` + `migrations/` | SQLite dry-run/apply/partial rollback/full rollback/drift/atomicity 已验证 |
| SQL dialects | SeaORM/Roze DB（PostgreSQL/MySQL/SQLite） | SQLite、PostgreSQL、MySQL 的真实 migration lifecycle 已覆盖 |
| Gremlin/GraphSON | 独立图后端适配 | 未迁移 |
| Atlas 深度集成 | Roze migration/gate 对接 | 未迁移 |
| entc 自定义 Go template | Roze generator extension API | 未迁移具体扩展 |
| entql | typed predicate 与应用组合层 | 基础查询已覆盖，高级表达式待补 |

## 所有权边界

生成器拥有 `src/handler`、`src/route`、`src/types`、`src/openapi`、
`src/svc` 和 `src/model` 中带 generated marker 的文件。应用只在以下位置写行为：

- `services/roze-ent-api/src/logic/**`；
- `services/roze-ent-api/src/logic/prelude.rs`；
- `services/roze-ent-api/src/application.rs`；
- `services/roze-ent-api/src/model/*_ext.rs`。

再生成必须使用 `--update`。模型生成必须在 API 生成之后运行，以恢复
`ServiceContext::model()` 接线。

## 后续阶段

1. 将真实 PostgreSQL/MySQL migration matrix 作为持续 CI 门禁，并把健康端点前缀差异反馈到 Roze 生成器。
2. 将当前 SQLite ent-style 查询兼容矩阵扩展到 PostgreSQL/MySQL，并把发现的生成器语义差异反馈到 Roze 上游门禁。
3. 扩展 Project 之外的领域 policy，并为事务型副作用接入 outbox 证据。
4. 评估 Gremlin、Atlas 和生成器扩展的真实使用需求，再决定是否实现 Rust 适配层。

只有对应测试和真实依赖证据通过后，才会把某项标记为完整兼容。

Project 的 SQLite 证据使用真实 SQL 连接验证提交和强制回滚，并在同一生成式 create
链中覆盖字段 mixin、tenant policy 与 client-level mutation hook。hook 在 mutation
成功后、外层事务提交前运行，因此事务内 hook 不得直接产生不可逆副作用；这类行为
应通过与领域写入同事务的 outbox 或明确的提交后机制完成。

SQLite 方言迁移位于 `migrations/sqlite/`，版本和名称与 PostgreSQL 迁移保持一致。
集成测试直接通过固定 revision 的 `roze-migration` 加载这些项目文件，验证确定性 dry-run、
apply、回滚到版本边界、全量回滚、名称漂移拒绝和失败批次原子回滚。PostgreSQL 与 MySQL
分别由独立 CI smoke 在真实容器中执行 apply、账本幂等检查和全量 rollback；MySQL 迁移对
保留关键字标识符使用方言引用。多语句项目迁移会先确定性拆分为单语句 ledger step，以
满足 SQLx prepared-statement 边界。

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
| predicates/order/pagination/projection/aggregate | 生成的 ent-style query API | 已落地 |
| edge traversal | `.ent` edge 与生成的 relation query | 已落地：User/Pet、User/Group/Membership Through M2M |
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
2. 对照 ent 的 predicate、projection、aggregate 与 edge 测试建立兼容矩阵。
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

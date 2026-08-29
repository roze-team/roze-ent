# 从 ent 到 Roze/Rust

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
| SQL migrations | `roze-migration` + `migrations/` | 首个 PostgreSQL schema 已落地 |
| SQL dialects | SeaORM/Roze DB（PostgreSQL/MySQL/SQLite） | 生成层已支持，当前运行配置为 PostgreSQL |
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

1. 增加 PostgreSQL/SQLite/MySQL 真实依赖矩阵和 `roze-migration` dry-run/rollback 证据。
2. 对照 ent 的 predicate、projection、aggregate 与 edge 测试建立兼容矩阵。
3. 扩展 Project 之外的领域 policy，并为事务型副作用接入 outbox 证据。
4. 评估 Gremlin、Atlas 和生成器扩展的真实使用需求，再决定是否实现 Rust 适配层。

只有对应测试和真实依赖证据通过后，才会把某项标记为完整兼容。

Project 的 SQLite 证据使用真实 SQL 连接验证提交和强制回滚，并在同一生成式 create
链中覆盖字段 mixin、tenant policy 与 client-level mutation hook。hook 在 mutation
成功后、外层事务提交前运行，因此事务内 hook 不得直接产生不可逆副作用；这类行为
应通过与领域写入同事务的 outbox 或明确的提交后机制完成。

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
| 依赖 | 所有 Roze crate 与 `rozectl` 固定到同一上游 revision | 已通过：`1945a03`，与当前 Roze `main` 一致 |
| 生成 | `rozectl api/model/openapi --update` 可重复执行，应用自有文件保留且生成结果无漂移 | 持续门禁 |
| 编译与质量 | Rust 1.98 下 fmt/check/test/clippy 全部通过 | CI 门禁 |
| 数据行为 | SQLite/PostgreSQL/MySQL 的查询、变更、事务与迁移语义有真实数据库证据 | 持续扩展；三方言 migration 已覆盖 |
| 运行时 | HTTP、配置、健康检查、中间件与数据库只使用 Roze 公共接口 | 已纳入服务 smoke |
| 上游归属 | 框架能力进入 `rozectl`、Roze 数据层 crate 或 `roze-migration`，生成文件不接受手工补丁 | 强制边界 |

任何一项失败都视为 Roze 兼容阻断。固定 Roze revision 原生生成的 `IContains/EqualFold` 使用 PostgreSQL
`ILIKE`，在 SQLite SeaQuery builder 上会 panic。修复位于
[`patches/roze/0001-fix-sea-orm-case-insensitive-predicates.patch`](../patches/roze/0001-fix-sea-orm-case-insensitive-predicates.patch)：
它统一生成 `LOWER(column) LIKE lowercased_pattern`，已通过 Roze 生成器测试、309 项 `rozectl` 测试和生成 SeaORM
SQLite crate 的真实运行证据。本项目的 patched-rozectl 生成链已应用该补丁，并由 SQLite
`IContains/EqualFold` 测试及 `generated-code` CI 固化；Roze 上游仍需合入补丁才能移除临时构建链。

mutation 矩阵还发现 SeaORM SQLite 的冲突 upsert 返回值错误：生成代码调用
`exec_with_returning` 时会按旧的 `last_insert_id` 读取其他记录。可上游应用的修复位于
[`patches/roze/0002-fix-sea-orm-sqlite-upsert-returning.patch`](../patches/roze/0002-fix-sea-orm-sqlite-upsert-returning.patch)：
它在消费模型前保存单一或复合主键，执行 upsert 后从主写连接按该主键重新读取目标行。
该补丁已通过 125 项 Roze model generator 测试和生成 SeaORM SQLite crate 的真实冲突
upsert 运行证据。本项目的 patched-rozectl 生成链已应用该补丁，并覆盖同一主键冲突更新、
不可更新字段保持与返回行重载；Roze 上游仍需合入补丁才能移除临时构建链。

自定义字符串 ID 和复合 ID 的 SQLite create 进一步发现 SeaORM `ActiveModel::insert()` 会按
`last_insert_id` 回读，从而在记录已写入后返回 `RecordNotFound`。修复位于
[`patches/roze/0003-fix-sea-orm-custom-id-insert-returning.patch`](../patches/roze/0003-fix-sea-orm-custom-id-insert-returning.patch)：
非自增主键在插入前保存单一或复合键，执行 `exec_without_returning` 后从同一主写连接按键重载；
自增主键保留原路径。生成器字符串/复合 ID 单测和本仓库 SQLite 真实测试均覆盖该行为。

新增 Decimal 与 `f64` 标量矩阵还暴露了生成代码的严格 Clippy 问题：Decimal 是 Copy 类型却被
不必要地 `clone()`，而 `f64` 聚合又生成同类型的 `as f64` 转换。修复位于
[`patches/roze/0004-fix-sea-orm-scalar-clippy-output.patch`](../patches/roze/0004-fix-sea-orm-scalar-clippy-output.patch)：
它将 Decimal 纳入 Copy filter 类型，并让平均值代码仅对非 `f64` 输入做转换；生成器回归测试、
本仓库全量测试与 `clippy -D warnings` 共同固化该行为。

字符串谓词矩阵进一步发现生成代码虽然会在 LIKE pattern 中转义 `%`、`_` 与反斜杠，
却没有向 SQL 方言声明转义字符，导致 SQLite 将转义后的 pattern 当作普通反斜杠文本。
修复位于 [`patches/roze/0005-fix-sea-orm-like-escape.patch`](../patches/roze/0005-fix-sea-orm-like-escape.patch)：
SeaORM 生成代码统一使用 `LikeExpr::escape('\\')`，使 contains、starts/ends-with、
大小写不敏感及其否定变体在 SQLite、PostgreSQL、MySQL 上共享显式转义语义。

悲观锁兼容由 [`patches/roze/0006-add-sea-orm-pessimistic-locks.patch`](../patches/roze/0006-add-sea-orm-pessimistic-locks.patch)
提供。生成的 SeaORM query 暴露 `.for_update()?` 与 `.for_share()?`，只允许事务作用域，
强制使用 primary transaction，并在 SQLite 上明确拒绝。PostgreSQL/MySQL smoke 同时验证
主键冲突 upsert、共享锁执行，以及第二个 `FOR UPDATE` 在首个事务释放前保持阻塞。

标量矩阵还确认 `.ent u64` 虽可生成和编译，但 SeaORM/sqlx-sqlite 在运行时拒绝绑定 `u64`。
该能力保持未完成，不能以 `i64` 替换后宣称等价；需要 Roze 提供带范围检查的 SQLite 存储转换，
并同时保持 Rust 公共模型的 unsigned 语义。

## 基线

迁移分析使用上游 `ent/ent` 提交 `69d5d4deb`。该基线约有 2,318 个 Go 文件，主要
能力分布在 `entc`、`dialect`、`schema`、`entql`、`privacy` 与生成模板。逐文件
翻译会复制 Roze 已有的 ORM 与代码生成基础设施，因此本项目采用行为映射：由 Roze
负责框架级生成和运行时，仓库只维护领域 schema、API 与应用逻辑。

## 能力映射

逐项、可验收的完整清单见 [`ENT_COMPATIBILITY.md`](ENT_COMPATIBILITY.md)。下表仅保留迁移总览。

| ent 概念 | Roze/Rust 落点 | 当前状态 |
| --- | --- | --- |
| Go schema as code | `model/schema.ent` | 已落地 |
| `ent generate` / `entc` | `rozectl model generate --format ent` | 已落地 |
| typed Client/Query/Create/Update/Delete | 生成的 SeaORM repository 与 builder | 已落地 |
| 标量类型与自定义/复合 ID | ScalarFixture、LocaleSetting 与 patched-rozectl | SQLite 已覆盖 bytes/JSON/Decimal/time/float/signed integer、自定义字符串 ID 和复合键；UUID 强类型、u64、三方言 CRUD 仍待完成 |
| create/update/delete one/many 与 batch/upsert | 生成的 mutation builder 与 repository batch API | SQLite 真实语义矩阵已覆盖必填/字段校验、唯一约束、one/many、原子条件更新、批量插入/删除，以及 upsert 插入和冲突更新返回语义；补丁 0002 待合入 Roze 上游 |
| predicates/order/pagination/projection/aggregate | 生成的 ent-style query API | 已落地；SQLite 真实语义矩阵覆盖复合/IN/range/contains/IContains/EqualFold predicate、typed order、offset/limit/page、nullable projection、grouped count 与数值聚合；补丁 0001 待合入 Roze 上游 |
| edge traversal | `.ent` edge 与生成的 relation query | 已落地；SQLite 真实语义矩阵覆盖 User/Group/Membership Through M2M、`HasXWith`、双向 traversal 与 eager loading，User/Pet 保留 ordinary edge 示例 |
| self/bidirectional/named edges | User manager/reports、friends/friended_by 与 Friendship user/friend | 已落地；SQLite 覆盖嵌套 eager load、同目标命名槽位、关系过滤及 add/remove/clear，三方言 migration 由 CI 固化 |
| optimistic update | `update_where().execute()` + `FailedPrecondition` | 已落地：Membership role |
| tenant scope / soft delete | tenant predicate、live scope、`soft_delete_by_id` | 已落地：Project |
| PostgreSQL smoke | migration + HTTP tenant/version/delete 流程 | 已接入 CI；本地需要 Docker |
| MySQL smoke | migration + HTTP tenant/version/delete 流程 | 已接入 CI；本地需要 Docker |
| transaction/hooks/privacy/mixins | `ModelClient::transaction`、operation chain 与 `*_ext.rs` | SQLite 真实事务提交/回滚及 Project hook/policy/mixin 已覆盖 |
| SQL migrations | `roze-migration` + `migrations/` | 当前 8 个版本均覆盖三方言 apply/full rollback；SQLite 另有 partial rollback/drift/atomicity 证据 |
| SQL dialects | SeaORM/Roze DB（PostgreSQL/MySQL/SQLite） | PostgreSQL、MySQL 均有真实 migration lifecycle 与同一套 HTTP 服务行为验证 |
| Gremlin/GraphSON | 独立图后端适配 | 未迁移 |
| Atlas 深度集成 | Roze migration/gate 对接 | 未迁移 |
| entc 自定义 Go template | Roze generator extension API | 已落地独立只读 View 扩展宿主；通用动态插件分发待扩展 |
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

1. 将当前 SQLite ent-style 查询兼容矩阵扩展到 PostgreSQL/MySQL，并把发现的生成器语义差异反馈到 Roze 上游门禁。
2. 增加 schema diff、危险变更分类、expand/backfill/contract 数据迁移及失败恢复证据。
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
分别由独立 CI smoke 在真实容器中执行全部 8 个版本的 apply、账本幂等检查和全量 rollback，
随后运行共享的鉴权、tenant 隔离、乐观锁和 soft-delete HTTP 行为套件；MySQL 迁移对
保留关键字标识符使用方言引用。多语句项目迁移会先确定性拆分为单语句 ledger step，以满足
SQLx prepared-statement 边界。PostgreSQL ledger lifecycle 使用同一容器内的专用临时数据库，
避免 Compose 初始化的业务 schema 与测试账本互相污染；业务服务 smoke 仍连接 `roze_ent`。

`AuditEvent` 使用 `.ent` 的 `schema "roze_ent"` 生成 SeaORM `schema_name`，并通过
`roze_ent.audit_events.user_id → public.users.id` 提供 PostgreSQL 跨 schema 外键。数据库
smoke 使用生成的 repository、predicate 和 edge traversal 验证 PostgreSQL schema 与 MySQL
同名数据库命名空间；SQLite 仅验证不带命名空间的兼容迁移，不宣称支持 attached database。

全局唯一 ID 的事实源是 `model/globalid.toml`。与 ent 的 `globalid` feature 一致，每个
`i64 auto_increment` 实体获得一个稳定且不可复用的 `2^32` 区间；0007 迁移分别设置
PostgreSQL sequence、MySQL `AUTO_INCREMENT` 与 SQLite `sqlite_sequence`。新增类型只能追加
到配置末尾，不能重排或复用已发布区间。迁移不会改写已有主键；若历史环境已经存在跨表重复
ID，必须先进行显式 backfill/外键重映射，再启用全局 ID。回滚只取消区间起点，保留已经分配
的高位 ID，避免破坏引用完整性。

外部生成扩展源码位于 `extensions/roze-ent-codegen.rs`。构建脚本把它作为附加 binary 编译，
但实现只依赖 `rozectl` 公开的 `MODEL_GENERATOR_EXTENSION_API_VERSION`、模型图和扩展文件类型。
宿主在隔离临时项目中运行内建生成器，只将固定允许路径
`src/model/user_activity_view.rs` 同步到目标服务；核心模型文件不会被扩展覆盖。再生成脚本要求该
宿主存在，因此 CI 的 generated-code drift gate 同时覆盖外部扩展输出。

0008 创建 `user_activity_view`，按用户聚合宠物数和群组数。生成模块只暴露 `all` 与
`find_by_user_id`，没有 create/update/delete 类型或方法。SQLite migration 测试验证 View 查询
和回滚；PostgreSQL/MySQL smoke 使用生成 repository 验证真实数据库结果。

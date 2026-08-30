# ent → Roze/Rust 全功能兼容矩阵

本矩阵以 `ent/ent@69d5d4deb19599f129166634e09d33addcf3f2cc` 为上游行为基线，
以 `roze-team/roze@1945a037558717ae9253fa61060fe900567e52de` 为当前集成基线。
“已兼容”必须同时具备生成器/公共 API、真实运行行为和可重复测试证据；只有相似概念或
文档声明不计为完成。

状态定义：

- **已兼容**：本仓库已有可执行证据，且不依赖手改生成文件；
- **部分兼容**：核心路径可用，但方言、边界语义或真实数据库证据不完整；
- **待实现**：ent 基线提供，而当前 Roze/Rust 落点尚无等价实现和验收证据；
- **框架外适配**：不应进入 ORM 核心，但若要求 ent 全量能力，仍必须提供独立适配 crate。

## Schema 与类型系统

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| schema-as-code、生成、格式化、校验 | `.ent` + `rozectl model` | 已兼容 | `scripts/regenerate.ps1` 可重复执行 |
| string/bool/signed integer/enum/optional/default/immutable | 生成 entity、field 常量和 builder | 已兼容 | User/Pet/Group/Membership/Project 编译与 SQLite 行为测试 |
| bytes、time、JSON、UUID、float、unsigned integer、custom/Other | `.ent` 类型解析与 SeaORM 映射 | 部分兼容 | SQLite 已覆盖 bytes/JSON/Decimal/timestamp/timestamptz/i16/i32/i64/f32/f64 与 nullable 往返；UUID 当前映射字符串，`u64` 被 sqlx-sqlite 拒绝，且 PostgreSQL/MySQL CRUD 矩阵待补 |
| 字段 validator、unique、sensitive、comment | 生成 mutation 校验与元数据 | 部分兼容 | 已覆盖必填/唯一/部分 validator；补齐 sensitive redaction、comment/DDL 和所有类型 validator 证据 |
| 单字段、自定义及复合 ID | 生成主键类型和查询 API | 部分兼容 | patched-rozectl + SQLite 已覆盖自定义字符串 ID 插入/查改与复合键创建/查改删；补齐 edge、upsert 和 PostgreSQL/MySQL 证据 |
| indexes、复合唯一、部分索引、前缀/类型/包含列 | schema index + migration | 部分兼容 | 基础索引已生成；补齐方言专属索引选项及 schema diff 证据 |
| annotations、mixins、views | generator extension / model extension | 部分兼容 | mixin 已有运行证据；补齐 annotation 消费和只读 view 的生成/查询测试 |

## 关系与图遍历

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| O2O、O2M、M2M、inverse、required/unique edge | 生成 relation API | 已兼容 | User/Pet、User/Group/Membership SQLite traversal 矩阵 |
| edge schema / Through 与 edge 字段 | Membership 显式关联实体 | 已兼容 | 双向 traversal、`HasXWith`、eager-load 证据 |
| self-reference、双向边、named edges | 生成 relation API | 已兼容 | User manager/reports、friends/friended_by 与 Friendship user/friend SQLite 矩阵覆盖新增、删除、清空、双向遍历及命名 eager load |
| eager loading 与嵌套 eager loading | `with_*` 生成 API | 部分兼容 | 单层及 manager→reports 嵌套已有证据；补齐空关系、分页组合和三方言查询证据 |
| Gremlin/GraphSON 图存储 | 独立图后端适配 crate | 框架外适配 | 实现连接、CRUD、predicate、traversal、事务能力，并通过兼容套件 |

## Query、聚合与 entql

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| typed predicates、AND/OR/NOT、IN/range/null | 生成 query builder | 已兼容 | SQLite 真实查询矩阵 |
| contains/prefix/suffix/equal-fold/contains-fold | 生成字符串 predicate | 已兼容 | patched-rozectl 显式生成 `LIKE ... ESCAPE`；SQLite 及 PostgreSQL/MySQL CI 覆盖正向/否定变体与 `%`、`_`、反斜杠字面量 |
| HasEdge/HasEdgeWith 与反向 traversal | relation predicate | 已兼容 | Through M2M SQLite 行为测试 |
| order、offset/limit、page、projection、only/exist/count | typed query API | 已兼容 | SQLite 查询矩阵 |
| scalar aggregate、group-by、group aggregate、自定义 scan | aggregate API | 部分兼容 | count/sum/min/max/group 已覆盖；补齐多列 group、自定义 selector/scan 和三方言结果类型 |
| entql 动态 AST、字段/边表达式、序列化 | typed predicate + 动态表达式层 | 待实现 | 提供安全 AST、参数绑定、JSON round-trip、edge expression 与拒绝非法字段测试 |
| query interceptor、traversal interceptor | operation chain | 部分兼容 | 基础 interceptor 已生成；补齐全终结方法、嵌套 traversal、错误短路与顺序保证 |

## Mutation 与事务

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| create/update/delete one/many | mutation builder/repository | 已兼容 | SQLite 必填、唯一、one/many、批量删除矩阵 |
| bulk create / map create / update where | batch API | 已兼容 | SQLite batch 与条件更新证据 |
| arithmetic mutation、clear/null、edge add/remove/clear | mutation builder | 部分兼容 | 原子数值更新、nullable manager clear 与 self-Through friends add/remove/clear 已覆盖；补齐其余标量和 relation 组合 |
| upsert / on-conflict / conflict columns | generated upsert API | 部分兼容 | 主键冲突 upsert 已有 SQLite/PostgreSQL/MySQL 真实证据；补齐自定义冲突列和选择性更新 |
| mutation hooks、policy/privacy、mixins | `*_ext.rs` + operation chain | 部分兼容 | Project 已覆盖；补齐 query/mutation 全操作、组合顺序、deny/allow/skip 语义 |
| transaction、commit/rollback、事务 client | `ModelClient::transaction` | 已兼容 | SQLite 提交/强制回滚真实连接证据 |
| optimistic locking | `update_where().execute()` | 已兼容 | Membership version/role 冲突映射为 `FailedPrecondition` |
| pessimistic locking (`FOR UPDATE`/`FOR SHARE`) | SQL lock API | 已兼容 | 事务限定 typed API；PostgreSQL/MySQL 共享锁执行与排他锁并发阻塞证据，SQLite 明确拒绝 |
| SQL modifier / raw selector modifier / exec-query | 安全扩展接口 | 待实现 | 参数化 API、方言渲染、事务连接复用与注入防护测试 |

## Migration 与存储后端

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| create/drop/change schema、ledger、apply/rollback | `roze-migration` | 已兼容 | 当前 5 个迁移版本具备 SQLite/PostgreSQL/MySQL 全量 lifecycle CI，且跨方言版本、名称和 up/down 配对自动校验 |
| schema diff、offline plan、versioned migration | migration plan | 部分兼容 | 当前项目迁移可重复；补齐从模型自动 diff、危险变更分类和版本升级 fixture |
| data migration | 版本化 Rust/SQL migration | 部分兼容 | 增加 expand/backfill/contract、失败恢复和幂等证据 |
| external objects (trigger/view/function) | migration project objects | 待实现 | diff 忽略/管理策略、三方言 apply/rollback 证据 |
| multi-schema / schema config / global unique ID | generator + migration config | 待实现 | PostgreSQL schema 隔离、跨 schema edge、global ID 稳定性测试 |
| PostgreSQL、MySQL/MariaDB、SQLite | SeaORM/Roze DB | 部分兼容 | PostgreSQL/MySQL migration 与共享 HTTP tenant/version/delete 流程已覆盖；全部 query/mutation/transaction 矩阵仍需三方言一致 |
| CockroachDB、TiDB | SQL 方言兼容层 | 待实现 | 官方兼容版本的 migration、事务重试、CRUD 和并发 CI |
| Gremlin | 独立适配 crate | 框架外适配 | 见关系与图遍历部分 |
| Atlas 集成 | migration adapter / CI gate | 框架外适配 | schema inspect/diff/apply、lint 和版本目录互操作测试 |

## 工具链与生态扩展

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| codegen snapshot / deterministic generation | `rozectl --update` | 已兼容 | `generated-code` CI 使用固定 revision 再生成并要求 `git diff --exit-code` |
| external templates / generator extensions | rozectl extension API | 待实现 | 稳定扩展协议、版本约束、示例插件、冲突诊断和 golden tests |
| GraphQL (entgql relay、filter、mutation) | 独立 GraphQL adapter crate | 框架外适配 | schema/relay node/cursor pagination/filter/order/mutation/transaction/eager load 兼容套件 |
| OpenAPI/REST | Roze `.api` + `rozectl openapi` | 已兼容 | `docs/openapi.json` 与服务 smoke |
| existing `sql.DB` / driver integration | Roze DB connection injection | 部分兼容 | 补齐外部 pool、事务连接和生命周期/关闭所有权测试 |
| testing helpers、mock/debug client | Rust test-support crate | 待实现 | SQLite memory helper、fixture、golden SQL、mock interceptor、失败注入 |

## ent feature flags 对照

| ent feature | 状态 | 当前结论 |
| --- | --- | --- |
| `privacy` | 部分兼容 | Project policy 有证据，完整 allow/deny/skip 与全操作矩阵待补 |
| `intercept` | 部分兼容 | operation chain 已生成，完整终结方法/嵌套遍历待补 |
| `entql` | 待实现 | 缺动态 AST、序列化和 edge expression |
| `namedges` | 已兼容 | Friendship 的 user/friend 同目标边生成独立命名 eager-load 结果槽位并有 SQLite 证据 |
| `bidiedges` | 已兼容 | manager/reports 与 friends/friended_by self/bidirectional fixture 已覆盖双向查询和关系变更 |
| `schema/snapshot` | 部分兼容 | 生成可重复，缺显式 snapshot 升级协议 |
| `sql/schemaconfig` | 待实现 | 缺多 schema 运行证据 |
| `sql/lock` | 已兼容 | `.for_update()?` / `.for_share()?` 强制事务主连接，三方言行为有 CI 证据 |
| `sql/modifier` | 待实现 | 缺稳定、安全的 selector/mutation modifier API |
| `sql/execquery` | 待实现 | 缺 mutation query-returning 等价 API |
| `sql/upsert` | 部分兼容 | 项目生成链应用补丁 0002，主键冲突三方言矩阵已完成；自定义冲突列与选择性更新待补 |
| `sql/versioned-migration` | 部分兼容 | `roze-migration` 已有 ledger，模型 diff/version directory 互操作待补 |
| `sql/globalid` | 待实现 | 缺全局 ID range/稳定性/升级测试 |

## 完成判定

全量同步只有在本文件不再包含“部分兼容”或“待实现”，所有“框架外适配”均已有独立
crate 与兼容套件，并且 SQLite、PostgreSQL、MySQL 及声明支持的其他后端在 CI 中运行真实
行为测试后才能宣布完成。当前阶段不能判定为“全部满足需求”。

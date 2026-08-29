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
| bytes、time、JSON、UUID、float、unsigned integer、custom/Other | `.ent` 类型解析与 SeaORM 映射 | 部分兼容 | 增加全标量 fixture，并在 SQLite/PostgreSQL/MySQL 做 CRUD、nullable、default、round-trip 矩阵 |
| 字段 validator、unique、sensitive、comment | 生成 mutation 校验与元数据 | 部分兼容 | 已覆盖必填/唯一/部分 validator；补齐 sensitive redaction、comment/DDL 和所有类型 validator 证据 |
| 单字段、自定义及复合 ID | 生成主键类型和查询 API | 部分兼容 | 当前样例以单 ID 为主；补齐 custom/composite ID 三方言 CRUD、edge、upsert 证据 |
| indexes、复合唯一、部分索引、前缀/类型/包含列 | schema index + migration | 部分兼容 | 基础索引已生成；补齐方言专属索引选项及 schema diff 证据 |
| annotations、mixins、views | generator extension / model extension | 部分兼容 | mixin 已有运行证据；补齐 annotation 消费和只读 view 的生成/查询测试 |

## 关系与图遍历

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| O2O、O2M、M2M、inverse、required/unique edge | 生成 relation API | 已兼容 | User/Pet、User/Group/Membership SQLite traversal 矩阵 |
| edge schema / Through 与 edge 字段 | Membership 显式关联实体 | 已兼容 | 双向 traversal、`HasXWith`、eager-load 证据 |
| self-reference、双向边、named edges | 生成 relation API | 待实现 | 增加树/好友 fixture，覆盖新增、删除、遍历、命名 eager load |
| eager loading 与嵌套 eager loading | `with_*` 生成 API | 部分兼容 | 单层已有证据；补齐嵌套、空关系、分页组合和三方言证据 |
| Gremlin/GraphSON 图存储 | 独立图后端适配 crate | 框架外适配 | 实现连接、CRUD、predicate、traversal、事务能力，并通过兼容套件 |

## Query、聚合与 entql

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| typed predicates、AND/OR/NOT、IN/range/null | 生成 query builder | 已兼容 | SQLite 真实查询矩阵 |
| contains/prefix/suffix/equal-fold/contains-fold | 生成字符串 predicate | 部分兼容 | 普通 contains 已覆盖；大小写无关查询须合入补丁 0001，并在三方言运行 |
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
| arithmetic mutation、clear/null、edge add/remove/clear | mutation builder | 部分兼容 | 原子数值更新已覆盖；补齐所有标量、nullable 和 relation mutation 组合 |
| upsert / on-conflict / conflict columns | generated upsert API | 部分兼容 | 插入路径已覆盖；SQLite 冲突返回值须合入补丁 0002，再跑三方言矩阵 |
| mutation hooks、policy/privacy、mixins | `*_ext.rs` + operation chain | 部分兼容 | Project 已覆盖；补齐 query/mutation 全操作、组合顺序、deny/allow/skip 语义 |
| transaction、commit/rollback、事务 client | `ModelClient::transaction` | 已兼容 | SQLite 提交/强制回滚真实连接证据 |
| optimistic locking | `update_where().execute()` | 已兼容 | Membership version/role 冲突映射为 `FailedPrecondition` |
| pessimistic locking (`FOR UPDATE`/`FOR SHARE`) | SQL lock API | 待实现 | PostgreSQL/MySQL 并发测试，SQLite 明确降级语义 |
| SQL modifier / raw selector modifier / exec-query | 安全扩展接口 | 待实现 | 参数化 API、方言渲染、事务连接复用与注入防护测试 |

## Migration 与存储后端

| ent 能力 | Roze/Rust 落点 | 状态 | 完成条件/证据 |
| --- | --- | --- | --- |
| create/drop/change schema、ledger、apply/rollback | `roze-migration` | 已兼容 | SQLite/PostgreSQL/MySQL lifecycle CI |
| schema diff、offline plan、versioned migration | migration plan | 部分兼容 | 当前项目迁移可重复；补齐从模型自动 diff、危险变更分类和版本升级 fixture |
| data migration | 版本化 Rust/SQL migration | 部分兼容 | 增加 expand/backfill/contract、失败恢复和幂等证据 |
| external objects (trigger/view/function) | migration project objects | 待实现 | diff 忽略/管理策略、三方言 apply/rollback 证据 |
| multi-schema / schema config / global unique ID | generator + migration config | 待实现 | PostgreSQL schema 隔离、跨 schema edge、global ID 稳定性测试 |
| PostgreSQL、MySQL/MariaDB、SQLite | SeaORM/Roze DB | 部分兼容 | migration 已覆盖；全部 query/mutation/transaction 矩阵仍需三方言一致 |
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
| `namedges` | 待实现 | 缺命名 eager-load 结果槽位证据 |
| `bidiedges` | 待实现 | 缺 self/bidirectional edge fixture |
| `schema/snapshot` | 部分兼容 | 生成可重复，缺显式 snapshot 升级协议 |
| `sql/schemaconfig` | 待实现 | 缺多 schema 运行证据 |
| `sql/lock` | 待实现 | 缺 typed lock API 与并发证据 |
| `sql/modifier` | 待实现 | 缺稳定、安全的 selector/mutation modifier API |
| `sql/execquery` | 待实现 | 缺 mutation query-returning 等价 API |
| `sql/upsert` | 部分兼容 | 补丁 0002 尚未进入固定 Roze revision |
| `sql/versioned-migration` | 部分兼容 | `roze-migration` 已有 ledger，模型 diff/version directory 互操作待补 |
| `sql/globalid` | 待实现 | 缺全局 ID range/稳定性/升级测试 |

## 完成判定

全量同步只有在本文件不再包含“部分兼容”或“待实现”，所有“框架外适配”均已有独立
crate 与兼容套件，并且 SQLite、PostgreSQL、MySQL 及声明支持的其他后端在 CI 中运行真实
行为测试后才能宣布完成。当前阶段不能判定为“全部满足需求”。

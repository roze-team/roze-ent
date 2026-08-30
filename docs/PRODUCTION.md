# 生产部署基线

当前仓库提供可验证的生产配置模板和 CI 前置门禁，但这不等同于已经获得 24/72 小时长稳证据。
正式发布仍须完成 `services/roze-ent-api/ops/` 中要求的容量、故障注入、安全、备份恢复、
灰度和回滚验收。

## 配置与秘密

部署时将 `deploy/config/rest.production.yaml` 以只读方式挂载到容器或主机，例如
`/etc/roze/rest.production.yaml`，并设置：

```bash
export ROZE_CONFIG_PATH=/etc/roze/rest.production.yaml
export ROZE_ENT_DATABASE_URL='postgres://...'
export ROZE_ENT_REDIS_URL='redis://...'
export ROZE_ENT_JWT_SECRET='at-least-32-random-bytes-from-a-secret-manager'
```

不得把这三个值写回仓库。JWT 密钥应由秘密管理系统注入并按 `jwt_keys`/`jwt_active_key_id`
执行有重叠窗口的轮换。生产配置默认关闭 CORS，不信任转发身份头，并要求 Redis 承载分布式限流。

## 认证、权限与租户边界

全部 21 个业务路由都要求 Bearer JWT，并按资源执行最小权限校验：`users:*`、`pets:*`、
`groups:*` 和 `projects:*` 分别使用 `read` 或 `write` 权限。权限声明以 `roze-ent.api` 为来源，
生成的处理器负责验签与授权，OpenAPI 的 `x-roze-permissions` 用于客户端和网关同步契约。

Project 路由还执行双重租户校验：JWT 的 `tenant` 必须存在，并且必须与 `x-tenant-id`
完全一致。缺少身份返回未认证，缺少租户或跨租户请求返回禁止访问；业务逻辑不会信任请求头覆盖
JWT 租户。签发令牌时必须写入所需 `permissions` 与 `tenant`，不要向普通调用方发放通配权限。

服务监听明文 HTTP `0.0.0.0:3000`，只能放在提供 TLS 的入口网关、Ingress 或服务网格之后；
不要直接暴露到公网。若由代理传递客户端 IP，必须同时配置可信代理 CIDR，禁止全网段信任。

## 发布前检查

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd services/roze-ent-api
bash ops/production-verify.sh
```

仓库根 CI 会执行 production verify 并上传 JSON 报告。该报告是发布前置条件，不是长稳证明。
报告保留在 CI artifact；本地产生的 `ops/production-verify-report.json` 已加入忽略规则。

发布平台还必须完成：

- 数据库迁移 dry-run、备份恢复演练和回滚责任人确认；
- 认证、授权、跨租户访问、密钥轮换、审计与敏感数据扫描；
- 基准、阶梯、突发负载以及至少 24 小时 soak；
- 1% → 10% → 50% → 100% 灰度门禁及自动回滚；
- `/readyz`、错误率、P99、连接池、内存趋势和进程重启告警。

未完成上述证据前，只能标记为候选版本或受控生产试点，不能标记为广泛生产稳定。

# EtcdPilot 中文设计文档

日期：2026-06-22

## 1. 项目名称

项目名称：**EtcdPilot**

命名含义：

- `Etcd` 表示项目面向 etcd 服务管理。
- `Pilot` 表示驾驶舱、导航员，强调对 etcd 集群的可视化巡检、安全操作和集中管理。

EtcdPilot 比 `etcd-manager` 更适合作为产品名称：它简短、易记，也不会只局限在“管理工具”这个较窄的表达里。

## 2. 项目目标

EtcdPilot 是一款基于 Rust + Vue 3 的内部 etcd Web 管理台。

第一版重点支持：

- 巡检 etcd 集群、成员、健康状态、Key 和 Lease。
- 在权限控制和审计记录下进行日常 Key 运维。
- 同时支持配置文件内置集群和页面维护集群。
- 支持 Docker 容器化部署。
- Rust 后端统一提供 API，并托管 Vue 3 构建后的静态资源。

## 3. 第一版不做的内容

V1 暂不包含：

- 多租户组织隔离。
- OIDC、LDAP、AD 等企业认证集成。
- 审批流。
- 分布式 Worker 服务。
- 高级告警。
- 自动备份和恢复 etcd。

这些能力可以在权限、审计、集群管理和 Key 运维链路稳定后逐步增加。

## 4. 总体架构

EtcdPilot 采用 Rust 单体服务 + Vue 3 SPA 的方式。

```text
Browser
  |
  | Vue 3 SPA
  v
Rust Web Server
  |-- 静态资源服务
  |-- 登录与会话
  |-- RBAC 权限控制
  |-- 集群注册表
  |-- Etcd 客户端服务
  |-- 审计服务
  |-- 数据库存储层
  |
  | SQLite 或 PostgreSQL
  |
  v
Etcd Clusters
```

该架构的核心原则是：部署简单、模块清晰、风险可控。Rust 服务负责认证、授权、etcd 访问、审计记录和前端静态文件托管。

## 5. 技术选型

### 后端

- Rust stable
- `axum`：HTTP API、中间件、静态文件托管
- `tokio`：异步运行时
- `tonic` 或成熟 Rust etcd client：访问 etcd gRPC API
- `sqlx`：数据库访问
- SQLite：默认本地和容器部署数据库
- PostgreSQL：可选生产数据库
- `serde`：序列化和反序列化
- `validator`：请求参数校验
- `argon2`：密码哈希
- `tracing`：日志
- `rustls`：TLS 支持

### 前端

- Vue 3
- Vite
- TypeScript
- Pinia
- Vue Router
- Element Plus 或 Naive UI
- 可选 Monaco Editor，用于编辑较大的文本、JSON 或 YAML Value

## 6. 部署形态

开发期：

```text
backend: cargo run
frontend: npm run dev
```

生产期：

```text
npm run build
cargo build --release
./etcdpilot --config config.toml
```

生产环境中，Rust 服务同时提供：

```text
/api/*
/assets/*
/index.html
```

前端路由 fallback 到 `index.html`。

## 7. Docker 容器部署

EtcdPilot 必须支持 Docker 容器部署。

推荐容器内目录：

```text
/app/etcdpilot
/app/web/dist
/etc/etcdpilot/config.toml
/var/lib/etcdpilot/etcdpilot.db
/var/log/etcdpilot
/certs
```

容器暴露端口：

```text
8080/tcp
```

推荐环境变量：

```text
ETCD_MANAGER_CONFIG=/etc/etcdpilot/config.toml
ETCD_MANAGER_DATABASE_URL=sqlite:///var/lib/etcdpilot/etcdpilot.db
ETCD_MANAGER_HTTP_ADDR=0.0.0.0:8080
ETCD_MANAGER_SESSION_SECRET=replace-with-random-secret
RUST_LOG=info
```

推荐使用多阶段 Docker 构建：

1. 使用 Node 镜像构建 Vue 3 SPA。
2. 使用 Rust 镜像构建后端二进制。
3. 将 Rust 二进制和 Vue `dist` 拷贝到精简运行时镜像。

运行时镜像不需要 Node.js 和 Cargo。

示例运行方式：

```text
docker run \
  -p 8080:8080 \
  -v ./config.toml:/etc/etcdpilot/config.toml:ro \
  -v ./data:/var/lib/etcdpilot \
  -v ./certs:/certs:ro \
  -e ETCD_MANAGER_SESSION_SECRET=replace-with-random-secret \
  etcdpilot:latest
```

MVP 默认使用 SQLite，便于单容器部署。需要更强的数据持久化和运维能力时，可通过配置切换到 PostgreSQL。

## 8. 配置设计

EtcdPilot 支持混合集群配置：

- 配置文件内置集群：从 `config.toml` 加载，页面只读展示，不允许页面删除。
- 页面维护集群：由管理员在 Web 页面中新增、编辑、禁用和删除。

示例配置：

```toml
[server]
addr = "0.0.0.0:8080"
public_url = "http://localhost:8080"

[database]
url = "sqlite:///var/lib/etcdpilot/etcdpilot.db"

[security]
session_secret_env = "ETCD_MANAGER_SESSION_SECRET"
cookie_secure = false

[[clusters]]
id = "prod"
name = "Production"
endpoints = ["https://10.0.0.10:2379", "https://10.0.0.11:2379"]
readonly = true
tls_ca_cert = "/certs/prod/ca.pem"
tls_client_cert = "/certs/prod/client.pem"
tls_client_key = "/certs/prod/client-key.pem"

[[clusters]]
id = "dev"
name = "Development"
endpoints = ["http://host.docker.internal:2379"]
readonly = false
```

`readonly = true` 表示该集群只能巡检，后端会拒绝所有写操作，即使当前用户是管理员。

## 9. 后端模块

```text
config
  加载并校验 config.toml。
  提供配置文件内置集群。

auth
  登录、登出、会话创建、密码校验。

rbac
  管理用户、角色和权限点。
  在调用 etcd 服务前执行权限检查。

cluster
  合并配置文件集群和数据库集群。
  校验连接配置。
  创建 etcd 客户端。
  执行健康检查。

etcd
  封装 status、member、key-value、lease 操作。
  将 etcd 错误转换为统一 API 错误。

audit
  记录写操作和敏感读操作。

storage
  基于 sqlx 的数据库 repository。

api
  axum 路由、handler、参数校验和响应转换。

web
  托管 Vue dist。
  将前端路由 fallback 到 index.html。
```

## 10. 前端页面

### 登录页

- 用户名密码登录。
- 已登录用户自动进入仪表盘。
- 对账号错误、密码错误、会话过期给出清晰提示。

### 仪表盘

- 集群数量。
- 健康和异常集群统计。
- 最近审计事件。
- 快速进入集群详情和 Key 浏览器。

### 集群管理

- 展示配置文件集群和页面维护集群。
- 标识来源：`config` 或 `database`。
- 配置文件集群只读展示。
- 页面维护集群支持新增、编辑、禁用、测试连接和删除。
- TLS 证书路径和敏感字段尽量只写不读，避免泄露。

### 集群详情

- Endpoint 健康状态。
- Member 列表。
- Leader 信息。
- etcd 版本。
- DB size。
- Raft index。

### Key 浏览器

- 按 prefix 浏览。
- 搜索 key。
- 展示 key 元数据：revision、version、create revision、modify revision、lease。
- 查看 value。
- 有权限时编辑 value。
- 新建 key。
- 更新 key。
- 删除单个 key，并要求二次确认。

### Lease 管理

- Lease 列表。
- TTL。
- 关联 keys。
- V1 只读。
- Lease revoke 放到后续阶段。

### 用户与角色

- 用户列表。
- 创建、禁用、重置用户。
- 分配角色。
- 查看角色权限。
- 仅管理员可访问。

### 审计日志

- 操作人。
- 集群。
- 操作类型。
- Key 或 prefix。
- 请求摘要。
- 操作结果。
- 时间。
- 失败原因。

## 11. 权限模型

内置角色：

```text
admin
  系统完整管理权限。

operator
  集群巡检、Key 读写、后续阶段的导入导出。

readonly
  集群巡检、Key 读取、Lease 读取。
```

权限点：

```text
cluster:read
cluster:write
key:read
key:write
key:delete
key:delete_prefix
lease:read
lease:revoke
import_export:read
import_export:write
user:read
user:write
audit:read
```

V1 启用：

- `cluster:read`
- `cluster:write`
- `key:read`
- `key:write`
- `key:delete`
- `lease:read`
- `user:read`
- `user:write`
- `audit:read`

V1 先定义但不在页面开放：

- `key:delete_prefix`
- `lease:revoke`
- `import_export:read`
- `import_export:write`

## 12. API 草案

```text
POST   /api/auth/login
POST   /api/auth/logout
GET    /api/me

GET    /api/clusters
POST   /api/clusters
GET    /api/clusters/:id
PUT    /api/clusters/:id
DELETE /api/clusters/:id
POST   /api/clusters/:id/test

GET    /api/clusters/:id/status
GET    /api/clusters/:id/members

GET    /api/clusters/:id/kv?prefix=/foo
GET    /api/clusters/:id/kv/item?key=/foo/a
PUT    /api/clusters/:id/kv/item
DELETE /api/clusters/:id/kv/item

GET    /api/clusters/:id/leases
GET    /api/clusters/:id/leases/:lease_id

GET    /api/users
POST   /api/users
PUT    /api/users/:id
DELETE /api/users/:id

GET    /api/roles
GET    /api/audits
```

后续阶段 API：

```text
DELETE /api/clusters/:id/kv/prefix
POST   /api/clusters/:id/leases
DELETE /api/clusters/:id/leases/:lease_id
POST   /api/clusters/:id/export
POST   /api/clusters/:id/import/preview
POST   /api/clusters/:id/import/commit
```

## 13. 数据库模型

核心表：

```text
users
roles
user_roles
role_permissions
clusters
audit_logs
app_settings
```

`clusters` 只存页面维护的集群：

```text
id
name
endpoints
auth_type
username
password_secret_ref
tls_ca_cert
tls_client_cert
tls_client_key
readonly
disabled
created_at
updated_at
```

配置文件内置集群在运行时加载，并和数据库集群合并后展示。

`audit_logs` 存储：

```text
id
user_id
username
cluster_id
operation
resource_type
resource_key
request_summary
success
error_message
client_ip
created_at
```

审计日志不保存原始密钥、密码或完整敏感 value。

## 14. 安全规则

- 所有写操作必须经过认证、授权和审计。
- 单 Key 删除必须在 UI 二次确认。
- 配置文件内置集群不能从 UI 删除。
- `readonly` 集群在后端拒绝所有写操作。
- V1 不开放前缀删除。
- V1 不开放 Lease revoke。
- V1 不开放导入导出。
- Key value 响应大小需要限制。
- 大 value 需要用户显式点击后再加载。
- TLS key 路径和密钥不返回前端。
- 用户密码使用 Argon2 哈希。
- 会话使用 HttpOnly Cookie。
- 生产环境应启用 secure cookie 和 SameSite 保护。

## 15. 错误处理

API 返回统一错误结构：

```json
{
  "code": "ETCD_UNAVAILABLE",
  "message": "Cluster endpoint is unavailable",
  "request_id": "..."
}
```

常见错误码：

```text
UNAUTHENTICATED
FORBIDDEN
VALIDATION_ERROR
CLUSTER_NOT_FOUND
CLUSTER_READONLY
ETCD_UNAVAILABLE
ETCD_TIMEOUT
KEY_NOT_FOUND
VALUE_TOO_LARGE
INTERNAL_ERROR
```

前端处理原则：

- 表单校验错误展示在字段附近。
- 权限不足的操作隐藏或禁用。
- 集群不可达错误展示在集群详情页。
- 破坏性操作的确认文案必须具体到 key、prefix 或 lease。

## 16. MVP 范围

MVP 包含：

1. 登录和会话管理。
2. admin、operator、readonly 三类内置角色。
3. 配置文件集群和页面维护集群。
4. 集群连接测试。
5. 集群状态和 member 查看。
6. Key 浏览器。
7. 单 Key 新增、更新、删除。
8. Lease 只读查看。
9. 用户管理和角色分配。
10. 审计日志查看。
11. Docker 镜像和 Docker Compose 部署。

MVP 不包含：

1. 前缀删除。
2. Lease revoke。
3. 导入导出。
4. 企业 SSO。
5. 告警。

## 17. 测试策略

### 后端

- RBAC 权限检查单元测试。
- 配置解析和集群合并单元测试。
- 审计日志脱敏单元测试。
- 使用真实 etcd 容器做集成测试。
- 覆盖认证、授权、只读集群、Key 操作的 API 测试。

### 前端

- Key 浏览器组件测试。
- 权限控制下按钮显示和禁用测试。
- 集群表单组件测试。
- 登录、选择集群、读 Key、写 Key、删 Key、查看审计日志的 E2E 测试。

### Docker

- CI 构建 Docker 镜像。
- 使用 Docker Compose 启动服务。
- 验证 `/api/health`。
- 验证前端静态资源和路由 fallback。
- 验证 SQLite 挂载卷持久化。

## 18. 实施阶段

### 阶段一：项目基础

- Rust 后端工程。
- Vue 3 前端工程。
- 开发代理。
- Dockerfile 和 Compose。
- 健康检查接口。
- 静态资源托管。

### 阶段二：认证、权限和存储

- 数据库迁移。
- 管理员初始化。
- 登录登出。
- 会话中间件。
- 角色和权限控制。

### 阶段三：集群注册表

- 配置文件集群加载。
- 页面维护集群 CRUD。
- 运行时集群合并。
- 连接测试。

### 阶段四：etcd 巡检

- 集群状态。
- Member 列表。
- Lease 只读列表。

### 阶段五：Key 管理

- Prefix 浏览。
- 单 Key 读取。
- 单 Key 新增和更新。
- 单 Key 删除和确认。
- 所有写操作写入审计日志。

### 阶段六：管理 UI 和加固

- 用户管理。
- 角色分配。
- 审计日志 UI。
- 错误提示优化。
- Docker 部署验证。

## 19. 推荐默认决策

- UI 组件库：Element Plus。
- 数据库：MVP 默认 SQLite，保留 PostgreSQL 扩展能力。
- 会话：服务端 session + HttpOnly Cookie。
- 前端资源：V1 在 Docker 镜像中随 Rust 二进制一起发布，不强制编译进单文件二进制。

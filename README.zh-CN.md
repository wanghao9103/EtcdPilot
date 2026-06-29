# EtcdPilot

## 观测能力

- 查看 endpoint 级别的 etcd 健康和 raft 状态。
- 在 etcd 仍保留 revision 时读取指定 revision 的 key value。
- 实时监听 key 或 prefix 变化。
- Revision 历史依赖 etcd MVCC 保留策略，已经被 compact 的 revision 无法通过 EtcdPilot 读取。

EtcdPilot 是一个轻量级 etcd 管理控制台，用于通过 Web 界面浏览、编辑和审计 etcd 数据。项目后端使用 Rust Axum，前端使用 Vue 3 和 Vite，本地应用元数据使用 SQLite 存储。

## 功能特性

- 管理多个 etcd 集群并检查连接状态。
- 浏览、读取、编辑和删除 key-value 数据。
- 按 `/services/` 等可配置前缀查看服务注册数据。
- 查看 lease 以及关联的 key。
- 记录并浏览用户操作审计日志。
- 支持 admin、operator、readonly、viewer 等角色权限。
- 内置英文和简体中文界面。
- 支持 Docker 部署和本地开发流程。

## 项目结构

```text
.
├── backend/          # Rust Axum API 服务
├── frontend/         # Vue 3 + Vite Web 控制台
├── docs/             # 设计文档
├── scripts/          # 本地构建和同步脚本
├── config/           # 分环境配置文件
├── Dockerfile        # 容器镜像构建文件
└── docker-compose.yml
```

## 环境要求

- Rust 1.80 或更高版本
- Node.js 20 或更高版本
- npm
- 可访问的 etcd endpoint

如需容器化部署，可额外安装 Docker。

## 配置说明

后端默认读取 `config/config.test.toml`。现网示例配置为 `config/config.prod.toml`。可以通过 `ETCD_MANAGER_CONFIG` 指定其他配置文件路径。

重要配置项：

- `server.addr`：HTTP 监听地址，默认 `0.0.0.0:8080`。
- `database.url`：SQLite 数据库 URL。
- `security.session_secret_env`：保存会话密钥的环境变量名。
- `web.dist_dir`：后端托管的前端静态文件目录。
- `clusters`：预配置的 etcd 集群。

在非本地开发环境运行前，请设置足够强的 session secret：

```bash
export ETCD_MANAGER_SESSION_SECRET="replace-with-a-long-random-secret"
```

Windows PowerShell：

```powershell
$env:ETCD_MANAGER_SESSION_SECRET = "replace-with-a-long-random-secret"
```

## 本地开发

安装并构建前端，然后将构建产物同步到后端静态目录：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sync-web.ps1
```

如果前端依赖已经安装，可以跳过安装步骤：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sync-web.ps1 -SkipInstall
```

启动后端：

```bash
cd backend
cargo run
```

打开：

```text
http://127.0.0.1:8080
```

默认本地账号：

```text
admin / admin123
```

在任何共享环境中使用前，请先修改默认密码。

## VS Code 工作流

项目包含可共享的 VS Code 任务和启动配置：

- `.vscode/tasks.json`
- `.vscode/launch.json`

推荐安装：

- C/C++ 调试器，用于 Windows 下调试 Rust 后端
- Vue Language Features (Volar)

可以在 Run and Debug 面板中运行 `EtcdPilot: All (Backend + Frontend)`，也可以通过 `Tasks: Run Task` 运行 `dev: all`。

## Docker 启动

使用 Docker Compose 启动服务：

```bash
docker compose up --build
```

容器会暴露 `8080` 端口，并将 `config/config.prod.toml` 挂载到 `/etc/etcdpilot/config.toml`。

## 设计文档

- [中文设计文档](docs/superpowers/specs/2026-06-22-etcdpilot-design-zh.md)
- [英文设计文档](docs/superpowers/specs/2026-06-22-etcd-manager-design.md)

## 许可证

EtcdPilot 基于 [MIT License](LICENSE) 发布。

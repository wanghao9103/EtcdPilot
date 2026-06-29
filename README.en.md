# EtcdPilot

EtcdPilot is a lightweight etcd management console for browsing, editing, and auditing etcd data from a web UI. It uses a Rust Axum backend, a Vue 3 frontend, and SQLite for local application metadata.

## Features

- Manage multiple etcd clusters and check connectivity.
- Browse, read, edit, and delete key-value data.
- Inspect endpoint-level etcd health and raft status.
- Read key values at a specific revision when etcd still retains the revision.
- Watch key or prefix changes in real time.
- Inspect service registration data under configurable prefixes such as `/services/`.
- View leases and linked keys.
- Record and browse audit logs for user operations.
- Role-based permissions for admin, operator, readonly, and viewer users.
- Bilingual UI with English and Simplified Chinese locales.
- Docker and local development workflows.

## Project Structure

```text
.
├── backend/          # Rust Axum API service
├── frontend/         # Vue 3 + Vite web console
├── docs/             # Design documents
├── scripts/          # Local build and sync scripts
├── config/           # Environment configuration files
├── Dockerfile        # Container image build
└── docker-compose.yml
```

## Requirements

- Rust 1.80 or newer
- Node.js 20 or newer
- npm
- An accessible etcd endpoint

Docker is optional for containerized deployment.

## Configuration

The backend reads configuration from `config/config.test.toml` by default. Production settings live in `config/config.prod.toml`. You can override the path with `ETCD_MANAGER_CONFIG`.

Important settings:

- `server.addr`: HTTP listen address, default `0.0.0.0:8080`.
- `database.url`: SQLite database URL.
- `security.session_secret_env`: environment variable name that contains the session secret.
- `web.dist_dir`: frontend static file directory served by the backend.
- `clusters`: configured etcd clusters.

Revision history depends on etcd MVCC retention. Revisions compacted by etcd cannot be read from EtcdPilot.

Set a strong session secret before running outside local development:

```bash
export ETCD_MANAGER_SESSION_SECRET="replace-with-a-long-random-secret"
```

On Windows PowerShell:

```powershell
$env:ETCD_MANAGER_SESSION_SECRET = "replace-with-a-long-random-secret"
```

## Local Development

Install and build the frontend, then sync the generated files into the backend static directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sync-web.ps1
```

If frontend dependencies are already installed:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sync-web.ps1 -SkipInstall
```

Start the backend:

```bash
cd backend
cargo run
```

Open:

```text
http://127.0.0.1:8080
```

Default local account:

```text
admin / admin123
```

Change the default password before using the service in any shared environment.

## VS Code Workflow

Shared VS Code task and launch files are included:

- `.vscode/tasks.json`
- `.vscode/launch.json`

Recommended extensions:

- C/C++ debugger for Rust backend debugging on Windows
- Vue Language Features (Volar)

You can run `EtcdPilot: All (Backend + Frontend)` from the Run and Debug panel, or run the `dev: all` task from `Tasks: Run Task`.

## Docker

Start the service with Docker Compose:

```bash
docker compose up --build
```

The container exposes port `8080` and mounts `config/config.prod.toml` into `/etc/etcdpilot/config.toml`.

## Portable Release Packages

Build a local portable package:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package.ps1 -Version dev -Runtime windows-x64
```

Release packages are written to `artifacts/`. Each package contains the backend binary, `web/dist`, `config`, `data`, and startup scripts.

Create GitHub release packages by pushing a tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The release workflow builds:

- Windows portable `.zip`
- Windows installer `.exe`
- Linux portable `.tar.gz`
- Linux RPM package
- macOS portable `.tar.gz`
- macOS `.pkg` installer

The macOS `.pkg` is unsigned by default. Public macOS distribution should add Developer ID signing and notarization.

## Design Documents

- [Chinese design document](docs/superpowers/specs/2026-06-22-etcdpilot-design-zh.md)
- [English design document](docs/superpowers/specs/2026-06-22-etcd-manager-design.md)

## License

EtcdPilot is released under the [MIT License](LICENSE).

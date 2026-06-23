# EtcdPilot Design

Date: 2026-06-22

## 1. Goal

Build EtcdPilot, an internal etcd Web management console with Rust and Vue 3.

The first version focuses on safe inspection and controlled daily operations:

- Inspect etcd clusters, members, health, keys, and leases.
- Manage keys with role-based access control and audit logs.
- Manage etcd cluster connection profiles from both server config and the Web UI.
- Deploy as a Docker container or as a single Rust service that serves the Vue SPA.

## 2. Non-Goals For V1

The first version does not include:

- Multi-tenant organization isolation.
- OIDC, LDAP, or AD integration.
- Approval workflows.
- Distributed worker services.
- Advanced alerting.
- Automatic etcd backup and restore.

These can be added after the core management, permission, and audit flows are stable.

## 3. Recommended Architecture

Use a Rust single-service backend that exposes APIs and serves the Vue 3 SPA.

```text
Browser
  |
  | Vue 3 SPA
  v
Rust Web Server
  |-- Static SPA Server
  |-- Auth / Session
  |-- RBAC
  |-- Cluster Registry
  |-- Etcd Client Service
  |-- Audit Service
  |-- Storage Repository
  |
  | SQLite or PostgreSQL
  |
  v
Etcd Clusters
```

This keeps deployment simple while preserving clear module boundaries. The Rust service owns authentication, authorization, etcd access, audit logging, and static frontend hosting.

## 4. Technology Choices

### Backend

- Rust stable
- `axum` for HTTP APIs, middleware, and static file serving
- `tokio` for async runtime
- `tonic` or a mature Rust etcd client for etcd gRPC access
- `sqlx` for database access
- SQLite for default local/container deployment
- PostgreSQL as an optional production database
- `serde` for request and response serialization
- `validator` for input validation
- `argon2` for password hashing
- `tracing` for logs
- `rustls` for TLS support

### Frontend

- Vue 3
- Vite
- TypeScript
- Pinia
- Vue Router
- Element Plus or Naive UI
- Optional Monaco Editor for large text, JSON, or YAML values

### Deployment

- Docker image containing the Rust binary and built Vue assets
- Docker Compose example for quick local deployment
- Persistent volume for config, database, logs, and certificates

## 5. Runtime Layout

Recommended container paths:

```text
/app/etcdpilot
/app/web/dist
/etc/etcdpilot/config.toml
/var/lib/etcdpilot/etcdpilot.db
/var/log/etcdpilot
/certs
```

The container should expose:

```text
8080/tcp
```

The service should support these environment variables:

```text
ETCD_MANAGER_CONFIG=/etc/etcdpilot/config.toml
ETCD_MANAGER_DATABASE_URL=sqlite:///var/lib/etcdpilot/etcdpilot.db
ETCD_MANAGER_HTTP_ADDR=0.0.0.0:8080
ETCD_MANAGER_SESSION_SECRET=replace-with-random-secret
RUST_LOG=info
```

## 6. Docker Deployment Design

Use a multi-stage Docker build:

1. Build the Vue SPA with Node.
2. Build the Rust backend with Cargo.
3. Copy the Rust binary and Vue `dist` into a slim runtime image.

The runtime image should not require Node.js or Cargo.

Example container behavior:

```text
docker run \
  -p 8080:8080 \
  -v ./config.toml:/etc/etcdpilot/config.toml:ro \
  -v ./data:/var/lib/etcdpilot \
  -v ./certs:/certs:ro \
  -e ETCD_MANAGER_SESSION_SECRET=replace-with-random-secret \
  etcdpilot:latest
```

Recommended Docker Compose services:

- `etcdpilot`
- Optional `postgres` if PostgreSQL is used instead of SQLite

For the MVP, SQLite is the default because it keeps the container deployment small. PostgreSQL should be supported by configuration for teams that need external database persistence.

## 7. Configuration

The application supports mixed cluster configuration:

- Config-defined clusters: loaded from `config.toml`, shown in the UI, not removable from the UI.
- Database-defined clusters: created and managed from the Web UI.

Example config:

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

`readonly = true` means the cluster can be inspected but write operations are blocked even for admin users.

## 8. Backend Modules

```text
config
  Load and validate config.toml.
  Provide config-defined clusters.

auth
  Login, logout, session creation, password verification.

rbac
  Map users, roles, and permissions.
  Enforce permissions before handlers call etcd services.

cluster
  Merge config-defined and database-defined clusters.
  Validate connection settings.
  Create etcd clients.
  Run health checks.

etcd
  Status, member, key-value, and lease operations.
  Normalize etcd errors into API errors.

audit
  Record all write operations and sensitive read operations.

storage
  Database repositories through sqlx.

api
  Axum routes, handlers, request validation, response mapping.

web
  Serve Vue dist files.
  Fallback frontend routes to index.html.
```

## 9. Frontend Pages

### Login

- Username and password login.
- Redirect authenticated users to the dashboard.
- Show clear messages for invalid credentials or expired sessions.

### Dashboard

- Cluster count.
- Healthy and unhealthy cluster summary.
- Recent audit events.
- Quick entry to cluster details and key browser.

### Cluster Management

- List config-defined and database-defined clusters.
- Show source: `config` or `database`.
- Config-defined clusters are read-only in the UI.
- Database-defined clusters can be created, edited, disabled, tested, and deleted.
- TLS fields are write-only where possible. Certificate paths are shown only when safe.

### Cluster Detail

- Endpoint health.
- Member list.
- Leader information.
- Version.
- DB size.
- Raft index.

### Key Browser

- Prefix-based browsing.
- Key search.
- Key metadata: revision, version, create revision, modify revision, lease.
- Value viewer.
- Value editor for permitted users.
- Create key.
- Update key.
- Delete single key with confirmation.

### Lease Management

- Lease list.
- TTL.
- Associated keys.
- V1 is read-only.
- Lease revoke is planned for a later phase.

### User And Role Management

- User list.
- Create, disable, and reset users.
- Role assignment.
- Role permission view.
- Admin-only access.

### Audit Logs

- Operator.
- Cluster.
- Operation type.
- Key or prefix.
- Request summary.
- Result.
- Timestamp.
- Error message if failed.

## 10. RBAC

Built-in roles:

```text
admin
  Full system management.

operator
  Cluster inspection, key read/write, import/export in later phases.

readonly
  Cluster inspection, key read, lease read.
```

Permission points:

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

V1 enables:

- `cluster:read`
- `cluster:write`
- `key:read`
- `key:write`
- `key:delete`
- `lease:read`
- `user:read`
- `user:write`
- `audit:read`

V1 keeps these permissions defined but not exposed in the UI:

- `key:delete_prefix`
- `lease:revoke`
- `import_export:read`
- `import_export:write`

## 11. API Draft

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

Later phase APIs:

```text
DELETE /api/clusters/:id/kv/prefix
POST   /api/clusters/:id/leases
DELETE /api/clusters/:id/leases/:lease_id
POST   /api/clusters/:id/export
POST   /api/clusters/:id/import/preview
POST   /api/clusters/:id/import/commit
```

## 12. Database Model

Tables:

```text
users
roles
user_roles
role_permissions
clusters
audit_logs
app_settings
```

`clusters` stores only UI-managed clusters:

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

Config-defined clusters are loaded at runtime and merged with database-defined clusters for listing.

`audit_logs` should store:

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

Do not store raw secrets or full sensitive values in audit logs.

## 13. Safety Rules

- All write operations require authentication, authorization, and audit logging.
- Single-key delete requires confirmation in the UI.
- Config-defined clusters cannot be deleted from the UI.
- `readonly` clusters reject all write operations at the backend layer.
- Prefix delete is not enabled in V1.
- Lease revoke is not enabled in V1.
- Import and export are not enabled in V1.
- Key value response size should be limited.
- Large values should be fetched and displayed with explicit user action.
- TLS key paths and secrets are never returned to the frontend.
- Passwords are hashed with Argon2.
- Sessions use HttpOnly cookies.
- In production, cookies should be secure and same-site protected.

## 14. Error Handling

The API returns structured errors:

```json
{
  "code": "ETCD_UNAVAILABLE",
  "message": "Cluster endpoint is unavailable",
  "request_id": "..."
}
```

Common error codes:

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

Frontend behavior:

- Show validation errors inline.
- Show permission errors as disabled actions or clear messages.
- Show etcd connectivity errors on cluster detail pages.
- Keep destructive operation confirmations explicit and specific.

## 15. MVP Scope

The MVP includes:

1. Login and session management.
2. Built-in admin, operator, and readonly roles.
3. Config-defined and UI-defined cluster registry.
4. Cluster connection test.
5. Cluster status and members.
6. Key browser.
7. Single key create, update, and delete.
8. Lease read-only view.
9. User and role assignment management.
10. Audit log view.
11. Docker image and Docker Compose deployment.

The MVP excludes:

1. Prefix delete.
2. Lease revoke.
3. Import and export.
4. Enterprise SSO.
5. Alerting.

## 16. Testing Strategy

### Backend

- Unit tests for RBAC permission checks.
- Unit tests for config parsing and cluster merge behavior.
- Unit tests for audit log summary redaction.
- Integration tests with a real etcd container.
- API tests for auth, authorization, readonly clusters, and key operations.

### Frontend

- Component tests for the key browser.
- Component tests for permission-based button visibility.
- Component tests for cluster forms.
- E2E tests for login, cluster selection, key read, key write, key delete, and audit lookup.

### Docker

- Build the image in CI.
- Start with Docker Compose.
- Verify `/api/health`.
- Verify static frontend route fallback.
- Verify SQLite persistence through a mounted volume.

## 17. Implementation Phases

### Phase 1: Project Foundation

- Rust workspace or single crate backend.
- Vue 3 app.
- Shared development proxy.
- Basic Dockerfile and Compose file.
- Health endpoint and static file serving.

### Phase 2: Auth, RBAC, And Storage

- Database migrations.
- Admin bootstrap.
- Login/logout.
- Session middleware.
- Role and permission enforcement.

### Phase 3: Cluster Registry

- Config cluster loading.
- Database cluster CRUD.
- Runtime cluster merge.
- Connection test.

### Phase 4: Etcd Inspection

- Cluster status.
- Member list.
- Lease read-only list.

### Phase 5: Key Management

- Prefix browsing.
- Single key read.
- Single key create/update.
- Single key delete with confirmation.
- Audit logging for all writes.

### Phase 6: Admin UI And Hardening

- User management.
- Role assignment.
- Audit log UI.
- Error polishing.
- Docker deployment verification.

## 18. Open Decisions

These choices can be made during implementation without changing the product design:

- Use Element Plus or Naive UI.
- Use SQLite only for MVP or implement PostgreSQL support immediately.
- Use server-side sessions or signed session cookies.
- Embed Vue `dist` into the Rust binary or copy it beside the binary in the Docker image.

Recommended defaults:

- Element Plus.
- SQLite first, with database URL abstraction ready for PostgreSQL.
- Server-side sessions with HttpOnly cookie.
- Copy Vue `dist` beside the binary in Docker for V1; consider binary embedding later.

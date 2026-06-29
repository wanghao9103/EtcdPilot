use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::{
    auth::{self, AuthContext},
    db,
    error::{AppError, Result},
    etcd,
    models::{
        AuditLogRow, ClusterInfo, ConfiguredCluster, KvItem, KvRequest, LoginRequest, LoginResponse,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct PrefixQuery {
    pub prefix: Option<String>,
}

#[derive(Deserialize)]
pub struct ServiceQuery {
    pub prefix: Option<String>,
    pub cluster_id: Option<String>,
}

#[derive(Deserialize)]
pub struct KvQuery {
    pub key: Option<String>,
    pub revision: Option<i64>,
}

#[derive(Deserialize)]
pub struct KvHistoryQuery {
    pub key: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct KvWatchQuery {
    pub prefix: String,
}

#[derive(Serialize)]
pub struct ServiceItem {
    pub cluster_id: String,
    pub cluster_name: String,
    pub key: String,
    pub service_name: String,
    pub service_id: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub service: String,
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/clusters", get(list_clusters).post(create_cluster))
        .route(
            "/api/clusters/:id",
            get(get_cluster).put(update_cluster).delete(delete_cluster),
        )
        .route("/api/clusters/:id/test", post(test_cluster))
        .route("/api/clusters/:id/status", get(cluster_status))
        .route(
            "/api/clusters/:id/endpoints/status",
            get(cluster_endpoint_statuses),
        )
        .route("/api/clusters/:id/members", get(cluster_members))
        .route("/api/clusters/:id/kv", get(list_kv))
        .route(
            "/api/clusters/:id/kv/item",
            get(get_kv_item).put(put_kv_item).delete(delete_kv_item),
        )
        .route("/api/clusters/:id/kv/history", get(get_kv_history))
        .route("/api/clusters/:id/kv/watch", get(watch_kv))
        .route("/api/clusters/:id/leases", get(list_leases))
        .route("/api/clusters/:id/leases/:lease_id", get(get_lease))
        .route("/api/services", get(list_services))
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/:id", put(update_user).delete(delete_user))
        .route("/api/roles", get(list_roles))
        .route("/api/audits", get(list_audits))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "etcdpilot-backend".to_string(),
    })
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    let user = auth::verify_login(&state, &payload.username, &payload.password).await?;
    let token = auth::issue_session(&user, &state.config)?;
    let perms = auth::role_permissions(&user.role);
    let user_response = LoginResponse {
        user_id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        permissions: perms.clone(),
    };

    let mut response = (StatusCode::OK, Json(user_response)).into_response();
    let cookie = auth::build_cookie(
        &token,
        state.config.security.session_ttl_seconds,
        state.config.security.cookie_secure,
    );
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        cookie.parse().expect("set-cookie"),
    );
    Ok(response)
}

async fn logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, auth::clear_cookie())],
        Json(serde_json::json!({ "ok": true })),
    )
}

async fn me(headers: HeaderMap, State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    let user = auth::require_user(&headers, &state.config)?;
    Ok(Json(serde_json::json!({
        "user_id": user.user_id,
        "username": user.username,
        "role": user.role,
        "permissions": user.permissions,
    })))
}

async fn list_clusters(State(state): State<AppState>) -> Json<Vec<ClusterInfo>> {
    let mut merged = Vec::new();
    for item in &state.config.clusters {
        merged.push(ClusterInfo {
            id: item.id.clone(),
            name: item.name.clone(),
            endpoints: item.endpoints.clone(),
            readonly: item.readonly,
            disabled: false,
            source: "config".to_string(),
        })
    }
    if let Ok(mut list) = db::list_db_clusters(&state.pool).await {
        merged.append(&mut list);
    }
    Json(merged)
}

async fn create_cluster(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ConfiguredCluster>,
) -> Result<(StatusCode, Json<ClusterInfo>)> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:write")?;
    db::create_db_cluster(&state.pool, &payload).await?;
    let resp = ClusterInfo {
        id: payload.id,
        name: payload.name,
        endpoints: payload.endpoints,
        readonly: payload.readonly,
        disabled: false,
        source: "database".to_string(),
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn get_cluster(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClusterInfo>> {
    if let Some(item) = state
        .config
        .clusters
        .iter()
        .find(|item| item.id == id)
        .map(|item| ClusterInfo {
            id: item.id.clone(),
            name: item.name.clone(),
            endpoints: item.endpoints.clone(),
            readonly: item.readonly,
            disabled: false,
            source: "config".to_string(),
        })
    {
        return Ok(Json(item));
    }
    let list = db::list_db_clusters(&state.pool).await?;
    let found = list
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| AppError::NotFound("cluster not found".to_string()))?;
    Ok(Json(found))
}

async fn update_cluster(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:write")?;
    if state.config.clusters.iter().any(|item| item.id == id) {
        return Err(AppError::Validation(
            "config cluster cannot be edited".to_string(),
        ));
    }
    Ok(Json(serde_json::json!({"updated": id})))
}

async fn delete_cluster(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:write")?;
    if state.config.clusters.iter().any(|item| item.id == id) {
        return Err(AppError::Validation(
            "config cluster cannot be deleted".to_string(),
        ));
    }
    db::delete_db_cluster(&state.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn test_cluster(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    auth::require_user(&headers, &state.config)?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let connected = etcd::ping_cluster(&cluster).await?;
    Ok(Json(
        serde_json::json!({ "cluster_id": id, "connected": connected }),
    ))
}

async fn cluster_status(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let status = etcd::status(&cluster).await?;
    Ok(Json(status))
}

async fn cluster_endpoint_statuses(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::EndpointStatusResponse>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let status = etcd::endpoint_statuses(&cluster).await?;
    Ok(Json(status))
}

async fn cluster_members(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let members = etcd::members(&cluster).await?;
    Ok(Json(
        serde_json::json!({ "cluster_id": id, "members": members }),
    ))
}

async fn list_kv(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PrefixQuery>,
) -> Result<Json<Vec<KvItem>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let data = etcd::list_kv(&cluster, query.prefix).await?;
    Ok(Json(data))
}

async fn get_kv_item(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<KvQuery>,
) -> Result<Json<Option<KvItem>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let key = query.key.unwrap_or_default();
    let item = etcd::get_kv_item(&cluster, key, query.revision).await?;
    Ok(Json(item))
}

async fn get_kv_history(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<KvHistoryQuery>,
) -> Result<Json<crate::models::KvHistoryResponse>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let history = etcd::kv_history(&cluster, query.key, query.limit.unwrap_or(20)).await?;
    Ok(Json(history))
}

async fn watch_kv(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<KvWatchQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let stream = etcd::watch_prefix(&cluster, query.prefix).await?;
    let events = stream.map(|item| {
        let event = match item {
            Ok(payload) => Event::default()
                .event(payload.event_type.clone())
                .json_data(payload)
                .unwrap_or_else(|_| Event::default().event("error").data("serialize failed")),
            Err(err) => Event::default().event("error").data(err.to_string()),
        };
        Ok::<Event, Infallible>(event)
    });
    Ok(Sse::new(events))
}

async fn put_kv_item(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<KvRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:write")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    etcd::put_kv(
        &cluster,
        payload.key.clone(),
        payload.value.clone(),
        payload.lease,
    )
    .await?;
    log_audit(
        &state,
        &user,
        &id,
        "key:write",
        "kv",
        Some(&payload.key),
        "upsert key",
        true,
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"ok": true}))))
}

async fn delete_kv_item(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<KvQuery>,
) -> Result<StatusCode> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:delete")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let key = query
        .key
        .ok_or_else(|| AppError::Validation("key required".to_string()))?;
    etcd::delete_kv(&cluster, key.clone()).await?;
    log_audit(
        &state,
        &user,
        &id,
        "key:delete",
        "kv",
        Some(&key),
        "delete key",
        true,
        None,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_leases(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "lease:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let leases = etcd::list_leases(&cluster).await?;
    Ok(Json(leases))
}

async fn list_services(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<ServiceQuery>,
) -> Result<Json<Vec<ServiceItem>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:read")?;

    let clusters = all_clusters(&state).await?;
    let target = query.cluster_id.unwrap_or_default();
    let prefixes = parse_service_prefixes(query.prefix);

    let mut items = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();
    for cluster in clusters {
        if !target.is_empty() && cluster.id != target {
            continue;
        }

        let resolved = etcd::resolve_cluster_by_id(&state, &cluster.id).await?;
        for prefix in &prefixes {
            let kvs = etcd::list_kv(&resolved, Some(prefix.clone())).await?;
            for kv in kvs {
                let dedupe_key = format!("{}:{}", cluster.id, kv.key);
                if !seen_keys.insert(dedupe_key) {
                    continue;
                }
                let (service_name, service_id) = split_service_key(&kv.key);
                let metadata = parse_json_or_none(&kv.value);
                items.push(ServiceItem {
                    cluster_id: cluster.id.clone(),
                    cluster_name: cluster.name.clone(),
                    key: kv.key,
                    service_name,
                    service_id,
                    value: kv.value.clone(),
                    address: parse_service_address(&kv.value),
                    metadata,
                });
            }
        }
    }

    Ok(Json(items))
}

async fn get_lease(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((_id, lease_id)): Path<(String, i64)>,
) -> Result<Json<serde_json::Value>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "lease:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &_id).await?;
    let details = etcd::get_lease(&cluster, lease_id).await?;
    Ok(Json(details))
}

async fn list_users(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "user:read")?;
    let rows = db::list_users(&state.pool).await?;
    let list = rows
        .into_iter()
        .map(|u| serde_json::json!({"id":u.id,"username":u.username,"role":u.role,"disabled":u.disabled}))
        .collect();
    Ok(Json(list))
}

async fn create_user(headers: HeaderMap, State(state): State<AppState>) -> Result<StatusCode> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "user:write")?;
    Ok(StatusCode::CREATED)
}

async fn update_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let _ = id;
    auth::require_user(&headers, &state.config)?;
    Ok(StatusCode::OK)
}

async fn delete_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let _ = id;
    auth::require_user(&headers, &state.config)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_roles() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "roles": ["admin","operator","readonly"]
    }))
}

async fn list_audits(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditLogRow>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "audit:read")?;
    let records = db::list_audit_logs(&state.pool, 100).await?;
    Ok(Json(records))
}

fn require_permission(user: &AuthContext, perm: &str) -> Result<()> {
    auth::require_permission(user, perm)
}

async fn log_audit(
    state: &AppState,
    user: &AuthContext,
    cluster_id: &str,
    operation: &str,
    resource_type: &str,
    key: Option<&str>,
    summary: &str,
    success: bool,
    error: Option<&str>,
) -> Result<()> {
    db::save_audit_log(
        &state.pool,
        user.user_id,
        &user.username,
        cluster_id,
        operation,
        resource_type,
        key,
        summary,
        success,
        error,
        None,
    )
    .await
}

async fn all_clusters(state: &AppState) -> Result<Vec<ClusterInfo>> {
    let mut merged = Vec::new();
    merged.extend(state.config.clusters.iter().map(|item| ClusterInfo {
        id: item.id.clone(),
        name: item.name.clone(),
        endpoints: item.endpoints.clone(),
        readonly: item.readonly,
        disabled: false,
        source: "config".to_string(),
    }));

    let mut db_clusters = db::list_db_clusters(&state.pool).await?;
    merged.append(&mut db_clusters);
    Ok(merged)
}

fn parse_service_prefixes(raw: Option<String>) -> Vec<String> {
    let Some(raw) = raw else {
        return vec!["/services/".to_string()];
    };

    let prefixes: Vec<String> = raw
        .split(&[',', '\n', ';'][..])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect();

    if prefixes.is_empty() {
        vec!["/services/".to_string()]
    } else {
        prefixes
    }
}

fn split_service_key(key: &str) -> (String, String) {
    let mut segments: Vec<&str> = key
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    if segments.first() == Some(&"services") {
        segments = segments[1..].to_vec();
    }

    if segments.is_empty() {
        return ("unknown".to_string(), "default".to_string());
    }

    if segments.len() == 1 {
        return (segments[0].to_string(), "default".to_string());
    }

    let service_id = segments.last().unwrap_or(&"default").to_string();
    let service_name = segments[segments.len() - 2].to_string();
    (service_name, service_id)
}

fn parse_json_or_none(value: &str) -> Option<Value> {
    serde_json::from_str::<Value>(value).ok()
}

fn parse_service_address(value: &str) -> Option<String> {
    let parsed = parse_json_or_none(value)?;
    let obj = parsed.as_object()?;

    if let Some(address) = obj.get("address").and_then(Value::as_str) {
        return Some(address.to_string());
    }

    if let Some(endpoints) = obj.get("endpoints").and_then(Value::as_array) {
        if let Some(first) = endpoints.first().and_then(Value::as_str) {
            return Some(first.to_string());
        }
    }

    if let Some(host) = obj.get("host").and_then(Value::as_str) {
        let port = obj
            .get("port")
            .and_then(Value::as_u64)
            .or_else(|| obj.get("port").and_then(Value::as_str).and_then(|p| p.parse().ok()))?;
        return Some(format!("{host}:{port}"));
    }

    None
}

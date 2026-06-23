use std::{fs, path::Path};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    error::Result,
    models::{AuditLogRow, ClusterInfo, ConfiguredCluster, UserInfo, UserRow},
};

pub async fn new_pool(url: &str) -> Result<SqlitePool> {
    let mut last_error: Option<AppError> = None;
    let candidates = candidate_pool_paths(url);
    for path in &candidates {
        if let Err(err) = ensure_sqlite_parent_exists(path) {
            last_error = Some(err);
            continue;
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        match SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(err) => last_error = Some(AppError::Database(err)),
        }
    }

    Err(AppError::Internal(format!(
        "failed to initialize sqlite pool, tried: {:?}, last_error: {:?}",
        candidates, last_error
    )))
}

fn candidate_pool_paths(url: &str) -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(primary) = parse_sqlite_path(url) {
        paths.push(primary);
    }
    if let Some(fallback) = fallback_sqlite_path(url) {
        if !paths.contains(&fallback) {
            paths.push(fallback);
        }
    }

    if paths.is_empty() {
        vec!["data/etcdpilot.db".to_string()]
    } else {
        paths
    }
}

fn parse_sqlite_path(url: &str) -> Option<String> {
    if !url.starts_with("sqlite:") {
        return None;
    }

    if let Some(path) = url.strip_prefix("sqlite:///") {
        Some(path.to_string())
    } else if let Some(path) = url.strip_prefix("sqlite://") {
        Some(path.trim_start_matches('/').to_string())
    } else if let Some(path) = url.strip_prefix("sqlite:") {
        Some(path.to_string())
    } else {
        None
    }
}

fn fallback_sqlite_path(url: &str) -> Option<String> {
    let primary_path = parse_sqlite_path(url)?;
    if primary_path == ":memory:" {
        return Some(primary_path);
    }

    let file_name = Path::new(&primary_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("etcdpilot.db");
    Some(format!("./{}", file_name))
}

fn ensure_sqlite_parent_exists(path: &str) -> Result<()> {
    if path == ":memory:" {
        return Ok(());
    }

    let file_path = Path::new(path);
    if file_path.file_name().is_none() {
        return Ok(());
    }
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            AppError::Internal(format!("failed to create sqlite parent dir: {err}"))
        })?;
    }
    Ok(())
}

pub async fn init_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'readonly',
            disabled INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clusters (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            endpoints TEXT NOT NULL,
            auth_type TEXT DEFAULT 'none',
            username TEXT,
            password_secret_ref TEXT,
            tls_ca_cert TEXT,
            tls_client_cert TEXT,
            tls_client_key TEXT,
            readonly INTEGER NOT NULL DEFAULT 0,
            disabled INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_logs (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            cluster_id TEXT NOT NULL,
            operation TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_key TEXT,
            request_summary TEXT NOT NULL,
            success INTEGER NOT NULL DEFAULT 1,
            error_message TEXT,
            client_ip TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn ensure_default_admin(pool: &SqlitePool) -> Result<()> {
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM users WHERE username = 'admin'")
        .fetch_one(pool)
        .await?;
    if exists == 0 {
        let hash = super::auth::hash_password("admin123");
        sqlx::query(
            "INSERT INTO users (username, password_hash, role) VALUES ('admin', ?, 'admin')",
        )
        .bind(hash)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn find_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, (i64, String, String, String, i64)>(
        "SELECT id, username, password_hash, role, disabled FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(
        row.map(|(id, username, password_hash, role, disabled)| UserRow {
            id,
            username,
            password_hash,
            role,
            disabled: disabled != 0,
        }),
    )
}

pub async fn list_users(pool: &SqlitePool) -> Result<Vec<UserInfo>> {
    let rows = sqlx::query_as::<_, (i64, String, String, i64)>(
        "SELECT id, username, role, disabled FROM users ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, username, role, disabled)| UserInfo {
            id,
            username,
            role,
            disabled: disabled != 0,
        })
        .collect())
}

pub async fn list_db_clusters(pool: &SqlitePool) -> Result<Vec<ClusterInfo>> {
    let rows = sqlx::query_as::<_, (String, String, String, i64, i64)>(
        "SELECT id,name,endpoints,readonly,disabled FROM clusters ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    let mut clusters = Vec::new();
    for (id, name, endpoints_json, readonly, disabled) in rows {
        let endpoints: Vec<String> = serde_json::from_str(&endpoints_json).unwrap_or_default();
        clusters.push(ClusterInfo {
            id,
            name,
            endpoints,
            readonly: readonly != 0,
            disabled: disabled != 0,
            source: "database".to_string(),
        });
    }
    Ok(clusters)
}

pub async fn create_db_cluster(pool: &SqlitePool, cluster: &ConfiguredCluster) -> Result<()> {
    let endpoints_json =
        serde_json::to_string(&cluster.endpoints).unwrap_or_else(|_| "[]".to_string());
    sqlx::query(
        "INSERT INTO clusters (id,name,endpoints,auth_type,username,password_secret_ref,tls_ca_cert,tls_client_cert,tls_client_key,readonly,disabled)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(&cluster.id)
    .bind(&cluster.name)
    .bind(endpoints_json)
    .bind(cluster.auth_type.clone().unwrap_or_else(|| "none".to_string()))
    .bind(cluster.username.clone())
    .bind(cluster.password_secret_ref.clone())
    .bind(cluster.tls_ca_cert.clone())
    .bind(cluster.tls_client_cert.clone())
    .bind(cluster.tls_client_key.clone())
    .bind(if cluster.readonly { 1i64 } else { 0i64 })
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_db_cluster(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM clusters WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn save_audit_log(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    cluster_id: &str,
    operation: &str,
    resource_type: &str,
    resource_key: Option<&str>,
    request_summary: &str,
    success: bool,
    error_message: Option<&str>,
    client_ip: Option<&str>,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO audit_logs (id,user_id,username,cluster_id,operation,resource_type,resource_key,request_summary,success,error_message,client_ip)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(user_id)
        .bind(username)
        .bind(cluster_id)
        .bind(operation)
        .bind(resource_type)
        .bind(resource_key)
        .bind(request_summary)
        .bind(if success { 1i64 } else { 0i64 })
        .bind(error_message)
        .bind(client_ip)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_audit_logs(pool: &SqlitePool, limit: i64) -> Result<Vec<AuditLogRow>> {
    let rows = sqlx::query_as::<_, (String,i64,String,String,String,String,Option<String>,String,i64,Option<String>,Option<String>,i64)>(
        "SELECT id, user_id, username, cluster_id, operation, resource_type, resource_key, request_summary, success, error_message, client_ip, created_at
         FROM audit_logs ORDER BY created_at DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                user_id,
                username,
                cluster_id,
                operation,
                resource_type,
                resource_key,
                request_summary,
                success,
                error_message,
                client_ip,
                created_at,
            )| AuditLogRow {
                id,
                user_id,
                username,
                cluster_id,
                operation,
                resource_type,
                resource_key,
                request_summary,
                success: success != 0,
                error_message,
                client_ip,
                created_at,
            },
        )
        .collect())
}

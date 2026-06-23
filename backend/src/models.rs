use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub clusters: Vec<ConfiguredCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    #[serde(default = "default_session_secret_env")]
    pub session_secret_env: String,
    #[serde(default)]
    pub cookie_secure: bool,
    #[serde(default)]
    pub session_ttl_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebConfig {
    #[serde(default = "default_web_root")]
    pub dist_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredCluster {
    pub id: String,
    pub name: String,
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password_secret_ref: Option<String>,
    #[serde(default)]
    pub tls_ca_cert: Option<String>,
    #[serde(default)]
    pub tls_client_cert: Option<String>,
    #[serde(default)]
    pub tls_client_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub id: String,
    pub name: String,
    pub endpoints: Vec<String>,
    pub readonly: bool,
    pub disabled: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: String,
    pub user_id: i64,
    pub username: String,
    pub cluster_id: String,
    pub operation: String,
    pub resource_type: String,
    pub resource_key: Option<String>,
    pub request_summary: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub client_ip: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvRequest {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub lease: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvItem {
    pub key: String,
    pub value: String,
    pub revision: i64,
    pub version: i64,
    pub create_revision: i64,
    pub mod_revision: i64,
    pub lease: Option<i64>,
}

pub fn default_session_secret_env() -> String {
    "ETCD_MANAGER_SESSION_SECRET".to_string()
}

pub fn default_web_root() -> String {
    "web/dist".to_string()
}

use std::{env, fs};

use crate::error::Result;
use crate::models::{AppConfig, DatabaseConfig, SecurityConfig, ServerConfig, WebConfig};

pub fn load_config() -> Result<AppConfig> {
    let config_path = resolve_config_path();
    let raw = fs::read_to_string(config_path).unwrap_or_else(|_| {
        format!(
            "[server]\naddr = \"0.0.0.0:8080\"\n\n[database]\nurl = \"{}\"\n\n[security]\nsession_secret_env = \"ETCD_MANAGER_SESSION_SECRET\"\ncookie_secure = false\nsession_ttl_seconds = 3600\n\n[web]\ndist_dir = \"web/dist\"\n\n[[clusters]]\nid = \"demo\"\nname = \"demo\"\nendpoints = [\"http://127.0.0.1:2379\"]\nreadonly = true\n",
            default_database_url()
        )
    });
    let parsed: AppConfig = toml::from_str(&raw).unwrap_or_else(|_| AppConfig {
        server: ServerConfig {
            addr: "0.0.0.0:8080".to_string(),
        },
        database: DatabaseConfig {
            url: default_database_url(),
        },
        security: SecurityConfig {
            session_secret_env: "ETCD_MANAGER_SESSION_SECRET".to_string(),
            cookie_secure: false,
            session_ttl_seconds: 3600,
        },
        web: WebConfig {
            dist_dir: "web/dist".to_string(),
        },
        clusters: Vec::new(),
    });
    Ok(parsed)
}

fn resolve_config_path() -> String {
    if let Ok(path) = env::var("ETCD_MANAGER_CONFIG") {
        return path;
    }
    if fs::metadata("config/config.test.toml").is_ok() {
        return "config/config.test.toml".to_string();
    }
    if fs::metadata("../config/config.test.toml").is_ok() {
        return "../config/config.test.toml".to_string();
    }
    if fs::metadata("config.toml").is_ok() {
        return "config.toml".to_string();
    }
    if fs::metadata("../config.toml").is_ok() {
        return "../config.toml".to_string();
    }
    "config/config.test.toml".to_string()
}

fn default_database_url() -> String {
    "sqlite://./data/etcdpilot.db".to_string()
}

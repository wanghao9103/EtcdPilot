use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::http::HeaderMap;
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::{
    error::{AppError, Result},
    models::{AppConfig, UserRow},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub perms: Vec<String>,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
}

pub const COOKIE_NAME: &str = "etcdpilot_session";

pub fn role_permissions(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "cluster:read",
            "cluster:write",
            "key:read",
            "key:write",
            "key:delete",
            "lease:read",
            "user:read",
            "user:write",
            "audit:read",
        ],
        "operator" => vec![
            "cluster:read",
            "key:read",
            "key:write",
            "key:delete",
            "lease:read",
        ],
        _ => vec!["cluster:read", "key:read", "lease:read"],
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

pub async fn verify_login(
    state: &crate::AppState,
    username: &str,
    password: &str,
) -> Result<UserRow> {
    let user = db::find_user_by_username(&state.pool, username).await?;
    let Some(user) = user else {
        return Err(AppError::Unauthenticated);
    };
    if user.disabled {
        return Err(AppError::Forbidden);
    }
    if !verify_password(password, &user.password_hash) {
        return Err(AppError::Unauthenticated);
    }
    Ok(user)
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut rand::thread_rng());
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("password hash")
        .to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn build_cookie(jwt: &str, ttl: i64, secure: bool) -> String {
    let mut cookie = format!(
        "{name}={value}; Path=/; HttpOnly; Max-Age={ttl}; SameSite=Lax",
        name = COOKIE_NAME,
        value = jwt,
        ttl = ttl
    );
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

pub fn clear_cookie() -> String {
    format!(
        "{name}=; Path=/; HttpOnly; Max-Age=0; SameSite=Lax",
        name = COOKIE_NAME
    )
}

pub fn issue_session(user: &UserRow, config: &AppConfig) -> Result<String> {
    let exp = now_unix() + config.security.session_ttl_seconds.max(300) as usize;
    let perms = role_permissions(&user.role);
    let claims = SessionClaims {
        sub: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        perms,
        exp,
    };
    let secret = state_secret(config)?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("token encode failed: {e}")))
}

pub fn extract_token(headers: &HeaderMap) -> Result<Option<String>> {
    let cookie = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthenticated)?;
    for item in cookie.split(';') {
        let v = item.trim();
        if let Some(rest) = v.strip_prefix(&(COOKIE_NAME.to_string() + "=")) {
            return Ok(Some(rest.to_string()));
        }
    }
    Ok(None)
}

pub fn require_user(headers: &HeaderMap, config: &AppConfig) -> Result<AuthContext> {
    let token = extract_token(headers)?.ok_or(AppError::Unauthenticated)?;
    let secret = state_secret(config)?;
    let decoded = decode::<SessionClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| map_jwt_err(e))?;
    let claims = decoded.claims;
    Ok(AuthContext {
        user_id: claims.sub,
        username: claims.username,
        role: claims.role,
        permissions: claims.perms,
    })
}

pub fn require_permission(user: &AuthContext, perm: &str) -> Result<()> {
    if user.permissions.iter().any(|item| item == perm) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn now_unix() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

fn state_secret(config: &AppConfig) -> Result<String> {
    let secret_env = config.security.session_secret_env.trim();
    match env::var(secret_env) {
        Ok(secret) if !secret.trim().is_empty() => Ok(secret),
        Ok(secret) if secret.trim().is_empty() => Err(AppError::config(format!(
            "session secret env `{secret_env}` is empty"
        ))),
        _ if cfg!(debug_assertions) => {
            tracing::warn!(
                "missing session secret env `{}`; fallback dev secret is used temporarily",
                secret_env
            );
            Ok("etcdpilot-dev-session-secret-insecure".to_string())
        }
        _ => Err(AppError::config(format!(
            "missing session secret env `{secret_env}`, set it before starting"
        ))),
    }
}

fn map_jwt_err(err: JwtError) -> AppError {
    match err.to_string().as_str() {
        _ => AppError::Unauthenticated,
    }
}

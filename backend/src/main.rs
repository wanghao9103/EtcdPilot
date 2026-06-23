use axum::Router;
use tower_http::services::ServeDir;
use tracing_subscriber::{fmt, EnvFilter};

mod auth;
mod config;
mod db;
mod error;
mod etcd;
mod handlers;
mod models;

#[derive(Clone)]
pub struct AppState {
    pub config: models::AppConfig,
    pub pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    if let Err(err) = run().await {
        tracing::error!(message = %err, "startup failed");
        std::process::exit(1);
    }
}

async fn run() -> error::Result<()> {
    let config = config::load_config()?;
    tracing::info!("loaded database url: {}", config.database.url);
    let pool = db::new_pool(&config.database.url).await?;
    db::init_schema(&pool).await?;
    db::ensure_default_admin(&pool).await?;

    let state = AppState {
        config: config.clone(),
        pool,
    };
    let dist_dir = state.config.web.dist_dir.clone();
    let static_service = ServeDir::new(dist_dir);
    let addr = config
        .server
        .addr
        .parse::<std::net::SocketAddr>()
        .map_err(|err| error::AppError::Internal(format!("invalid server addr: {err}")))?;

    let api = handlers::api_routes().with_state(state.clone());
    let app = Router::new()
        .merge(api)
        .nest_service("/", static_service)
        .with_state(state);

    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| error::AppError::Internal(format!("failed to bind listener: {err}")))?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|err| error::AppError::Internal(format!("server error: {err}")))?;
    Ok(())
}

mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod sse;

use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use handlers::auth::AppState;
use sse::SseBroadcaster;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rust_restaurant_system=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    // Ensure uploads directory exists
    let upload_dir = PathBuf::from(&config.upload_dir);
    if !upload_dir.exists() {
        fs::create_dir_all(&upload_dir).await?;
    }

    tracing::info!("Connecting to PostgreSQL database...");
    let pool = db::create_pool(&config.database_url).await?;

    tracing::info!("Running database migrations...");
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::warn!("Migration notice: {}", e);
    }

    let sse = SseBroadcaster::new();

    let state = AppState {
        db: pool,
        config: config.clone(),
        sse,
    };

    let app = routes::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Restaurant System server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

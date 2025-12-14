use dotenvy::dotenv;
use tracing::info;

mod app;
mod common;
mod config;
mod infrastructure;
mod middleware;
mod modules;
mod routes;
mod state;

use config::settings::AppConfig;
use infrastructure::db::pool::connect_to_db;
use infrastructure::redis::client::RedisService;
use infrastructure::storage::s3::StorageService;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    
    info!("🚀 Initializing HiuraMovie Backend...");

    // 1. Load Config
    let config = AppConfig::new().expect("Failed to load configuration");

    // 2. Connect to Database (Postgres)
    let db_pool = connect_to_db(&config.database_url)
        .await
        .expect("Failed to connect to Database");

    // 3. Connect to Redis
    let redis_service = RedisService::new(&config.redis_url)
        .await
        .expect("Failed to connect to Redis");

    // 4. Connect to Storage (S3/MinIO)
    let storage_service = StorageService::new(
        &config.minio_url,
        &config.minio_bucket,
        &config.minio_access_key,
        &config.minio_secret_key,
    ).await;

    // 5. Create App State
    let state = AppState::new(config.clone(), db_pool, redis_service, storage_service);

    // 6. Start Server
    let app = app::create_app(state).await;
    
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    info!("✅ Server running on http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}

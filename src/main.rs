use dotenvy::dotenv;
use tracing::info;

mod app;
mod common;
mod config;
mod docs;
mod infrastructure;
mod middleware;
mod modules;
mod routes;
mod state;
mod workers;

use config::settings::AppConfig;
use infrastructure::db::pool::connect_to_db;
use infrastructure::redis::client::RedisService;
use infrastructure::storage::s3::StorageService;
use infrastructure::queue::rabbitmq::RabbitMqService;
use state::AppState;

const HIURA_BANNER: &str = r#"
██╗  ██╗██╗██╗   ██╗██████╗  █████╗ 
██║  ██║██║██║   ██║██╔══██╗██╔══██╗
███████║██║██║   ██║██████╔╝███████║
██╔══██║██║██║   ██║██╔══██╗██╔══██║
██║  ██║██║╚██████╔╝██║  ██║██║  ██║
╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
Hiura Movie Backend — Rust Native Binary
"#;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // tracing_subscriber::fmt::init(); // Replace this generic init
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .init();
    
    println!("{HIURA_BANNER}");
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

    // Ensure all required buckets exist
    let buckets = vec![
        &config.minio_bucket,
        &config.minio_bucket_thumbnails,
    ];
    
    for bucket in buckets {
        if let Err(e) = storage_service.ensure_bucket_exists(bucket).await {
            tracing::warn!("Failed to ensure bucket '{}' exists: {}", bucket, e);
        }
    }

    // 5. Connect to RabbitMQ
    let queue_service = RabbitMqService::new(&config.rabbitmq_url)
        .await
        .expect("Failed to connect to RabbitMQ");

    // 6. Create App State
    let state = AppState::new(config.clone(), db_pool, redis_service, storage_service, queue_service);

    // 7. Start Workers
    let worker_state = state.clone();
    tokio::spawn(async move {
        workers::transcoder::start_transcoder_worker(worker_state).await;
    });

    // 8. Start Server
    let app = app::create_app(state).await;
    
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    info!("✅ Server running on http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}

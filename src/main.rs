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

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting server...");

    let app = app::create_app().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}

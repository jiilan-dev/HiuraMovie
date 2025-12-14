use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use sqlx::{Pool, Postgres, ConnectOptions};
use std::str::FromStr;
use std::time::Duration;
use tracing::info;
use tracing::log::LevelFilter;

pub type DbPool = Pool<Postgres>;

pub async fn connect_to_db(connection_string: &str) -> Result<DbPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(connection_string)?
        .log_statements(LevelFilter::Debug);

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect_with(options)
        .await?;

    info!("✅ Connected to PostgreSQL");
    Ok(pool)
}

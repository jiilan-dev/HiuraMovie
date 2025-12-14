use redis::{Client, aio::MultiplexedConnection};
use tracing::info;

#[derive(Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub async fn new(connection_string: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(connection_string)?;
        
        // Test connection
        let _conn = client.get_multiplexed_async_connection().await?;
        
        info!("✅ Connected to Redis");
        Ok(Self { client })
    }

    pub async fn get_conn(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_async_connection().await
    }
}

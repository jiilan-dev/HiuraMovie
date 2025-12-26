use anyhow::{anyhow, Result};
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection,
    ConnectionProperties,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Clone)]
pub struct RabbitMqService {
    url: String,
    conn: Arc<Mutex<Connection>>,
    channel: Arc<Mutex<Channel>>,
}

impl RabbitMqService {
    async fn connect(url: &str) -> Result<(Connection, Channel)> {
        info!("Connecting to RabbitMQ at {}", url);
        let conn = Connection::connect(url, ConnectionProperties::default())
            .await
            .map_err(|e| anyhow!("Failed to connect to RabbitMQ: {}", e))?;

        let channel = conn
            .create_channel()
            .await
            .map_err(|e| anyhow!("Failed to create channel: {}", e))?;

        info!("Connected to RabbitMQ");
        Ok((conn, channel))
    }

    pub async fn new(url: &str) -> Result<Self> {
        let (conn, channel) = Self::connect(url).await?;

        Ok(Self {
            url: url.to_string(),
            conn: Arc::new(Mutex::new(conn)),
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    async fn reconnect(&self) -> Result<()> {
        warn!("RabbitMQ connection dropped, reconnecting...");
        let (conn, channel) = Self::connect(&self.url).await?;
        *self.conn.lock().await = conn;
        *self.channel.lock().await = channel;
        Ok(())
    }

    async fn publish_internal(&self, queue: &str, payload: &[u8]) -> Result<()> {
        let channel = self.channel.lock().await;

        // Ensure queue exists
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| anyhow!("Failed to declare queue: {}", e))?;

        channel
            .basic_publish(
                "",
                queue,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default().with_delivery_mode(2), // Persistent
            )
            .await
            .map_err(|e| anyhow!("Failed to publish message: {}", e))?
            .await
            .map_err(|e| anyhow!("Failed to confirm publication: {}", e))?;

        Ok(())
    }

    pub async fn publish(&self, queue: &str, payload: &[u8]) -> Result<()> {
        if let Err(e) = self.publish_internal(queue, payload).await {
            warn!("RabbitMQ publish failed: {}. Retrying after reconnect.", e);
            self.reconnect().await?;
            self.publish_internal(queue, payload).await?;
        }

        Ok(())
    }

    pub async fn get_channel(&self) -> Arc<Mutex<Channel>> {
        self.channel.clone()
    }
}

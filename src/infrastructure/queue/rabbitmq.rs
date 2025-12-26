use anyhow::{anyhow, Result};
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection,
    ConnectionProperties,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Clone)]
pub struct RabbitMqService {
    conn: Arc<Mutex<Connection>>,
    channel: Arc<Mutex<Channel>>,
}

impl RabbitMqService {
    pub async fn new(url: &str) -> Result<Self> {
        info!("Connecting to RabbitMQ at {}", url);
        let conn = Connection::connect(url, ConnectionProperties::default())
            .await
            .map_err(|e| anyhow!("Failed to connect to RabbitMQ: {}", e))?;

        let channel = conn
            .create_channel()
            .await
            .map_err(|e| anyhow!("Failed to create channel: {}", e))?;

        info!("Connected to RabbitMQ");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    pub async fn publish(&self, queue: &str, payload: &[u8]) -> Result<()> {
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

    pub async fn get_channel(&self) -> Arc<Mutex<Channel>> {
        self.channel.clone()
    }
}

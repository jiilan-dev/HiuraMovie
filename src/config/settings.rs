use serde::Deserialize;
use crate::config::env::{self, EnvKey};

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub minio_url: String,
    pub minio_bucket: String,
    pub minio_bucket_thumbnails: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub jwt_secret: String,
    pub rabbitmq_url: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, std::env::VarError> {
        Ok(Self {
            server_port: env::get_parsed(EnvKey::ServerPort, 3000),
            database_url: env::get(EnvKey::DatabaseUrl)?,
            redis_url: env::get(EnvKey::RedisUrl)?,
            minio_url: env::get(EnvKey::MinioUrl)?,
            minio_bucket: env::get(EnvKey::MinioBucket)?,
            minio_bucket_thumbnails: env::get(EnvKey::MinioBucketThumbnails).unwrap_or("thumbnails".to_string()),
            minio_access_key: env::get(EnvKey::MinioAccessKey)?,
            minio_secret_key: env::get(EnvKey::MinioSecretKey)?,
            jwt_secret: env::get(EnvKey::JwtSecret)?,
            rabbitmq_url: env::get(EnvKey::RabbitMqUrl).unwrap_or("amqp://guest:guest@localhost:5672".to_string()),
        })
    }
}

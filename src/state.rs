use crate::config::settings::AppConfig;
use crate::infrastructure::db::pool::DbPool;
use crate::infrastructure::redis::client::RedisService;
use crate::infrastructure::storage::s3::StorageService;
use crate::infrastructure::queue::rabbitmq::RabbitMqService;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: DbPool,
    pub redis: RedisService,
    pub storage: StorageService,
    pub queue: RabbitMqService,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        db: DbPool,
        redis: RedisService,
        storage: StorageService,
        queue: RabbitMqService,
    ) -> Self {
        Self {
            config,
            db,
            redis,
            storage,
            queue,
        }
    }
}

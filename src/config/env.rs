use std::env;
use std::str::FromStr;

pub enum EnvKey {
    ServerPort,
    DatabaseUrl,
    RedisUrl,
    MinioUrl,
    MinioBucket,
    MinioAccessKey,
    MinioSecretKey,
    JwtSecret,
}

impl EnvKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnvKey::ServerPort => "APP_PORT",
            EnvKey::DatabaseUrl => "DATABASE_URL",
            EnvKey::RedisUrl => "REDIS_URL",
            EnvKey::MinioUrl => "MINIO_ENDPOINT",
            EnvKey::MinioBucket => "MINIO_BUCKET_VIDEOS",
            EnvKey::MinioAccessKey => "AWS_ACCESS_KEY_ID",
            EnvKey::MinioSecretKey => "AWS_SECRET_ACCESS_KEY",
            EnvKey::JwtSecret => "JWT_SECRET",
        }
    }
}

pub fn get(key: EnvKey) -> Result<String, env::VarError> {
    env::var(key.as_str())
}

pub fn get_or(key: EnvKey, default: &str) -> String {
    env::var(key.as_str()).unwrap_or_else(|_| default.to_string())
}

pub fn get_parsed<T: FromStr>(key: EnvKey, default: T) -> T {
    match get(key) {
        Ok(val) => val.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

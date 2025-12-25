use crate::modules::auth::model::{User, UserRole};
use anyhow::Result;
use redis::AsyncCommands;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub struct AuthRepository;

impl AuthRepository {
    pub async fn create_user(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
        full_name: &str,
    ) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash, full_name, role)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, full_name, role as "role: UserRole", password_hash, created_at, updated_at
            "#,
            username,
            email,
            password_hash,
            full_name,
            UserRole::User as UserRole
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, full_name, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, full_name, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn store_refresh_token(
        redis: &mut redis::aio::MultiplexedConnection,
        user_id: Uuid,
        refresh_token: &str,
        ttl_seconds: usize,
    ) -> Result<()> {
        let key = format!("refresh_token:{}", user_id);
        redis.set_ex(key, refresh_token, ttl_seconds).await?;
        Ok(())
    }

    pub async fn get_refresh_token(
        redis: &mut redis::aio::MultiplexedConnection,
        user_id: Uuid,
    ) -> Result<Option<String>> {
        let key = format!("refresh_token:{}", user_id);
        let token: Option<String> = redis.get(key).await?;
        Ok(token)
    }

    pub async fn delete_refresh_token(
        redis: &mut redis::aio::MultiplexedConnection,
        user_id: Uuid,
    ) -> Result<()> {
        let key = format!("refresh_token:{}", user_id);
        redis.del(key).await?;
        Ok(())
    }
}

use super::dto::{AuthResponse, LoginRequest, RegisterRequest, TokenClaims, UserResponse};
use super::model::UserRole;
use super::repository::AuthRepository;
use crate::state::AppState;
use crate::common::security;
use anyhow::{anyhow, Result};
use jsonwebtoken::{encode, get_current_timestamp, EncodingKey, Header};
use time::Duration;
use uuid::Uuid;

pub struct AuthService;

impl AuthService {
    pub async fn register(state: AppState, req: RegisterRequest) -> Result<UserResponse> {
        // Check if user exists
        if AuthRepository::find_user_by_email(&state.db, &req.email)
            .await?
            .is_some()
        {
            return Err(anyhow!("Email already exists"));
        }
        
        if AuthRepository::find_user_by_username(&state.db, &req.username)
            .await?
            .is_some()
        {
            return Err(anyhow!("Username already exists"));
        }

        // Hash password
        let password_hash = security::hash_password(&req.password)?;

        // Create user
        let user = AuthRepository::create_user(
            &state.db,
            &req.username,
            &req.email,
            &password_hash,
            &req.full_name,
        )
        .await?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name: user.full_name,
            role: user.role.to_string(),
        })
    }

    pub async fn login(state: AppState, req: LoginRequest) -> Result<(AuthResponse, String)> {
        let user = AuthRepository::find_user_by_email(&state.db, &req.email)
            .await?
            .ok_or_else(|| anyhow!("Invalid credentials"))?;

        // Verify password
        security::verify_password(&req.password, &user.password_hash)
            .map_err(|_| anyhow!("Invalid credentials"))?;

        // Generate tokens
        let access_token = Self::create_access_token(user.id, user.role.clone())?;
        // Format: user_id:random_uuid
        let refresh_token = format!("{}:{}", user.id, Uuid::new_v4());

        // Store refresh token in Redis (7 days)
        let mut redis_conn = state.redis.get_conn().await?;
        AuthRepository::store_refresh_token(
            &mut redis_conn,
            user.id,
            &refresh_token,
            7 * 24 * 60 * 60, // 7 days in seconds
        )
        .await?;
        

        let user_response = UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name: user.full_name,
            role: user.role.to_string(),
        };

        Ok((
            AuthResponse {
                access_token,
                user: user_response,
            },
            refresh_token,
        ))
    }
    
    pub async fn logout(state: AppState, user_id: Uuid) -> Result<()> {
        let mut redis_conn = state.redis.get_conn().await?;
        AuthRepository::delete_refresh_token(&mut redis_conn, user_id).await?;
        Ok(())
    }

    pub async fn block_token(state: AppState, token: String, ttl: usize) -> Result<()> {
        let mut redis_conn = state.redis.get_conn().await?;
        let key = format!("blocked_token:{}", token);
        // Use set_ex to blocking token with expiration
        redis_conn.set_ex(key, "blocked", ttl).await?;
        Ok(())
    }
    
    pub async fn refresh_access(state: AppState, refresh_token: String, user_id: Uuid) -> Result<AuthResponse> {
        let mut redis_conn = state.redis.get_conn().await?;
        
        // Verify token in Redis
        let stored_token = AuthRepository::get_refresh_token(&mut redis_conn, user_id).await?;
        if let Some(token) = stored_token {
            if token != refresh_token {
                return Err(anyhow!("Invalid refresh token"));
            }
        } else {
            return Err(anyhow!("Refresh token expired or invalid"));
        }
        
        // Get user to ensure role is up to date
        // Note: In a real app we might cache user role in redis too to avoid DB hit on refresh
        // For now, we query DB
             let user = sqlx::query_as!(
            crate::modules::auth::model::User,
            r#"
            SELECT id, username, email, full_name, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(anyhow!("User not found"))?;

        let access_token = Self::create_access_token(user.id, user.role.clone())?;
        
        let user_response = UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name: user.full_name,
            role: user.role.to_string(),
        };

        Ok(AuthResponse {
            access_token,
            user: user_response,
        })
    }

    fn create_access_token(user_id: Uuid, role: UserRole) -> Result<String> {
        let expiration = get_current_timestamp() as usize + 15 * 60; // 15 minutes
        
        let claims = TokenClaims {
            sub: user_id,
            role: role.to_string(),
            exp: expiration,
            iat: get_current_timestamp() as usize,
        };

        // TODO: Move secret to config
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| anyhow!(e.to_string()))
    }
}

use anyhow::Context;
use bb8_redis::{bb8::Pool, redis::AsyncCommands, RedisConnectionManager};
use thiserror::Error;
use uuid::Uuid;

use crate::schema::RefreshToken;

pub struct TokenStore {
    redis_pool: Pool<RedisConnectionManager>,
    access_token_ttl: u64,
    refresh_token_ttl: u64,
}

#[derive(Debug, Error)]
pub enum TokenValidationError {
    #[error("Token not found")]
    NotFound,

    #[error("Fingerprint mismatch")]
    FingerprintMismatch,
    
    #[error("Other error: {0}")]
    Other(String),
}

impl TokenStore {
    pub fn new(redis_pool: Pool<RedisConnectionManager>) -> Self {
        // TODO: ttl from_config
        Self {
            redis_pool,
            access_token_ttl: 60 * 15,
            refresh_token_ttl: 60 * 60 * 24 * 30,
        }
    }

    fn get_key(&self, token: &str) -> String {
        format!("refresh_token:{}", token)
    }

    pub async fn get_refresh_token(&self, token: &str) -> anyhow::Result<Option<RefreshToken>> {
        let mut conn = self.redis_pool.get().await.context("Failed to get Redis connection")?;
        
        let json_data: Option<String> = conn
            .get(self.get_key(token))
            .await
            .context("Failed to get token from Redis")?;
            
        match json_data {
            Some(data) => {
                let token_data: RefreshToken = serde_json::from_str(&data)
                    .context("Failed to deserialize token data")?;
                Ok(Some(token_data))
            },
            None => Ok(None)
        }
    }

    pub async fn generate_refresh_token(&self, token_data: RefreshToken) -> anyhow::Result<String> {
        let mut conn = self.redis_pool
            .get()
            .await
            .context("Failed to get Redis connection")?;
        
        let token = Uuid::new_v4().to_string();
        
        let json_data = serde_json::to_string(&token_data)
            .context("Failed to serialize token data")?;

        conn.set_ex::<_, _, ()>(
            self.get_key(&token), 
            json_data, 
            self.refresh_token_ttl
        )
        .await
        .context("Failed to save token to Redis")?;
        
        Ok(token)
    }

    pub async fn invalidate_refresh_token(&self, token: &str) -> anyhow::Result<()> {
        let mut conn = self.redis_pool.get().await.context("Failed to get Redis connection")?;
        
        conn.del::<_, ()>(format!("refresh_token:{}", token))
            .await
            .context("Failed to delete token from Redis")?;
        
        Ok(())
    }

    pub async fn revoke_access_token(&self, token: &str) -> anyhow::Result<()> {
        let mut conn = self.redis_pool.get().await.context("Failed to get Redis connection")?;
        
        conn.set_ex::<_, _, ()>(
            self.get_key(token), 
            "1", 
            self.access_token_ttl
        )
        .await
        .context("Failed to add token to revocation list")?;
        
        Ok(())
    }

    pub async fn is_access_token_revoked(&self, token: &str) -> anyhow::Result<bool> {
        let mut conn = self.redis_pool.get().await.context("Failed to get Redis connection")?;
        
        let exists: bool = conn
            .exists(self.get_key(token))
            .await
            .context("Failed to check token revocation status")?;
        
        Ok(exists)
    }

    pub async fn validate_refresh_token(
        &self, 
        token: &str, 
        fingerprint: &str
    ) -> Result<RefreshToken, TokenValidationError> {
        let token_data = match self.get_refresh_token(token).await {
            Ok(Some(data)) => data,
            
            Ok(None) => return Err(TokenValidationError::NotFound),
            
            Err(e) => {
                return Err(TokenValidationError::Other(e.to_string()));
            }
        };
        
        if token_data.fingerprint != fingerprint {
            return Err(TokenValidationError::FingerprintMismatch);
        }
        
        Ok(token_data)
    }

    pub async fn rotate_refresh_token(
        &self, 
        old_token: &str,
        new_token_data: RefreshToken
    ) -> anyhow::Result<String> {
        self.get_refresh_token(old_token).await?;
        
        self.invalidate_refresh_token(old_token).await?;
        
        let new_token = self.generate_refresh_token(new_token_data).await?;
        
        Ok(new_token)
    }
}
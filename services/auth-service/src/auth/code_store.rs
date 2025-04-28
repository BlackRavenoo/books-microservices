use chrono::Utc;
use uuid::Uuid;
use anyhow::{Result, anyhow, Context};
use bb8_redis::{
    bb8::Pool, 
    redis::{AsyncCommands, RedisError},
    RedisConnectionManager
};
use serde_json;
use crate::schema::AuthCode;

pub struct CodeStore {
    redis_pool: Pool<RedisConnectionManager>,
    code_expiry_seconds: u64,
    key_prefix: String,
}

impl CodeStore {
    pub fn new(redis_pool: Pool<RedisConnectionManager>) -> Self {
        // TODO: from_config
        let code_expiry_seconds = 10 * 60;
        
        Self {
            redis_pool,
            code_expiry_seconds,
            key_prefix: "auth-code:".to_owned(),
        }
    }
    
    fn get_key(&self, code: &str) -> String {
        format!("{}{}", self.key_prefix, code)
    }
    
    pub async fn create_code(
        &self,
        client_id: String,
        user_id: i32,
        redirect_uri: String,
        code_challenge: String,
        code_challenge_method: String,
    ) -> Result<String> {
        let code = Uuid::new_v4().to_string();
        
        let auth_code = AuthCode {
            code: code.clone(),
            client_id,
            user_id,
            redirect_uri,
            expires_at: Utc::now() + chrono::Duration::seconds(self.code_expiry_seconds as i64),
            code_challenge,
            code_challenge_method,
        };
        
        let serialized = serde_json::to_string(&auth_code)
            .context("Failed to serialize auth code")?;
        
        let key = self.get_key(&code);
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection")?;
        
        conn.set_ex::<_, _, ()>(key, serialized, self.code_expiry_seconds)
            .await
            .map_err(|e: RedisError| anyhow!("Redis error: {}", e))?;
        
        Ok(code)
    }
    
    pub async fn consume_code(&self, code: &str) -> Result<AuthCode> {
        let key = self.get_key(code);
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection")?;
        
        let value: Option<String> = conn.get(&key).await
            .map_err(|e: RedisError| anyhow!("Redis error: {}", e))?;
        
        let value = value.ok_or_else(|| anyhow!("Invalid or expired code"))?;
        
        let auth_code: AuthCode = serde_json::from_str(&value)
            .context("Failed to deserialize auth code")?;
        
        conn.del::<_, ()>(&key).await
            .map_err(|e: RedisError| anyhow!("Redis error: {}", e))?;
        
        Ok(auth_code)
    }
}
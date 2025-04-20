use std::{fmt::Display, hash::Hash, sync::Arc};

use bb8_redis::{bb8::{Pool, PooledConnection, RunError}, redis::{AsyncCommands, FromRedisValue, RedisError, ToRedisArgs}, RedisConnectionManager};
use bincode::{config::Configuration, Decode, Encode};
use metrics::describe_counter;
use moka::future::{Cache, CacheBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use metrics::counter;

use crate::expiry::{CacheExpiry, Expiration};

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("Encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("Redis pool error: {0}")]
    RedisPool(#[from] RunError<RedisError>),
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError)
}

pub struct HybridCache<K, V> {
    prefix: String,
    local_cache: Cache<K, (Expiration, V)>,
    redis_pool: Arc<Pool<RedisConnectionManager>>
}

impl<K, V> HybridCache<K, V> 
where 
    K: AsRef<str> + FromRedisValue + Display + ToRedisArgs + Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Decode<()> + Encode + Send + Sync + 'static,
{
    pub fn new(prefix: String, redis_pool: Arc<Pool<RedisConnectionManager>>, capacity: u64) -> Self {
        let local_cache = CacheBuilder::new(capacity)
            .expire_after(CacheExpiry)
            .build();

        describe_counter!("cache.requests.total", "Total cache requests");
        describe_counter!("cache.hits.total", "Cache hits");
        
        Self {
            prefix,
            local_cache,
            redis_pool
        }
    }

    fn format_key(&self, key: &K) -> String {
        format!("{}_{}", self.prefix, key)
    }

    pub async fn get(&self, key: &K, expiry: Expiration) -> Result<Option<V>, CacheError> {
        counter!("cache.requests.total", "layer" => "l1").increment(1);

        if let Some((_, v)) = self.local_cache.get(key).await {
            counter!("cache.hits.total", "layer" => "l1").increment(1);
            return Ok(Some(v))
        }

        counter!("cache.requests.total", "layer" => "l2").increment(1);

        let mut con = self.get_redis_connection().await?;

        let result = con
            .get::<String, Option<Vec<u8>>>(self.format_key(key))
            .await;

        match result {
            Ok(Some(value)) => {
                counter!("cache.hits.total", "layer" => "l2").increment(1);
                let result = bincode::decode_from_slice::<V, Configuration>(
                    &value,
                    bincode::config::standard(),
                );
                match result {
                    Ok((value, _)) => {
                        self.local_cache.insert(key.clone(), (expiry, value.clone())).await;
                        Ok(Some(value))
                    }
                    Err(e) => Err(CacheError::Decode(e)),
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::error!("Failed to get value from redis: {:#?}", e);
                Err(CacheError::Redis(e))
            }
        }
    }

    pub async fn set(&self, key: K, value: V, expiry: Expiration) -> Result<(), CacheError> {
        self.local_cache.insert(key.clone(), (expiry, value.clone())).await;
        
        let mut con = self.get_redis_connection().await?;

        let encoded = bincode::encode_to_vec(&value, bincode::config::standard())?;
        con.set_ex::<_, _, ()>(self.format_key(&key), encoded, expiry.get_seconds()).await?;

        Ok(())
    }

    pub async fn invalidate(&self, key: K) -> Result<(), CacheError> {
        self.local_cache
            .invalidate(&key)
            .await;

        let mut con = self.get_redis_connection().await?;
        con.del::<_, ()>(self.format_key(&key)).await?;

        Ok(())
    }

    async fn get_redis_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>, CacheError> {
        match self.redis_pool.get().await {
            Ok(con) => Ok(con),
            Err(e) => {
                tracing::error!("Failed to get redis connection: {e:?}");
                Err(CacheError::RedisPool(e))
            }
        }
    }
}
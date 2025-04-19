use std::{fmt::Display, hash::Hash, sync::Arc};

use bb8_redis::{bb8::{Pool, RunError}, redis::{AsyncCommands, FromRedisValue, RedisError, ToRedisArgs}, RedisConnectionManager};
use metrics::describe_counter;
use moka::future::{Cache, CacheBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use metrics::counter;

use crate::expiry::{CacheExpiry, Expiration};

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("Redis pool error: {0}")]
    RedisPool(#[from] RunError<RedisError>),
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError)
}

pub struct HybridCache<K, V>
where 
    K: AsRef<str> + Display + ToRedisArgs + Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + FromRedisValue +Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    prefix: String,
    local_cache: Cache<K, (Expiration, V)>,
    redis_pool: Arc<Pool<RedisConnectionManager>>
}

impl<K, V> HybridCache<K, V> 
where 
K: AsRef<str> + Display + ToRedisArgs + Clone + Eq + Hash + Send + Sync + 'static,
V: Clone + FromRedisValue +Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
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

    pub async fn get(&self, key: &K, expiry: Expiration) -> Result<Option<V>, CacheError> {
        counter!("cache.requests.total", "layer" => "l1").increment(1);

        if let Some((_, v)) = self.local_cache.get(key).await {
            counter!("cache.hits.total", "layer" => "l1").increment(1);
            return Ok(Some(v))
        }

        counter!("cache.requests.total", "layer" => "l2").increment(1);

        let con = self.redis_pool
            .get()
            .await;

        if let Err(e) = con {
            tracing::error!("Failed to get redis connection: {:#?}", e);
            return Err(CacheError::RedisPool(e));
        }

        let result = con.unwrap()
            .get::<String, Option<V>>(
                format!("{}_{}", self.prefix, key)
            ).await;

        match result {
            Ok(v) => {
                if let Some(value) = v {
                    counter!("cache.hits.total", "layer" => "l2").increment(1);
                    self.local_cache.insert(key.clone(), (expiry, value.clone())).await;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            },
            Err(e) => {
                tracing::error!("Failed to get value from redis: {:#?}", e);
                Err(CacheError::Redis(e))
            }
        }
    }
}
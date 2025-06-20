use std::{fmt::Display, hash::Hash, time::Instant};

use bb8_redis::{bb8::{Pool, PooledConnection, RunError}, redis::{AsyncCommands, FromRedisValue, RedisError, ToRedisArgs}, RedisConnectionManager};
use metrics::{describe_counter, describe_histogram, histogram, counter};
use moka::future::{Cache, CacheBuilder};
use thiserror::Error;

use crate::{expiry::{CacheExpiry, Expiration}, serializer::CacheSerializer};

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Redis pool error: {0}")]
    RedisPool(#[from] RunError<RedisError>),
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError)
}

#[derive(Clone)]
pub struct HybridCache<K, V, S> {
    prefix: String,
    local_cache: Cache<K, (Expiration, V)>,
    redis_pool: Pool<RedisConnectionManager>,
    serializer: S,
}

impl<K, V, S> HybridCache<K, V, S> 
where 
    K: AsRef<str> + FromRedisValue + Display + ToRedisArgs + Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    S: CacheSerializer<V> + Send + Sync + 'static,
{
    pub fn new(
        prefix: String,
        redis_pool: Pool<RedisConnectionManager>,
        capacity: u64,
        serializer: S,
    ) -> Self {
        let local_cache = CacheBuilder::new(capacity)
            .expire_after(CacheExpiry)
            .build();

        describe_counter!("cache.requests.total", "Total cache requests");
        describe_counter!("cache.hits.total", "Cache hits");
        describe_histogram!("cache.operation.duration", "Cache operation duration in seconds");
        
        Self {
            prefix,
            local_cache,
            redis_pool,
            serializer,
        }
    }

    fn format_key(&self, key: &K) -> String {
        format!("{}_{}", self.prefix, key)
    }

    pub async fn get(&self, key: &K, expiry: Expiration) -> Result<Option<V>, CacheError> {
        let start_time = Instant::now();
        
        counter!("cache.requests.total", "layer" => "l1").increment(1);

        if let Some((_, v)) = self.local_cache.get(key).await {
            counter!("cache.hits.total", "layer" => "l1").increment(1);
            histogram!("cache.operation.duration", "operation" => "get", "layer" => "l1").record(start_time.elapsed().as_secs_f64());
            return Ok(Some(v))
        }

        counter!("cache.requests.total", "layer" => "l2").increment(1);

        let mut con = self.get_redis_connection().await?;

        let result = con
            .get::<String, Option<Vec<u8>>>(self.format_key(key))
            .await;

        let duration = start_time.elapsed().as_secs_f64();

        match result {
            Ok(Some(value)) => {
                counter!("cache.hits.total", "layer" => "l2").increment(1);
                let result = self.serializer.deserialize(&value);
                match result {
                    Ok(value) => {
                        self.local_cache.insert(key.clone(), (expiry, value.clone())).await;
                        histogram!("cache.operation.duration", "operation" => "get", "layer" => "l2").record(duration);
                        Ok(Some(value))
                    }
                    Err(e) => {
                        tracing::error!("Failed to deserialize value from Redis for key {}: {:#?}", key, e);
                        let _ = con.del::<_, ()>(self.format_key(key)).await; // Delete bad data
                        Err(e)
                    },
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
        let start_time = Instant::now();

        self.local_cache.insert(key.clone(), (expiry, value.clone())).await;
        
        let mut con = self.get_redis_connection().await?;

        let encoded = self.serializer.serialize(&value)?;
        con.set_ex::<_, _, ()>(self.format_key(&key), encoded, expiry.get_seconds()).await?;
        
        histogram!("cache.operation.duration", "operation" => "set").record(start_time.elapsed().as_secs_f64());

        Ok(())
    }

    pub async fn invalidate(&self, key: K) -> Result<(), CacheError> {
        let start_time = Instant::now();

        self.local_cache
            .invalidate(&key)
            .await;

        let mut con = self.get_redis_connection().await?;
        con.del::<_, ()>(self.format_key(&key)).await?;

        histogram!("cache.operation.duration", "operation" => "invalidate").record(start_time.elapsed().as_secs_f64());

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
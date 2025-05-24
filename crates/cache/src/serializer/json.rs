use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::cache::CacheError;

use super::CacheSerializer;

pub struct JsonSerializer<V> {
    _phantom: PhantomData<V>
}

impl<V> Default for JsonSerializer<V> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<V> CacheSerializer<V> for JsonSerializer<V>
where
    V: Serialize + for<'de> Deserialize<'de>,
{
    fn serialize(&self, value: &V) -> Result<Vec<u8>, CacheError> {
        serde_json::to_vec(value)
            .map_err(|e| CacheError::Serialization(e.to_string()))
    }

    fn deserialize(&self, data: &[u8]) -> Result<V, CacheError> {
        serde_json::from_slice(data)
            .map_err(|e| CacheError::Serialization(e.to_string()))
    }
}
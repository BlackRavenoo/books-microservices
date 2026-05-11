use std::marker::PhantomData;
use serde::{Serialize, Deserialize};
use crate::cache::CacheError;
use super::CacheSerializer;

#[derive(Clone)]
pub struct BitcodeSerializer<V> {
    _phantom: PhantomData<V>,
}

impl<V> Default for BitcodeSerializer<V> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<V> CacheSerializer<V> for BitcodeSerializer<V>
where
    V: Serialize + for<'de> Deserialize<'de>,
{
    fn serialize(&self, value: &V) -> Result<Vec<u8>, CacheError> {
        bitcode::serialize(value)
            .map_err(|e| CacheError::Serialization(e.to_string()))
    }

    fn deserialize(&self, data: &[u8]) -> Result<V, CacheError> {
        bitcode::deserialize(data)
            .map_err(|e| CacheError::Serialization(e.to_string()))
    }
}
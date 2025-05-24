use std::marker::PhantomData;

use bincode::{Decode, Encode};

use crate::cache::CacheError;

use super::CacheSerializer;

pub struct BincodeSerializer<V> {
    _phantom: PhantomData<V>,
}

impl<V> Default for BincodeSerializer<V> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<V> CacheSerializer<V> for BincodeSerializer<V>
where
    V: Encode + Decode<()>,
{
    fn serialize(&self, value: &V) -> Result<Vec<u8>, CacheError> {
        bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| CacheError::Serialization(e.to_string()))
    }

    fn deserialize(&self, data: &[u8]) -> Result<V, CacheError> {
        let (value, _) = bincode::decode_from_slice(data, bincode::config::standard())
            .map_err(|e| CacheError::Serialization(e.to_string()))?;
        Ok(value)
    }
}
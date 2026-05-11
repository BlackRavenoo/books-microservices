use crate::cache::CacheError;

pub mod json;
pub mod bitcode;

pub trait CacheSerializer<V> {
    fn serialize(&self, value: &V) -> Result<Vec<u8>, CacheError>;
    fn deserialize(&self, data: &[u8]) -> Result<V, CacheError>;
}
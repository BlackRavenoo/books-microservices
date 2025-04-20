use std::{time::{Duration, Instant}, hash::Hash};

use moka::Expiry;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Expiration {
    Never,
    Seconds(u16),
    Minutes(u8)
}

impl Expiration {
    pub fn as_duration(&self) -> Option<Duration> {
        match self {
            Expiration::Never => None,
            Expiration::Seconds(s) => Some(Duration::from_secs(*s as u64)),
            Expiration::Minutes(m) => Some(Duration::from_secs(60 * *m as u64))
        }
    }

    pub fn get_seconds(&self) -> u64 {
        match self {
            Expiration::Never => u64::MAX,
            Expiration::Seconds(s) => *s as u64,
            Expiration::Minutes(m) => *m as u64 * 60,
        }
    }
}

pub struct CacheExpiry;

impl<K, V> Expiry<K, (Expiration, V)> for CacheExpiry
where
    K: AsRef<str> + Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn expire_after_create(
        &self,
        _key: &K,
        value: &(Expiration, V),
        _current_time: Instant,
    ) -> Option<Duration> {
        value.0.as_duration()
    }
}
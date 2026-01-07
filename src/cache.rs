//! Caching layer for frequently accessed data
//!
//! Reduces API calls and improves performance for read-heavy workloads.

use crate::error::{SfError, SfResult};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Configuration for the cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries to store
    pub max_capacity: u64,

    /// Time-to-live for cache entries
    pub ttl: Duration,

    /// Time-to-idle for cache entries
    pub tti: Option<Duration>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            ttl: Duration::from_secs(300),      // 5 minutes
            tti: Some(Duration::from_secs(60)), // 1 minute idle
        }
    }
}

impl CacheConfig {
    /// Create a new cache config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum capacity
    pub fn max_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// Set time-to-live
    pub fn ttl(mut self, duration: Duration) -> Self {
        self.ttl = duration;
        self
    }

    /// Set time-to-idle
    pub fn tti(mut self, duration: Duration) -> Self {
        self.tti = Some(duration);
        self
    }

    /// Disable caching (for testing)
    pub fn disabled() -> Self {
        Self {
            max_capacity: 0,
            ttl: Duration::from_secs(0),
            tti: None,
        }
    }
}

/// Cache key for query results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct QueryKey {
    query: String,
}

impl QueryKey {
    fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

/// Cache key for individual records
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordKey {
    sobject: String,
    id: String,
}

impl Hash for RecordKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sobject.hash(state);
        self.id.hash(state);
    }
}

impl RecordKey {
    fn new(sobject: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            sobject: sobject.into(),
            id: id.into(),
        }
    }
}

/// Cached value wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedValue<T> {
    data: T,
    cached_at: i64, // Unix timestamp
}

impl<T> CachedValue<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            cached_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Query result cache
pub struct QueryCache {
    cache: Arc<Cache<QueryKey, Vec<u8>>>,
    enabled: bool,
}

impl QueryCache {
    /// Create a new query cache
    pub fn new(config: CacheConfig) -> Self {
        let enabled = config.max_capacity > 0 && config.ttl.as_secs() > 0;

        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .time_to_idle(config.tti.unwrap_or(config.ttl))
            .build();

        if enabled {
            info!(
                "Query cache enabled with capacity {} and TTL {:?}",
                config.max_capacity, config.ttl
            );
        } else {
            info!("Query cache disabled");
        }

        Self {
            cache: Arc::new(cache),
            enabled,
        }
    }

    /// Get cached query results
    pub async fn get<T>(&self, query: &str) -> Option<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled {
            return None;
        }

        let key = QueryKey::new(query);

        if let Some(cached_bytes) = self.cache.get(&key).await {
            match serde_json::from_slice::<CachedValue<Vec<T>>>(&cached_bytes) {
                Ok(cached_value) => {
                    debug!("Cache hit for query: {}", query);
                    Some(cached_value.data)
                }
                Err(e) => {
                    debug!("Cache deserialization error: {}", e);
                    None
                }
            }
        } else {
            debug!("Cache miss for query: {}", query);
            None
        }
    }

    /// Store query results in cache
    pub async fn set<T>(&self, query: &str, data: Vec<T>) -> SfResult<()>
    where
        T: Serialize,
    {
        if !self.enabled {
            return Ok(());
        }

        let key = QueryKey::new(query);
        let cached_value = CachedValue::new(data);

        match serde_json::to_vec(&cached_value) {
            Ok(bytes) => {
                self.cache.insert(key, bytes).await;
                debug!("Cached query results: {}", query);
                Ok(())
            }
            Err(e) => {
                debug!("Failed to serialize cache entry: {}", e);
                Err(SfError::Cache(format!("Serialization failed: {}", e)))
            }
        }
    }

    /// Invalidate cached query results
    pub async fn invalidate(&self, query: &str) {
        if !self.enabled {
            return;
        }

        let key = QueryKey::new(query);
        self.cache.invalidate(&key).await;
        debug!("Invalidated cache for query: {}", query);
    }

    /// Clear all cached queries
    pub async fn clear(&self) {
        if !self.enabled {
            return;
        }

        self.cache.invalidate_all();
        info!("Cleared all query cache entries");
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in cache
    pub entry_count: u64,

    /// Total weighted size of entries
    pub weighted_size: u64,
}

/// Record-level cache for individual SObject records
pub struct RecordCache {
    cache: Arc<Cache<RecordKey, Vec<u8>>>,
    enabled: bool,
}

impl RecordCache {
    /// Create a new record cache
    pub fn new(config: CacheConfig) -> Self {
        let enabled = config.max_capacity > 0 && config.ttl.as_secs() > 0;

        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .time_to_idle(config.tti.unwrap_or(config.ttl))
            .build();

        Self {
            cache: Arc::new(cache),
            enabled,
        }
    }

    /// Get cached record
    pub async fn get<T>(&self, sobject: &str, id: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled {
            return None;
        }

        let key = RecordKey::new(sobject, id);

        if let Some(cached_bytes) = self.cache.get(&key).await {
            match serde_json::from_slice::<CachedValue<T>>(&cached_bytes) {
                Ok(cached_value) => {
                    debug!("Cache hit for {} {}", sobject, id);
                    Some(cached_value.data)
                }
                Err(e) => {
                    debug!("Cache deserialization error: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Store record in cache
    pub async fn set<T>(&self, sobject: &str, id: &str, data: T) -> SfResult<()>
    where
        T: Serialize,
    {
        if !self.enabled {
            return Ok(());
        }

        let key = RecordKey::new(sobject, id);
        let cached_value = CachedValue::new(data);

        match serde_json::to_vec(&cached_value) {
            Ok(bytes) => {
                self.cache.insert(key, bytes).await;
                debug!("Cached {} {}", sobject, id);
                Ok(())
            }
            Err(e) => Err(SfError::Cache(format!("Serialization failed: {}", e))),
        }
    }

    /// Invalidate cached record
    pub async fn invalidate(&self, sobject: &str, id: &str) {
        if !self.enabled {
            return;
        }

        let key = RecordKey::new(sobject, id);
        self.cache.invalidate(&key).await;
        debug!("Invalidated cache for {} {}", sobject, id);
    }

    /// Invalidate all records of a given SObject type
    pub async fn invalidate_sobject(&self, sobject: &str) {
        if !self.enabled {
            return;
        }

        // Note: This is expensive - iterates all keys
        // Consider adding an index if this becomes a common operation
        let sobject_owned = sobject.to_string();
        self.cache
            .invalidate_entries_if(move |key, _| key.sobject == sobject_owned);
        info!("Invalidated all cached {} records", sobject);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_query_cache() {
        let config = CacheConfig::new().ttl(Duration::from_secs(60));
        let cache = QueryCache::new(config);

        let query = "SELECT Id FROM Account";
        let data = vec![TestRecord {
            id: "1".to_string(),
            name: "Test".to_string(),
        }];

        // Cache miss
        assert!(cache.get::<TestRecord>(query).await.is_none());

        // Store in cache
        cache.set(query, data.clone()).await.unwrap();

        // Cache hit
        let cached = cache.get::<TestRecord>(query).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), data);
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let config = CacheConfig::disabled();
        let cache = QueryCache::new(config);

        let query = "SELECT Id FROM Account";
        let data = vec![TestRecord {
            id: "1".to_string(),
            name: "Test".to_string(),
        }];

        cache.set(query, data).await.unwrap();

        // Should always return None when disabled
        assert!(cache.get::<TestRecord>(query).await.is_none());
    }
}

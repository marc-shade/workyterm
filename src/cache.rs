//! Response cache for repeated queries
//!
//! Simple file-based cache with TTL support to avoid redundant API calls.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cache entry with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub query: String,
    pub model: String,
    pub response: String,
    pub created_at: u64,
    pub ttl_secs: u64,
}

impl CacheEntry {
    /// Check if entry has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.created_at + self.ttl_secs
    }
}

/// Simple file-based response cache
pub struct ResponseCache {
    cache_dir: PathBuf,
    default_ttl: Duration,
    enabled: bool,
}

impl ResponseCache {
    /// Create a new cache instance
    pub fn new(enabled: bool, ttl_secs: u64) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("workyterm");

        Self {
            cache_dir,
            default_ttl: Duration::from_secs(ttl_secs),
            enabled,
        }
    }

    /// Initialize cache directory
    pub fn init(&self) -> Result<()> {
        if self.enabled {
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Generate cache key from query and model
    fn cache_key(&self, query: &str, model: &str) -> String {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        model.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get cache file path
    fn cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key))
    }

    /// Look up a cached response
    pub fn get(&self, query: &str, model: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let key = self.cache_key(query, model);
        let path = self.cache_path(&key);

        if !path.exists() {
            return None;
        }

        // Try to read and parse the cache entry
        let content = fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        // Check if expired
        if entry.is_expired() {
            // Clean up expired entry
            let _ = fs::remove_file(&path);
            return None;
        }

        Some(entry.response)
    }

    /// Store a response in cache
    pub fn set(&self, query: &str, model: &str, response: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let key = self.cache_key(query, model);
        let path = self.cache_path(&key);

        let entry = CacheEntry {
            query: query.to_string(),
            model: model.to_string(),
            response: response.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl_secs: self.default_ttl.as_secs(),
        };

        let content = serde_json::to_string_pretty(&entry)?;
        fs::write(&path, content)?;

        Ok(())
    }

    /// Clear all cached entries
    pub fn clear(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                fs::remove_file(&path)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Remove expired entries
    pub fn prune(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                        if cache_entry.is_expired() {
                            fs::remove_file(&path)?;
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();

        if !self.cache_dir.exists() {
            return stats;
        }

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    stats.total_entries += 1;
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                            stats.total_bytes += content.len();
                            if cache_entry.is_expired() {
                                stats.expired_entries += 1;
                            }
                        }
                    }
                }
            }
        }

        stats
    }

    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub total_bytes: usize,
}

impl CacheStats {
    pub fn active_entries(&self) -> usize {
        self.total_entries.saturating_sub(self.expired_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_cache_disabled() {
        let cache = ResponseCache::new(false, 3600);
        assert!(cache.get("test", "model").is_none());
        assert!(cache.set("test", "model", "response").is_ok());
        assert!(cache.get("test", "model").is_none()); // Still none, disabled
    }

    #[test]
    fn test_cache_key_generation() {
        let cache = ResponseCache::new(true, 3600);
        let key1 = cache.cache_key("hello", "gemini");
        let key2 = cache.cache_key("hello", "gemini");
        let key3 = cache.cache_key("world", "gemini");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_entry_expiry() {
        let entry = CacheEntry {
            query: "test".to_string(),
            model: "model".to_string(),
            response: "response".to_string(),
            created_at: 0, // Unix epoch - definitely expired
            ttl_secs: 1,
        };
        assert!(entry.is_expired());

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let fresh_entry = CacheEntry {
            query: "test".to_string(),
            model: "model".to_string(),
            response: "response".to_string(),
            created_at: now,
            ttl_secs: 3600,
        };
        assert!(!fresh_entry.is_expired());
    }
}

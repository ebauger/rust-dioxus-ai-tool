use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::tokenizer::TokenEstimator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub token_count: usize,
    pub mtime: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCache {
    entries: HashMap<PathBuf, CacheEntry>,
    estimator: TokenEstimator,
}

impl TokenCache {
    pub async fn new(estimator: TokenEstimator) -> std::io::Result<Self> {
        let cache_dir = dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rust-dioxus-ai-tool");

        fs::create_dir_all(&cache_dir).await?;
        let cache_file = cache_dir.join("token_cache.json");

        if let Ok(content) = fs::read_to_string(&cache_file).await {
            if let Ok(cache) = serde_json::from_str::<TokenCache>(&content) {
                if cache.estimator == estimator {
                    return Ok(cache);
                }
            }
        }

        Ok(TokenCache {
            entries: HashMap::new(),
            estimator,
        })
    }

    pub async fn save(&self) -> std::io::Result<()> {
        let cache_dir = dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rust-dioxus-ai-tool");
        let cache_file = cache_dir.join("token_cache.json");

        let content = serde_json::to_string_pretty(self)?;
        fs::write(cache_file, content).await?;
        Ok(())
    }

    pub fn get_entry(&self, path: &Path) -> Option<&CacheEntry> {
        self.entries.get(path)
    }

    pub fn insert_entry(&mut self, path: PathBuf, entry: CacheEntry) {
        self.entries.insert(path, entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cache_operations() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut cache = TokenCache::new(TokenEstimator::Cl100k).await.unwrap();
        assert!(cache.get_entry(&file_path).is_none());

        let entry = CacheEntry {
            token_count: 42,
            mtime: 123456789,
            hash: "test_hash".to_string(),
        };
        cache.insert_entry(file_path.clone(), entry);

        let retrieved = cache.get_entry(&file_path).unwrap();
        assert_eq!(retrieved.token_count, 42);
        assert_eq!(retrieved.mtime, 123456789);
        assert_eq!(retrieved.hash.as_str(), "test_hash");

        cache.clear();
        assert!(cache.get_entry(&file_path).is_none());
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
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
    #[serde(with = "path_map_serde")]
    entries: HashMap<PathBuf, CacheEntry>,
    estimator: TokenEstimator,
}

mod path_map_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::path::PathBuf;

    pub fn serialize<S>(
        map: &HashMap<PathBuf, super::CacheEntry>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, super::CacheEntry> = map
            .iter()
            .map(|(k, v)| (k.to_string_lossy().into_owned(), v.clone()))
            .collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<PathBuf, super::CacheEntry>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, super::CacheEntry> = HashMap::deserialize(deserializer)?;
        Ok(string_map
            .into_iter()
            .map(|(k, v)| (PathBuf::from(k), v))
            .collect())
    }
}

impl TokenCache {
    pub async fn new(estimator: TokenEstimator) -> std::io::Result<Self> {
        let dir = ensure_config_dir()?;
        let cache_file = dir.join("token_cache.json");

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
        let dir = ensure_config_dir()?;
        let cache_file = dir.join("token_cache.json");

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

fn ensure_config_dir() -> io::Result<PathBuf> {
    let path = dirs_next::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?
        .join("context-loader");
    std::fs::create_dir_all(&path)?;
    Ok(path)
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

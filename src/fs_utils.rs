use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;

use crate::cache::{CacheEntry, TokenCache};
use crate::tokenizer::{count_tokens, TokenEstimator};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub token_count: usize,
}

impl FileInfo {
    pub async fn new(path: PathBuf) -> std::io::Result<Self> {
        let metadata = fs::metadata(&path).await?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        Ok(FileInfo {
            name,
            size: metadata.len(),
            path,
            token_count: 0, // Will be computed later
        })
    }

    pub async fn with_tokens(mut self, estimator: TokenEstimator) -> std::io::Result<Self> {
        self.token_count = count_tokens(&self.path, estimator).await?;
        Ok(self)
    }
}

pub async fn read_children(dir: &PathBuf) -> Vec<FileInfo> {
    let mut files = Vec::new();

    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(path) = entry.path().canonicalize() {
                // Skip directories and hidden files
                if let Ok(metadata) = fs::metadata(&path).await {
                    if !metadata.is_dir() && !is_hidden(&path) {
                        if let Ok(file_info) = FileInfo::new(path).await {
                            files.push(file_info);
                        }
                    }
                }
            }
        }
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

pub async fn crawl_directory(dir: &PathBuf, estimator: TokenEstimator) -> Vec<FileInfo> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(dir).hidden(false).git_ignore(true).build();
    let semaphore = Arc::new(Semaphore::new(8)); // Limit to 8 concurrent tokenizations
    let mut cache = match TokenCache::new(estimator).await {
        Ok(cache) => cache,
        Err(_) => TokenCache::new(estimator).await.unwrap(),
    };
    let mut tasks = Vec::new();

    for result in walker {
        if let Ok(entry) = result {
            if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Ok(path) = entry.path().canonicalize() {
                    if !is_hidden(&path) {
                        if let Ok(metadata) = fs::metadata(&path).await {
                            let mtime = metadata
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                                .unwrap_or(0);

                            // Check cache first
                            if let Some(cache_entry) = cache.get_entry(&path) {
                                if cache_entry.mtime == mtime {
                                    if let Ok(mut file_info) = FileInfo::new(path.clone()).await {
                                        file_info.token_count = cache_entry.token_count;
                                        files.push(file_info);
                                        continue;
                                    }
                                }
                            }

                            // If not in cache or outdated, tokenize
                            let path_clone = path.clone();
                            let semaphore_clone = semaphore.clone();
                            let task = tokio::spawn(async move {
                                let _permit = semaphore_clone.acquire().await.unwrap();
                                if let Ok(file_info) = FileInfo::new(path_clone.clone()).await {
                                    if let Ok(file_info) = file_info.with_tokens(estimator).await {
                                        return Some((file_info, mtime, path_clone));
                                    }
                                }
                                None
                            });
                            tasks.push(task);
                        }
                    }
                }
            }
        }
    }

    // Wait for all tokenization tasks to complete
    for task in tasks {
        if let Ok(Some((file_info, mtime, path))) = task.await {
            // Update cache
            cache.insert_entry(
                path,
                CacheEntry {
                    token_count: file_info.token_count,
                    mtime,
                    hash: "".to_string(), // TODO: Implement file hashing if needed
                },
            );
            files.push(file_info);
        }
    }

    // Save updated cache
    let _ = cache.save().await;

    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

fn is_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.') || name == "target")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_info_new() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let file_info = FileInfo::new(file_path.clone()).await.unwrap();
        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, content.len() as u64);
        assert_eq!(file_info.path, file_path);
        assert_eq!(file_info.token_count, 0);
    }

    #[tokio::test]
    async fn test_file_info_with_tokens() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let file_info = FileInfo::new(file_path.clone()).await.unwrap();
        let file_info = file_info.with_tokens(TokenEstimator::Cl100k).await.unwrap();
        assert_eq!(file_info.token_count, 4); // "Hello", ",", " World", "!"
    }

    #[tokio::test]
    async fn test_crawl_directory() {
        let temp_dir = tempdir().unwrap();

        // Create a test directory structure
        let files = vec![
            ("root.txt", "Root file"),
            ("subdir/sub.txt", "Subdir file"),
            (".hidden.txt", "Hidden file"),
        ];

        for (path, content) in files {
            let full_path = temp_dir.path().join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut file = File::create(&full_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let files = crawl_directory(&temp_dir.path().to_path_buf(), TokenEstimator::Cl100k).await;

        // Should find both files but not the hidden one
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.name == "root.txt"));
        assert!(files.iter().any(|f| f.name == "sub.txt"));
        assert!(!files.iter().any(|f| f.name == ".hidden.txt"));
    }
}

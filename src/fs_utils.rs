use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::mpsc;

use crate::tokenizer::{count_tokens, TokenEstimator};

pub type ProgressCallback = Arc<Box<dyn Fn(usize, usize, String) + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressState {
    pub completed: usize,
    pub total: usize,
    pub message: String,
}

impl ProgressState {
    pub fn new() -> Self {
        Self {
            completed: 0,
            total: 0,
            message: String::new(),
        }
    }

    pub fn update(&mut self, completed: usize, total: usize, message: String) {
        self.completed = completed;
        self.total = total;
        self.message = message;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileInfo {
    pub name: String,
    #[serde(with = "path_serde")]
    pub path: PathBuf,
    pub size: u64,
    pub token_count: usize,
}

mod path_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::path::PathBuf;

    pub fn serialize<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        path.to_string_lossy().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(PathBuf::from(s))
    }
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

pub async fn crawl_directory(
    dir: &Path,
    estimator: TokenEstimator,
    progress_callback: Option<ProgressCallback>,
) -> Vec<FileInfo> {
    let mut files = Vec::new();
    let mut total_files = 0;
    let mut completed_files = 0;

    // First pass: count total files
    for entry in WalkBuilder::new(dir).hidden(false).git_ignore(true).build() {
        if let Ok(entry) = entry {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if !is_hidden(path) {
                    total_files += 1;
                }
            }
        }
    }

    if let Some(callback) = &progress_callback {
        callback(0, total_files, "Scanning files...".to_string());
    }

    // Second pass: process files
    for entry in WalkBuilder::new(dir).hidden(false).git_ignore(true).build() {
        if let Ok(entry) = entry {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if !is_hidden(path) {
                    let path = path.to_path_buf();
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    let metadata = fs::metadata(&path).await.unwrap();
                    let size = metadata.len();

                    let token_count = count_tokens(&path, estimator).await.unwrap_or(0);

                    files.push(FileInfo {
                        name,
                        path,
                        size,
                        token_count,
                    });

                    completed_files += 1;
                    if let Some(callback) = &progress_callback {
                        callback(
                            completed_files,
                            total_files,
                            "Processing files...".to_string(),
                        );
                    }
                }
            }
        }
    }

    files
}

async fn count_files(dir: &PathBuf) -> usize {
    let mut count = 0;
    let mut entries = tokio::fs::read_dir(dir).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        if entry.path().is_file() {
            count += 1;
        }
    }
    count
}

fn is_hidden(path: &Path) -> bool {
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
        // Create temp dir with explicit cleanup
        let temp_dir = {
            let dir = tempdir().expect("Failed to create temp dir");
            println!("Created temp dir at: {:?}", dir.path());
            dir
        };

        // Create a test directory structure
        let files = vec![
            ("root.txt", "Root file"),
            ("subdir/sub.txt", "Subdir file"),
            (".hidden.txt", "Hidden file"),
        ];

        for (path, content) in files {
            let full_path = temp_dir.path().join(path);
            println!("Creating file: {:?}", full_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut file = File::create(&full_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        println!("Starting directory crawl...");
        let start_time = std::time::Instant::now();
        let files = crawl_directory(temp_dir.path(), TokenEstimator::Cl100k, None).await;
        let duration = start_time.elapsed();
        println!(
            "Crawl completed in {:?}, found {} files",
            duration,
            files.len()
        );

        assert_eq!(files.len(), 2); // Should not include hidden file
        assert!(files.iter().any(|f| f.name == "root.txt"));
        assert!(files.iter().any(|f| f.name == "sub.txt"));

        // Explicitly drop temp_dir to ensure cleanup
        drop(temp_dir);
    }
}

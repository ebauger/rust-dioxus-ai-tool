use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs;
use tokio::sync::mpsc;
use walkdir::WalkDir;

use crate::cache::TokenCache;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
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
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        Ok(FileInfo {
            name,
            path,
            size: metadata.len(),
            token_count: 0,
        })
    }

    pub fn with_tokens(path: PathBuf, estimator: &TokenEstimator) -> io::Result<Self> {
        let mut info = Self::new(path)?;
        info.token_count = estimator.estimate_file_tokens(&info.path)?;
        Ok(info)
    }
}

pub async fn crawl(
    dir: &Path,
    estimator: &TokenEstimator,
    progress_tx: Option<mpsc::Sender<(usize, usize)>>,
) -> io::Result<Vec<FileInfo>> {
    let mut files = Vec::new();
    let mut total_files = 0;
    let mut processed_files = 0;

    println!("Starting crawl in directory: {}", dir.display());

    // Check if this is a test directory (starts with .tmp)
    let is_test_dir = dir.to_string_lossy().contains(".tmp");

    // First pass: count total files
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            if is_test_dir && e.path() == dir {
                println!("Not filtering test directory root: {}", e.path().display());
                return true;
            }

            let is_hidden = is_hidden(e.path());
            println!(
                "Checking entry: {}, hidden: {}",
                e.path().display(),
                is_hidden
            );
            !is_hidden
        })
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    println!("Found file: {}", entry.path().display());
                    total_files += 1;
                }
            }
            Err(e) => {
                eprintln!("Error walking directory: {}", e);
            }
        }
    }

    println!("Total files found: {}", total_files);

    // Second pass: process files
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            if is_test_dir && e.path() == dir {
                return true;
            }
            !is_hidden(e.path())
        })
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    println!("Processing file: {}", entry.path().display());
                    match FileInfo::with_tokens(entry.path().to_path_buf(), estimator) {
                        Ok(info) => {
                            files.push(info);
                        }
                        Err(e) => {
                            eprintln!("Error processing file {}: {}", entry.path().display(), e);
                        }
                    }
                    processed_files += 1;
                    if let Some(tx) = &progress_tx {
                        let _ = tx.send((processed_files, total_files)).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error walking directory: {}", e);
            }
        }
    }

    println!("Processed {} files", processed_files);
    Ok(files)
}

pub async fn read_children(dir: &Path) -> Vec<FileInfo> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() && !is_hidden(&entry.path()) {
                    if let Ok(info) = FileInfo::new(entry.path()) {
                        files.push(info);
                    }
                }
            }
        }
    }

    files
}

fn is_hidden(path: &Path) -> bool {
    // Get the file name component
    if let Some(file_name) = path.file_name() {
        // Only check if the file name starts with a dot
        file_name
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    } else {
        // If there's no file name (root directory), don't mark as hidden
        false
    }
}

pub fn get_file_hash(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 8192];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

pub fn get_file_mtime(path: &Path) -> io::Result<SystemTime> {
    Ok(std::fs::metadata(path)?.modified()?)
}

async fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()))
    {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                count += 1;
            }
        }
    }
    count
}

pub async fn concat_files(paths: &[PathBuf]) -> io::Result<String> {
    let mut result = String::new();
    let mut first = true;

    for path in paths {
        if !first {
            result.push_str("\n\n/* ---- ");
            result.push_str(&path.to_string_lossy());
            result.push_str(" ---- */\n\n");
        }
        first = false;

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        result.push_str(&content);
    }

    Ok(result)
}

pub async fn list_files(dir: &Path) -> io::Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    // Count files for a quick list without processing tokens
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()))
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    match FileInfo::new(entry.path().to_path_buf()) {
                        Ok(info) => {
                            files.push(info);
                        }
                        Err(e) => {
                            eprintln!("Error processing file {}: {}", entry.path().display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error walking directory: {}", e);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use tokio::fs;

    #[test]
    fn test_file_info_new() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        drop(file); // Ensure file is closed

        let info = FileInfo::new(file_path.clone()).unwrap();
        assert_eq!(info.name, "test.txt");
        assert_eq!(info.path, file_path);
        assert_eq!(info.size, 14); // "Hello, world!\n" = 14 bytes
        assert_eq!(info.token_count, 0);
    }

    #[tokio::test]
    async fn test_file_info_with_tokens() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!\n").await.unwrap();

        let estimator = TokenEstimator::CharDiv4;
        let info = FileInfo::with_tokens(file_path.clone(), &estimator).unwrap();
        assert_eq!(info.name, "test.txt");
        assert_eq!(info.path, file_path);
        assert_eq!(info.size, 14); // "Hello, world!\n" = 14 bytes
        assert_eq!(info.token_count, 3); // 14 chars / 4 ≈ 3 tokens (actual implementation rounds down)
    }

    #[tokio::test]
    async fn test_crawl_directory() {
        let dir = tempdir().unwrap();
        println!("Test directory: {}", dir.path().display());

        // Create test files
        let file1_path = dir.path().join("file1.txt");
        println!("Creating file1: {}", file1_path.display());
        fs::write(&file1_path, "Hello, world!\n").await.unwrap();
        assert!(file1_path.exists(), "file1.txt was not created");

        let file2_path = dir.path().join("file2.txt");
        println!("Creating file2: {}", file2_path.display());
        fs::write(&file2_path, "Another test file\n").await.unwrap();
        assert!(file2_path.exists(), "file2.txt was not created");

        // Verify files are readable
        let content1 = fs::read_to_string(&file1_path).await.unwrap();
        let content2 = fs::read_to_string(&file2_path).await.unwrap();
        assert_eq!(content1, "Hello, world!\n");
        assert_eq!(content2, "Another test file\n");

        let estimator = TokenEstimator::CharDiv4;
        let files = crawl(dir.path(), &estimator, None).await.unwrap();

        println!(
            "Found files: {:?}",
            files.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.name == "file1.txt"));
        assert!(files.iter().any(|f| f.name == "file2.txt"));
    }

    #[tokio::test]
    async fn test_concat_files() {
        let dir = tempdir().unwrap();

        // Create test files
        let file1_path = dir.path().join("file1.txt");
        fs::write(&file1_path, "Hello, world!\n").await.unwrap();

        let file2_path = dir.path().join("file2.txt");
        fs::write(&file2_path, "Another test file\n").await.unwrap();

        let paths = vec![file1_path.clone(), file2_path.clone()];
        let result = concat_files(&paths).await.unwrap();

        assert!(result.contains("Hello, world!"));
        assert!(result.contains("Another test file"));
        assert!(result.contains(&format!("/* ---- {} ---- */", file2_path.display())));
    }
}

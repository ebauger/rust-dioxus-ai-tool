use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

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
            token_count: 0, // Will be computed later in Story 4
        })
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
        let content = b"Hello, World!";

        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content).unwrap();

        let file_info = FileInfo::new(file_path.clone()).await.unwrap();
        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, content.len() as u64);
        assert_eq!(file_info.path, file_path);
        assert_eq!(file_info.token_count, 0);
    }

    #[tokio::test]
    async fn test_read_children() {
        let temp_dir = tempdir().unwrap();

        // Create some test files
        let files = vec![
            ("visible.txt", "Hello"),
            ("another.txt", "World"),
            (".hidden.txt", "Hidden"),
        ];

        for (name, content) in files {
            let path = temp_dir.path().join(name);
            let mut file = File::create(path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        // Create a subdirectory (should be ignored)
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let children = read_children(&temp_dir.path().to_path_buf()).await;

        // Should only see non-hidden files
        assert_eq!(children.len(), 2);
        assert!(children.iter().any(|f| f.name == "visible.txt"));
        assert!(children.iter().any(|f| f.name == "another.txt"));
        assert!(!children.iter().any(|f| f.name == ".hidden.txt"));

        // Should be sorted by name
        assert_eq!(children[0].name, "another.txt");
        assert_eq!(children[1].name, "visible.txt");
    }

    #[test]
    fn test_is_hidden() {
        let hidden = PathBuf::from("/path/to/.hidden");
        let visible = PathBuf::from("/path/to/visible");
        let target = PathBuf::from("/path/to/target");

        assert!(is_hidden(&hidden));
        assert!(!is_hidden(&visible));
        assert!(is_hidden(&target));
    }
}

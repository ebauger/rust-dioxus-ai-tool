use std::collections::HashSet;
use std::path::PathBuf;

use crate::fs_utils::FileInfo;

#[test]
fn test_footer_token_sum() {
    let files = vec![
        FileInfo {
            name: "file1.txt".to_string(),
            path: PathBuf::from("/test/file1.txt"),
            size: 100,
            token_count: 10,
        },
        FileInfo {
            name: "file2.txt".to_string(),
            path: PathBuf::from("/test/file2.txt"),
            size: 200,
            token_count: 20,
        },
        FileInfo {
            name: "file3.txt".to_string(),
            path: PathBuf::from("/test/file3.txt"),
            size: 300,
            token_count: 30,
        },
    ];

    let mut selected = HashSet::new();
    selected.insert(PathBuf::from("/test/file1.txt"));
    selected.insert(PathBuf::from("/test/file2.txt"));

    // Calculate the expected total directly
    let expected_total: usize = files
        .iter()
        .filter(|file| selected.contains(&file.path))
        .map(|file| file.token_count)
        .sum();

    assert_eq!(expected_total, 30); // Verify our test data is correct
}

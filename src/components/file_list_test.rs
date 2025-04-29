#![cfg(test)]

use crate::components::file_list::{SortColumn, SortDirection};
use crate::fs_utils::FileInfo;
use std::path::PathBuf;

#[test]
fn test_sorting_by_size() {
    let mut files = vec![
        FileInfo {
            name: "small.txt".to_string(),
            path: PathBuf::from("small.txt"),
            size: 100,
            token_count: 10,
        },
        FileInfo {
            name: "medium.txt".to_string(),
            path: PathBuf::from("medium.txt"),
            size: 1000,
            token_count: 100,
        },
        FileInfo {
            name: "large.txt".to_string(),
            path: PathBuf::from("large.txt"),
            size: 10000,
            token_count: 1000,
        },
    ];

    // Test ascending sort
    files.sort_by_cached_key(|file| file.size.to_string());
    assert_eq!(files[0].name, "small.txt");
    assert_eq!(files[1].name, "medium.txt");
    assert_eq!(files[2].name, "large.txt");

    // Test descending sort
    files.reverse();
    assert_eq!(files[0].name, "large.txt");
    assert_eq!(files[1].name, "medium.txt");
    assert_eq!(files[2].name, "small.txt");
}

#[test]
fn test_sorting_by_name() {
    let mut files = vec![
        FileInfo {
            name: "c.txt".to_string(),
            path: PathBuf::from("c.txt"),
            size: 100,
            token_count: 10,
        },
        FileInfo {
            name: "a.txt".to_string(),
            path: PathBuf::from("a.txt"),
            size: 1000,
            token_count: 100,
        },
        FileInfo {
            name: "b.txt".to_string(),
            path: PathBuf::from("b.txt"),
            size: 10000,
            token_count: 1000,
        },
    ];

    // Test ascending sort
    files.sort_by_cached_key(|file| file.name.clone());
    assert_eq!(files[0].name, "a.txt");
    assert_eq!(files[1].name, "b.txt");
    assert_eq!(files[2].name, "c.txt");

    // Test descending sort
    files.reverse();
    assert_eq!(files[0].name, "c.txt");
    assert_eq!(files[1].name, "b.txt");
    assert_eq!(files[2].name, "a.txt");
}

#[test]
fn test_sorting_by_tokens() {
    let mut files = vec![
        FileInfo {
            name: "few.txt".to_string(),
            path: PathBuf::from("few.txt"),
            size: 100,
            token_count: 10,
        },
        FileInfo {
            name: "some.txt".to_string(),
            path: PathBuf::from("some.txt"),
            size: 1000,
            token_count: 100,
        },
        FileInfo {
            name: "many.txt".to_string(),
            path: PathBuf::from("many.txt"),
            size: 10000,
            token_count: 1000,
        },
    ];

    // Test ascending sort
    files.sort_by_cached_key(|file| file.token_count.to_string());
    assert_eq!(files[0].name, "few.txt");
    assert_eq!(files[1].name, "some.txt");
    assert_eq!(files[2].name, "many.txt");

    // Test descending sort
    files.reverse();
    assert_eq!(files[0].name, "many.txt");
    assert_eq!(files[1].name, "some.txt");
    assert_eq!(files[2].name, "few.txt");
}

#![allow(non_snake_case)]

use dioxus::desktop::Config;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::fs_utils::FileInfo;
use crate::tokenizer::TokenEstimator;

#[derive(Props, Clone, PartialEq)]
pub struct FooterProps {
    files: Vec<FileInfo>,
    selected_files: Signal<HashSet<PathBuf>>,
    current_estimator: TokenEstimator,
}

#[component]
pub fn Footer(props: FooterProps) -> Element {
    let FooterProps {
        files,
        selected_files,
        current_estimator,
    } = props;

    // Calculate total tokens for selected files
    let total_tokens = use_memo(move || {
        let selected = selected_files.read();
        files
            .iter()
            .filter(|file| selected.contains(&file.path))
            .map(|file| file.token_count)
            .sum::<usize>()
    });

    let total = *total_tokens.read();
    let is_over_limit = total > 32_000;

    rsx! {
        div {
            class: "fixed bottom-0 left-0 right-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-4",
            div {
                class: "flex justify-between items-center max-w-7xl mx-auto",
                div {
                    class: "flex items-center space-x-2",
                    span {
                        class: if is_over_limit { "text-red-500 font-medium" } else { "text-gray-700 dark:text-gray-300 font-medium" },
                        "Total tokens: {total}"
                    }
                    if is_over_limit {
                        span {
                            class: "text-red-500",
                            title: "Token count exceeds 32k limit",
                            // Warning icon from Heroicons
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                class: "h-5 w-5",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path {
                                    d: "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "text-sm text-gray-500 dark:text-gray-400",
                    "Estimation via {current_estimator.name()}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::PathBuf;

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
}

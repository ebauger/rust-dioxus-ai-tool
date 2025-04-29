#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::fs_utils::FileInfo;

#[derive(Props, Clone, PartialEq)]
pub struct FileListProps {
    files: Vec<FileInfo>,
    selected_files: Signal<HashSet<PathBuf>>,
}

#[component]
pub fn FileList(props: FileListProps) -> Element {
    let FileListProps {
        files,
        mut selected_files,
    } = props;

    let mut toggle_selected = move |path: PathBuf| {
        let mut new_selection = selected_files.read().clone();
        if new_selection.contains(&path) {
            new_selection.remove(&path);
        } else {
            new_selection.insert(path);
        }
        selected_files.set(new_selection);
    };

    rsx! {
        div {
            class: "flex flex-col space-y-2",
            if files.is_empty() {
                div {
                    class: "text-gray-500 dark:text-gray-400 text-center py-4",
                    "No files loaded yet",
                }
            } else {
                ul {
                    class: "divide-y divide-gray-200 dark:divide-gray-700",
                    {files.iter().map(|file| {
                        let path = file.path.clone();
                        rsx! {
                            li {
                                key: "{file.path.to_string_lossy()}",
                                class: "flex items-center space-x-4 py-2 px-4 hover:bg-gray-50 dark:hover:bg-gray-800",
                                div {
                                    class: "flex items-center",
                                    input {
                                        r#type: "checkbox",
                                        class: "h-4 w-4 text-blue-600",
                                        checked: selected_files.read().contains(&file.path),
                                        onclick: move |_| {
                                            let path = path.clone();
                                            toggle_selected(path);
                                        },
                                    }
                                },
                                div {
                                    class: "flex-1 min-w-0",
                                    span {
                                        class: "text-sm font-medium text-gray-900 dark:text-gray-100",
                                        "{file.name}",
                                    }
                                },
                                div {
                                    class: "flex items-center space-x-4",
                                    span {
                                        class: "text-sm text-gray-500 dark:text-gray-400",
                                        "{format_size(file.size)}",
                                    },
                                    span {
                                        class: "text-sm text-gray-500 dark:text-gray-400",
                                        "{file.token_count} tokens",
                                    }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else {
        format!("{:.1} GB", size as f64 / GB as f64)
    }
}

#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq)]
pub struct FileListProps {
    files: Vec<PathBuf>,
}

#[component]
pub fn FileList(props: FileListProps) -> Element {
    if props.files.is_empty() {
        return rsx! {
            p {
                class: "text-gray-500 text-center py-4",
                "No files found"
            }
        };
    }

    rsx! {
        div {
            class: "space-y-2",
            for file in props.files.iter() {
                div {
                    class: "flex items-center justify-between py-2 px-4 hover:bg-gray-50 rounded-lg",
                    div {
                        class: "flex items-center space-x-2",
                        // File icon placeholder
                        div {
                            class: "w-5 h-5 bg-gray-200 rounded"
                        }
                        span {
                            class: "text-gray-800",
                            "{file.file_name().unwrap_or_default().to_string_lossy()}"
                        }
                    }
                    span {
                        class: "text-gray-500 text-sm",
                        "{human_readable_size(file.metadata().map(|m| m.len()).unwrap_or(0))}"
                    }
                }
            }
        }
    }
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

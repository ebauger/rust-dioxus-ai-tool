#![allow(non_snake_case)]

use crate::fs_utils::FileInfo;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FileListProps {
    files: Vec<FileInfo>,
}

#[component]
pub fn FileList(props: FileListProps) -> Element {
    let FileListProps { files } = props;

    rsx! {
        div {
            class: "flex-1 overflow-auto p-4",
            if files.is_empty() {
                div {
                    class: "text-gray-500 text-center mt-8",
                    "No files loaded yet. Select a folder to begin.",
                }
            } else {
                ul {
                    class: "space-y-2",
                    for file in files {
                        li {
                            class: "flex items-center justify-between p-2 hover:bg-gray-100 rounded",
                            div {
                                class: "flex items-center",
                                span {
                                    class: "text-gray-900",
                                    "{file.name}",
                                }
                            }
                            div {
                                class: "text-gray-500 text-sm",
                                "{format_size(file.size)}",
                            }
                        }
                    }
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

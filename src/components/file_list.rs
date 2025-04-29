#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::fs_utils::FileInfo;

#[derive(Clone, Copy, PartialEq)]
pub enum SortColumn {
    Name,
    Size,
    Tokens,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Props, Clone, PartialEq)]
pub struct FileListProps {
    files: Vec<FileInfo>,
    selected_files: Signal<HashSet<PathBuf>>,
    on_select_all: EventHandler<()>,
    on_deselect_all: EventHandler<()>,
}

#[component]
pub fn FileList(props: FileListProps) -> Element {
    let FileListProps {
        files,
        selected_files,
        on_select_all,
        on_deselect_all,
    } = props;

    let mut sort_state = use_signal(|| (SortColumn::Name, SortDirection::Ascending));
    let (sort_column, sort_direction) = *sort_state.read();

    // Create separate clones for each usage
    let mut sorted_files = files.clone();
    sorted_files.sort_by_cached_key(|file| match sort_column {
        SortColumn::Name => file.name.clone(),
        SortColumn::Size => file.size.to_string(),
        SortColumn::Tokens => file.token_count.to_string(),
    });

    if sort_direction == SortDirection::Descending {
        sorted_files.reverse();
    }

    let sorted_files = sorted_files; // Make immutable after sorting

    let mut selected_files_for_ui = selected_files.clone();

    let mut toggle_selected = move |path: PathBuf| {
        let mut new_selection = selected_files_for_ui.read().clone();
        if new_selection.contains(&path) {
            new_selection.remove(&path);
        } else {
            new_selection.insert(path);
        }
        selected_files_for_ui.set(new_selection);
    };

    let mut toggle_sort = move |column: SortColumn| {
        let (current_column, current_direction) = *sort_state.read();
        if current_column == column {
            // Toggle direction if clicking same column
            sort_state.set((
                column,
                match current_direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                },
            ));
        } else {
            // Switch to new column, default to ascending
            sort_state.set((column, SortDirection::Ascending));
        }
    };

    // Add keyboard shortcuts for individual file selection
    let sorted_files_for_shortcuts = sorted_files.clone();
    use_effect(move || {
        // First keyboard shortcut
        let mut selected_files_up = selected_files.clone();
        let files_up = sorted_files_for_shortcuts.clone();

        let _ = dioxus::desktop::use_global_shortcut("Shift+ArrowUp", move || {
            let current_selection = selected_files_up.read().clone();
            if let Some(current) = current_selection.iter().next() {
                if let Some(pos) = files_up.iter().position(|f| &f.path == current) {
                    if pos > 0 {
                        let path = files_up[pos - 1].path.clone();
                        let mut new_selection = current_selection;
                        new_selection.insert(path);
                        selected_files_up.set(new_selection);
                    }
                }
            }
        });

        // Second keyboard shortcut
        let mut selected_files_down = selected_files.clone();
        let files_down = sorted_files_for_shortcuts.clone();

        let _ = dioxus::desktop::use_global_shortcut("Shift+ArrowDown", move || {
            let current_selection = selected_files_down.read().clone();
            if let Some(current) = current_selection.iter().next() {
                if let Some(pos) = files_down.iter().position(|f| &f.path == current) {
                    if pos < files_down.len() - 1 {
                        let path = files_down[pos + 1].path.clone();
                        let mut new_selection = current_selection;
                        new_selection.insert(path);
                        selected_files_down.set(new_selection);
                    }
                }
            }
        });
    });

    rsx! {
        div {
            class: "flex flex-col space-y-2",
            div {
                class: "flex justify-end space-x-2",
                button {
                    class: "px-3 py-1 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700",
                    onclick: move |_| on_select_all.call(()),
                    "Select All"
                }
                button {
                    class: "px-3 py-1 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700",
                    onclick: move |_| on_deselect_all.call(()),
                    "Deselect All"
                }
            }
            if sorted_files.is_empty() {
                div {
                    class: "text-gray-500 dark:text-gray-400 text-center py-4",
                    "No files loaded yet",
                }
            } else {
                div {
                    class: "overflow-x-auto",
                    table {
                        class: "min-w-full divide-y divide-gray-200 dark:divide-gray-700",
                        thead {
                            class: "bg-gray-50 dark:bg-gray-800",
                            tr {
                                th {
                                    class: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer",
                                    onclick: move |_| toggle_sort(SortColumn::Name),
                                    "Name",
                                    if sort_column == SortColumn::Name {
                                        span {
                                            class: "ml-1",
                                            if sort_direction == SortDirection::Ascending {
                                                "↑"
                                            } else {
                                                "↓"
                                            }
                                        }
                                    }
                                },
                                th {
                                    class: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer",
                                    onclick: move |_| toggle_sort(SortColumn::Size),
                                    "Size",
                                    if sort_column == SortColumn::Size {
                                        span {
                                            class: "ml-1",
                                            if sort_direction == SortDirection::Ascending {
                                                "↑"
                                            } else {
                                                "↓"
                                            }
                                        }
                                    }
                                },
                                th {
                                    class: "px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer",
                                    onclick: move |_| toggle_sort(SortColumn::Tokens),
                                    "Tokens",
                                    if sort_column == SortColumn::Tokens {
                                        span {
                                            class: "ml-1",
                                            if sort_direction == SortDirection::Ascending {
                                                "↑"
                                            } else {
                                                "↓"
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        tbody {
                            class: "bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700",
                            {sorted_files.iter().map(|file| {
                                let path = file.path.clone();
                                rsx! {
                                    tr {
                                        key: "{file.path.to_string_lossy()}",
                                        class: "hover:bg-gray-50 dark:hover:bg-gray-800",
                                        td {
                                            class: "px-4 py-2 whitespace-nowrap",
                                            div {
                                                class: "flex items-center",
                                                input {
                                                    r#type: "checkbox",
                                                    class: "h-4 w-4 text-blue-600",
                                                    checked: selected_files_for_ui.read().contains(&file.path),
                                                    onclick: move |_| {
                                                        let path = path.clone();
                                                        toggle_selected(path);
                                                    },
                                                }
                                                span {
                                                    class: "ml-2 text-sm font-medium text-gray-900 dark:text-gray-100",
                                                    "{file.name}",
                                                }
                                            }
                                        },
                                        td {
                                            class: "px-4 py-2 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400",
                                            "{format_size(file.size)}",
                                        },
                                        td {
                                            class: "px-4 py-2 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400",
                                            "{file.token_count} tokens",
                                        }
                                    }
                                }
                            })}
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

use crate::fs_utils::concat_files;
use arboard::Clipboard;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::error;

#[derive(Props, Clone, PartialEq)]
pub struct CopyButtonProps {
    pub selected_files: Signal<HashSet<PathBuf>>,
    pub on_copy: EventHandler<Result<(), String>>,
}

#[component]
pub fn CopyButton(props: CopyButtonProps) -> Element {
    let CopyButtonProps {
        selected_files,
        on_copy,
    } = props;

    let mut is_copying = use_signal(|| false);

    let mut handle_copy = move |_| {
        let selected_files = selected_files.read().clone();
        if selected_files.is_empty() {
            return;
        }

        is_copying.set(true);
        let mut clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(e) => {
                error!("Failed to initialize clipboard: {}", e);
                on_copy.call(Err("Failed to initialize clipboard".to_string()));
                is_copying.set(false);
                return;
            }
        };

        let paths: Vec<PathBuf> = selected_files.iter().cloned().collect();
        let result = tokio::task::spawn_blocking(move || match concat_files(&paths) {
            Ok(content) => match clipboard.set_text(content) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Failed to copy to clipboard: {}", e);
                    Err("Failed to copy to clipboard".to_string())
                }
            },
            Err(e) => {
                error!("Failed to concatenate files: {}", e);
                Err("Failed to concatenate files".to_string())
            }
        });

        // Handle the async result
        spawn(async move {
            match result.await {
                Ok(result) => on_copy.call(result),
                Err(e) => {
                    error!("Failed to join copy task: {}", e);
                    on_copy.call(Err("Failed to join copy task".to_string()));
                }
            }
            is_copying.set(false);
        });
    };

    rsx! {
        button {
            class: "px-3 py-1 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed",
            disabled: selected_files.read().is_empty() || *is_copying.read(),
            onclick: handle_copy,
            if *is_copying.read() {
                "Copying..."
            } else {
                "Copy Selected Files"
            }
        }
    }
}

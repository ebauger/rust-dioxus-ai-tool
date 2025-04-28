#![allow(non_snake_case)]

use crate::settings::Settings;
use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    on_workspace_select: EventHandler<PathBuf>,
    on_select_all: EventHandler<()>,
    on_deselect_all: EventHandler<()>,
    has_files: bool,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut settings = use_signal(|| Settings::load());
    let recent_workspaces = settings.read().get_recent_workspaces().to_vec();

    let ToolbarProps {
        on_workspace_select,
        on_select_all,
        on_deselect_all,
        has_files,
    } = props;

    rsx! {
        div {
            class: "flex items-center justify-between p-4 bg-gray-100 border-b",
            div {
                class: "flex items-center space-x-2",
                button {
                    class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                    onclick: move |_| {
                        spawn(async move {
                            if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                                on_workspace_select.call(folder.path().to_path_buf());
                            }
                        });
                    },
                    "Open Folder..."
                }
            }
            div {
                class: "flex items-center space-x-2",
                button {
                    class: "px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 disabled:opacity-50",
                    disabled: !has_files,
                    onclick: move |_| on_select_all.call(()),
                    "Select All"
                }
                button {
                    class: "px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 disabled:opacity-50",
                    disabled: !has_files,
                    onclick: move |_| on_deselect_all.call(()),
                    "Deselect All"
                }
            }
        }
    }
}

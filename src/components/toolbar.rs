#![allow(non_snake_case)]

use crate::settings::Settings;
use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    on_workspace_select: EventHandler<PathBuf>,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut settings = use_signal(|| Settings::load());
    let recent_workspaces = settings.read().get_recent_workspaces().to_vec();

    rsx! {
        div {
            class: "flex items-center p-2 bg-gray-100",

            button {
                class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                onclick: move |_| {
                    spawn(async move {
                        if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
                            let path = folder.path().to_path_buf();
                            props.on_workspace_select.call(path.clone());
                            settings.write().add_recent_workspace(path);
                            let _ = settings.write().save();
                        }
                    });
                },
                "Open Folder..."
            }

            if !recent_workspaces.is_empty() {
                div {
                    class: "ml-4",
                    "Recent:"
                    for path in recent_workspaces {
                        button {
                            class: "ml-2 px-2 py-1 text-sm text-blue-600 hover:text-blue-800",
                            onclick: move |_| {
                                props.on_workspace_select.call(path.clone());
                            },
                            "{path.display()}"
                        }
                    }
                }
            }
        }
    }
}

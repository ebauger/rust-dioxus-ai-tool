#![allow(non_snake_case)]

use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

use crate::settings::Settings;
use crate::tokenizer::TokenEstimator;

#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    on_workspace_select: EventHandler<PathBuf>,
    on_select_all: EventHandler<()>,
    on_deselect_all: EventHandler<()>,
    on_estimator_change: EventHandler<TokenEstimator>,
    has_files: bool,
    current_estimator: TokenEstimator,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let ToolbarProps {
        on_workspace_select,
        on_select_all,
        on_deselect_all,
        on_estimator_change,
        has_files,
        current_estimator,
    } = props;

    let config_dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rust-dioxus-ai-tool");
    let settings_file = config_dir.join("settings.json");
    let mut settings = use_signal(|| Settings::new(settings_file));

    // Load settings on mount
    use_effect(move || {
        let settings_file = settings.read().config_path.clone().unwrap();
        spawn(async move {
            if let Ok(loaded_settings) = Settings::load(&settings_file).await {
                settings.set(loaded_settings);
            }
        });
    });

    let on_open_folder = move |_| {
        let on_workspace_select = on_workspace_select.clone();
        let mut settings = settings.clone();
        spawn(async move {
            if let Some(handle) = AsyncFileDialog::new().pick_folder().await {
                let path = handle.path().to_path_buf();
                on_workspace_select.call(path.clone());
                let mut current_settings = settings.read().clone();
                current_settings.add_recent_workspace(path);
                if let Err(e) = current_settings.save().await {
                    log::error!("Failed to save settings: {}", e);
                }
                settings.set(current_settings);
            }
        });
    };

    let on_recent_select = move |path: PathBuf| {
        let on_workspace_select = on_workspace_select.clone();
        let mut settings = settings.clone();
        spawn(async move {
            on_workspace_select.call(path.clone());
            let mut current_settings = settings.read().clone();
            current_settings.add_recent_workspace(path);
            if let Err(e) = current_settings.save().await {
                log::error!("Failed to save settings: {}", e);
            }
            settings.set(current_settings);
        });
    };

    let on_estimator_select = move |estimator: TokenEstimator| {
        let on_estimator_change = on_estimator_change.clone();
        let mut settings = settings.clone();
        spawn(async move {
            on_estimator_change.call(estimator.clone());
            let mut current_settings = settings.read().clone();
            current_settings.set_token_estimator(estimator);
            if let Err(e) = current_settings.save().await {
                log::error!("Failed to save settings: {}", e);
            }
            settings.set(current_settings);
        });
    };

    rsx! {
        div {
            class: "flex items-center space-x-4 p-4 bg-gray-100 dark:bg-gray-800",

            button {
                class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                onclick: on_open_folder,
                "Open Folder..."
            }

            // Recent workspaces dropdown
            select {
                class: "px-4 py-2 bg-white dark:bg-gray-700 rounded",
                onchange: move |evt| {
                    if let Some(path) = evt.value().parse::<PathBuf>().ok() {
                        on_recent_select(path);
                    }
                },
                option { value: "", "Recent Workspaces" }
                {settings.read().get_recent_workspaces().iter().map(|path| {
                    rsx! {
                        option {
                            value: "{path.to_string_lossy()}",
                            "{path.to_string_lossy()}"
                        }
                    }
                })}
            }

            // Tokenizer dropdown
            select {
                class: "px-4 py-2 bg-white dark:bg-gray-700 rounded",
                value: "{current_estimator}",
                onchange: move |evt| {
                    if let Ok(estimator) = evt.value().parse::<TokenEstimator>() {
                        on_estimator_select(estimator);
                    }
                },
                option { value: "CharDiv4", "Char/4 (Fast)" }
                option { value: "Cl100k", "GPT-3/4 (cl100k)" }
                option { value: "Llama2", "Llama2 BPE" }
                option { value: "SentencePiece", "Gemini SentencePiece" }
            }

            if has_files {
                button {
                    class: "px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600",
                    onclick: move |_| on_select_all.call(()),
                    "Select All"
                }

                button {
                    class: "px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600",
                    onclick: move |_| on_deselect_all.call(()),
                    "Deselect All"
                }
            }
        }
    }
}

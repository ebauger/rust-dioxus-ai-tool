#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, prelude::*, registry};

mod cache;
mod components;
mod fs_utils;
mod settings;
mod tokenizer;

use components::{FileList, Footer, ProgressModal, Toolbar};
use fs_utils::{FileInfo, ProgressState};
use settings::Settings;
use tokenizer::TokenEstimator;

fn main() {
    // Set up file logging
    let config_dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rust-dioxus-ai-tool");

    // Create config directory if it doesn't exist
    std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let log_file_path = config_dir.join("app.log");
    let file_appender = tracing_appender::rolling::daily(&config_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up logging to both console and file
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stdout).with_ansi(true))
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    info!("Starting app, logging to {}", log_file_path.display());

    dioxus::launch(app);
}

#[component]
fn app() -> Element {
    let _settings = use_signal(|| Settings::default());
    let mut current_dir = use_signal(|| None::<PathBuf>);
    let mut files = use_signal(Vec::<FileInfo>::new);
    let mut selected_files = use_signal(HashSet::<PathBuf>::new);
    let mut estimator = use_signal(|| TokenEstimator::default());
    let progress = use_signal(|| ProgressState::new());

    // Only process directory when it changes
    use_effect(move || {
        if let Some(dir) = current_dir.read().as_ref() {
            let dir = dir.clone();
            let mut progress = progress.clone();

            let (_status_tx, mut status_rx) = mpsc::channel(32);

            spawn(async move {
                // Just list files without processing tokens
                match fs_utils::list_files(&dir).await {
                    Ok(new_files) => {
                        files.set(new_files);
                        selected_files.set(HashSet::new()); // Ensure no files are selected by default
                    }
                    Err(e) => {
                        eprintln!("Error listing directory: {}", e);
                    }
                }

                // Reset progress state
                progress.set(ProgressState::new());
            });

            spawn(async move {
                while let Some((completed, total, message)) = status_rx.recv().await {
                    let mut new_state = ProgressState::new();
                    new_state.update(completed, total, message);
                    progress.set(new_state);
                }
            });
        }
    });

    // Process token counts for selected files
    use_effect(move || {
        let selected = selected_files.read().clone();
        if !selected.is_empty() {
            let mut files_signal = files.clone();
            let estimator = estimator.read().clone();
            let mut progress_signal = progress.clone();

            let (status_tx, mut status_rx) = mpsc::channel(32);

            spawn(async move {
                let mut current_files = files_signal.read().clone();
                let total = selected.len();
                let mut processed = 0;

                for file in &mut current_files {
                    if selected.contains(&file.path) {
                        match FileInfo::with_tokens(file.path.clone(), &estimator) {
                            Ok(updated_file) => {
                                *file = updated_file;
                            }
                            Err(e) => {
                                eprintln!("Error processing file {}: {}", file.path.display(), e);
                            }
                        }
                        processed += 1;
                        let _ = status_tx
                            .send((processed, total, "Processing files...".to_string()))
                            .await;
                    }
                }

                files_signal.write().clone_from(&current_files);
            });

            spawn(async move {
                while let Some((completed, total, message)) = status_rx.recv().await {
                    let mut new_state = ProgressState::new();
                    new_state.update(completed, total, message);
                    progress_signal.set(new_state);
                }
            });
        }
    });

    // Add keyboard shortcuts
    use_effect(move || {
        let mut selected_files = selected_files.clone();
        let files = files.read().clone();

        let _ = dioxus::desktop::use_global_shortcut("Ctrl+A", move || {
            let mut new_selection = HashSet::new();
            for file in files.iter() {
                new_selection.insert(file.path.clone());
            }
            selected_files.set(new_selection);
        });

        let _ = dioxus::desktop::use_global_shortcut("Escape", move || {
            selected_files.set(HashSet::new());
        });
    });

    let progress_state = progress.read();
    let show_progress = !progress_state.message.is_empty();

    rsx! {
        div {
            class: "flex flex-col h-screen",

            Toolbar {
                on_workspace_select: move |path| {
                    current_dir.set(Some(path));
                },
                on_select_all: move |_| {
                    let mut new_selection = HashSet::new();
                    for file in files.read().iter() {
                        new_selection.insert(file.path.clone());
                    }
                    selected_files.set(new_selection);
                },
                on_deselect_all: move |_| {
                    selected_files.set(HashSet::new());
                },
                on_estimator_change: move |new_estimator| {
                    estimator.set(new_estimator);
                },
                has_files: !files.read().is_empty(),
                current_estimator: estimator.read().clone(),
                selected_files: selected_files.clone(),
            }

            FileList {
                files: files.read().clone(),
                selected_files: selected_files.clone(),
                on_select_all: move |_| {
                    let mut new_selection = HashSet::new();
                    for file in files.read().iter() {
                        new_selection.insert(file.path.clone());
                    }
                    selected_files.set(new_selection);
                },
                on_deselect_all: move |_| {
                    selected_files.set(HashSet::new());
                },
            }

            Footer {
                files: files.read().clone(),
                selected_files: selected_files.clone(),
                current_estimator: estimator.read().clone(),
            }

            if show_progress {
                ProgressModal {
                    completed: progress_state.completed,
                    total: progress_state.total,
                    message: progress_state.message.clone(),
                }
            }
        }
    }
}

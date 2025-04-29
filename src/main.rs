#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

mod cache;
mod components;
mod fs_utils;
mod settings;
mod tokenizer;

use components::{FileList, ProgressModal, Toolbar};
use fs_utils::{FileInfo, ProgressCallback, ProgressState};
use settings::Settings;
use tokenizer::TokenEstimator;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");

    dioxus::launch(app);
}

#[component]
fn app() -> Element {
    let _settings = use_signal(|| Settings::default());
    let mut current_dir = use_signal(|| PathBuf::from("."));
    let mut files = use_signal(Vec::<FileInfo>::new);
    let mut selected_files = use_signal(HashSet::<PathBuf>::new);
    let mut estimator = use_signal(|| TokenEstimator::default());
    let progress = use_signal(|| ProgressState::new());

    use_effect(move || {
        let dir = current_dir.read().clone();
        let estimator = estimator.read().clone();
        let mut progress = progress.clone();

        let (tx, mut rx) = mpsc::channel(32);
        let progress_callback: ProgressCallback =
            Arc::new(Box::new(move |completed, total, message| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send((completed, total, message)).await;
                });
            }));

        spawn(async move {
            let new_files =
                fs_utils::crawl_directory(&dir, estimator, Some(progress_callback)).await;
            files.set(new_files);
            selected_files.set(HashSet::new()); // Clear selection when directory changes

            // Reset progress state
            progress.set(ProgressState::new());
        });

        spawn(async move {
            while let Some((completed, total, message)) = rx.recv().await {
                let mut new_state = ProgressState::new();
                new_state.update(completed, total, message);
                progress.set(new_state);
            }
        });
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
                    current_dir.set(path);
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
            }

            FileList {
                files: files.read().clone(),
                selected_files: selected_files.clone(),
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

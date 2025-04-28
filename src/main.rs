#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use std::collections::HashSet;
use std::path::PathBuf;

mod cache;
mod components;
mod fs_utils;
mod settings;
mod tokenizer;

use components::{FileList, Toolbar};
use fs_utils::FileInfo;
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
    let estimator = use_signal(|| TokenEstimator::default());

    use_effect(move || {
        let dir = current_dir.read().clone();
        let estimator = estimator.read().clone();
        spawn(async move {
            let new_files = fs_utils::crawl_directory(&dir, estimator).await;
            files.set(new_files);
            selected_files.set(HashSet::new()); // Clear selection when directory changes
        });
    });

    // Add keyboard shortcuts
    use_effect(move || {
        let mut new_selection = selected_files.read().clone();
        let files = files.read().clone();

        dioxus::desktop::use_global_shortcut("Ctrl+A", move || {
            new_selection.clear();
            for file in files.iter() {
                new_selection.insert(file.path.clone());
            }
            selected_files.set(new_selection.clone());
        });

        dioxus::desktop::use_global_shortcut("Escape", move || {
            selected_files.set(HashSet::new());
        });
    });

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
                has_files: !files.read().is_empty(),
            }

            FileList {
                files: files.read().clone(),
                selected_files: selected_files.read().clone(),
                on_select: move |path| {
                    let mut new_selection = selected_files.read().clone();
                    new_selection.insert(path);
                    selected_files.set(new_selection);
                },
                on_deselect: move |path| {
                    let mut new_selection = selected_files.read().clone();
                    new_selection.remove(&path);
                    selected_files.set(new_selection);
                },
            }
        }
    }
}

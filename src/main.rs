#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
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
    let estimator = use_signal(|| TokenEstimator::default());

    use_effect(move || {
        let dir = current_dir.read().clone();
        let estimator = estimator.read().clone();
        spawn(async move {
            let new_files = fs_utils::crawl_directory(&dir, estimator).await;
            files.set(new_files);
        });
    });

    rsx! {
        div {
            class: "flex flex-col h-screen",

            Toolbar {
                on_workspace_select: move |path| {
                    current_dir.set(path);
                }
            }

            FileList {
                files: files.read().clone()
            }
        }
    }
}

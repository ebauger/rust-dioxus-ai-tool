#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::muda;
use dioxus_desktop::use_muda_event_handler;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, prelude::*};

mod cache;
mod components;
mod fs_utils;
mod gitignore_handler;
mod settings;
mod tokenizer;
mod workspace_event_handler;

use components::{FileTree, Footer, Toolbar};
use fs_utils::FileInfo;
use settings::Settings;
use tokenizer::TokenEstimator;

// Define constant for max recent workspaces
const MAX_RECENTS: usize = 5;

// Store menu item IDs
#[derive(Clone, PartialEq)]
struct MenuIds {
    open: muda::MenuId,
    recent_items: Vec<muda::MenuId>,
    clear_recents: muda::MenuId,
}

fn create_menu(settings: &Settings) -> (muda::Menu, MenuIds) {
    // Create menu items
    let open_item = muda::MenuItem::new("Open...", true, None);
    let open_id = open_item.id().clone();
    let close_item = muda::PredefinedMenuItem::close_window(None);

    // Create recent workspace menu items
    let mut recent_items = Vec::new();
    let mut recent_menu_items = Vec::new();
    let recent_workspaces = settings.get_recent_workspaces();

    for path in recent_workspaces {
        let path_str = path.to_string_lossy().into_owned();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&path_str);
        let item = muda::MenuItem::new(name, true, None);
        recent_items.push(item.id().clone());
        recent_menu_items.push(item);
    }

    // Add "Clear Recent Workspaces" item if there are recent workspaces
    let clear_item = muda::MenuItem::new(
        "Clear Recent Workspaces",
        !recent_workspaces.is_empty(),
        None,
    );
    let clear_id = clear_item.id().clone();

    // Create Recent Workspaces submenu
    let mut menu_items = Vec::new();
    for item in &recent_menu_items {
        menu_items.push(item);
    }

    let mut submenu_items = Vec::new();
    let mut owned_items = Vec::new();
    if !recent_workspaces.is_empty() {
        submenu_items.extend(menu_items.iter().map(|item| *item as &dyn muda::IsMenuItem));
        owned_items.push(muda::PredefinedMenuItem::separator());
        submenu_items.push(&owned_items[0]);
        submenu_items.push(&clear_item);
    }

    let recent_submenu = muda::Submenu::with_items(
        "Recent Workspaces",
        !recent_workspaces.is_empty(),
        &submenu_items,
    )
    .unwrap();

    // Create File menu
    let file_submenu = muda::Submenu::with_items(
        "File",
        true,
        &[
            &open_item,
            &muda::PredefinedMenuItem::separator(),
            &recent_submenu,
            &muda::PredefinedMenuItem::separator(),
            &close_item,
        ],
    )
    .unwrap();

    // Create main menu and control order per platform
    let menu = muda::Menu::new();

    // macOS: App submenu first, then File, then Help
    #[cfg(target_os = "macos")]
    {
        let about_item = muda::PredefinedMenuItem::about(
            Some("Context Loader"),
            Some(muda::AboutMetadata {
                name: Some("Context Loader".into()),
                ..Default::default()
            }),
        );

        let app_submenu = muda::Submenu::with_items(
            "Context Loader",
            true,
            &[
                &about_item,
                &muda::PredefinedMenuItem::separator(),
                &muda::PredefinedMenuItem::services(None),
                &muda::PredefinedMenuItem::separator(),
                &muda::PredefinedMenuItem::hide(Some("Hide Context Loader".into())),
                &muda::PredefinedMenuItem::hide_others(Some("Hide Others".into())),
                &muda::PredefinedMenuItem::show_all(Some("Show All".into())),
                &muda::PredefinedMenuItem::separator(),
                &muda::PredefinedMenuItem::quit(Some("Quit Context Loader".into())),
            ],
        )
        .unwrap();

        menu.append(&app_submenu).unwrap();
        menu.append(&file_submenu).unwrap();
    }

    // Windows: File first then Help
    #[cfg(target_os = "windows")]
    {
        menu.append(&file_submenu).unwrap();

        let about_item = muda::PredefinedMenuItem::about(
            Some("Context Loader"),
            Some(muda::AboutMetadata {
                name: Some("Context Loader".into()),
                ..Default::default()
            }),
        );

        let help_submenu = muda::Submenu::with_items("Help", true, &[&about_item]).unwrap();
        menu.append(&help_submenu).unwrap();
    }

    // Linux or other unix: File first then Help
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        menu.append(&file_submenu).unwrap();

        let about_item = muda::PredefinedMenuItem::about(
            Some("Context Loader"),
            Some(muda::AboutMetadata {
                name: Some("Context Loader".into()),
                ..Default::default()
            }),
        );

        let help_submenu = muda::Submenu::with_items("Help", true, &[&about_item]).unwrap();
        menu.append(&help_submenu).unwrap();
    }

    (
        menu,
        MenuIds {
            open: open_id,
            recent_items,
            clear_recents: clear_id,
        },
    )
}

#[derive(Props, Clone, PartialEq)]
struct AppProps {
    menu_ids: MenuIds,
}

#[component]
fn App() -> Element {
    let menu_ids = use_context::<MenuIds>();
    let settings_file = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("context-loader")
        .join("settings.json");
    let settings_data = Settings::new(settings_file);
    let mut settings = use_signal(|| settings_data);
    let mut current_workspace = use_signal(|| None::<PathBuf>);
    let mut selected_files = use_signal(|| HashSet::new());
    let mut files = use_signal(|| Vec::<FileInfo>::new());

    // Load file list (without tokens) when workspace changes
    use_effect(move || {
        if let Some(path) = current_workspace.read().clone() {
            let mut files_signal = files.clone();
            let mut selected_files_signal = selected_files.clone();
            let workspace_path_for_handler = path.clone();

            spawn(async move {
                match fs_utils::list_files(&path).await {
                    Ok(list) => files_signal.set(list),
                    Err(e) => log::error!("Failed to list workspace files: {}", e),
                }

                let workspace_path_str = workspace_path_for_handler.to_string_lossy().into_owned();
                match crate::workspace_event_handler::handle_workspace_opened(workspace_path_str) {
                    Ok(initially_selected_relative_paths) => {
                        let workspace_root = workspace_path_for_handler;
                        let initial_selection_absolute: HashSet<PathBuf> =
                            initially_selected_relative_paths
                                .into_iter()
                                .map(|rel_path| workspace_root.join(rel_path))
                                .collect();

                        selected_files_signal.set(initial_selection_absolute);
                        log::info!("Initial file selection complete based on .gitignore.");
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to determine initial file selection: {}. Resetting selection.",
                            e
                        );
                        selected_files_signal.set(HashSet::new());
                    }
                }
            });
        } else {
            files.set(Vec::new());
            selected_files.set(HashSet::new());
        }
    });

    // Lazily compute token counts only for selected files
    use_effect(move || {
        let selected = selected_files.read().clone();
        if selected.is_empty() {
            return;
        }

        let mut files_signal = files.clone();
        let estimator = settings.read().get_token_estimator();

        spawn(async move {
            let mut list = files_signal.read().clone();
            let mut updated = false;
            for file in &mut list {
                if selected.contains(&file.path) && file.token_count == 0 {
                    match estimator.estimate_file_tokens(&file.path) {
                        Ok(tokens) => {
                            file.token_count = tokens;
                            updated = true;
                        }
                        Err(e) => log::error!(
                            "Failed to estimate tokens for {}: {}",
                            file.path.display(),
                            e
                        ),
                    }
                }
            }
            if updated {
                files_signal.set(list);
            }
        });
    });

    // Handle menu events
    use_muda_event_handler(move |event| {
        if event.id == menu_ids.open {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                println!("Opening workspace: {:?}", path);
                current_workspace.set(Some(path.clone()));
                spawn(async move {
                    let mut current_settings_data = settings.read().clone();
                    current_settings_data.add_recent_workspace(path.clone());
                    if let Err(e) = current_settings_data.save().await {
                        log::error!("Failed to save settings: {}", e);
                    }
                    settings.set(current_settings_data);
                });
            }
        } else if menu_ids.recent_items.iter().any(|id| *id == event.id) {
            let index = menu_ids
                .recent_items
                .iter()
                .position(|id| *id == event.id)
                .unwrap();
            let path = settings.read().get_recent_workspaces()[index].clone();
            println!("Opening recent workspace: {:?}", path);
            current_workspace.set(Some(path.clone()));
            spawn(async move {
                let mut current_settings_data = settings.read().clone();
                current_settings_data.add_recent_workspace(path.clone());
                if let Err(e) = current_settings_data.save().await {
                    log::error!("Failed to save settings: {}", e);
                }
                settings.set(current_settings_data);
            });
        } else if event.id == menu_ids.clear_recents {
            spawn(async move {
                let mut current_settings_data = settings.read().clone();
                current_settings_data.clear_recent_workspaces();
                if let Err(e) = current_settings_data.save().await {
                    log::error!("Failed to save settings: {}", e);
                }
                settings.set(current_settings_data);
            });
        }
    });

    rsx! {
        dioxus::prelude::document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        }
        dioxus::prelude::document::Stylesheet {
            href: asset!("/assets/main.css")
        }
        div {
            class: "flex flex-col h-screen bg-light-background text-light-foreground",
            if let Some(_) = current_workspace.read().as_ref() {
                div {
                    class: "flex flex-col flex-1 overflow-hidden", // take remaining height
                    Toolbar {
                        has_files: !files.read().is_empty(),
                        on_select_all: move |_| {
                            let all_paths: HashSet<PathBuf> = files.read().iter().map(|f| f.path.clone()).collect();
                            selected_files.set(all_paths);
                        },
                        on_deselect_all: move |_| {
                            selected_files.set(HashSet::new());
                        },
                        on_estimator_change: move |estimator: TokenEstimator| {
                            spawn(async move {
                                let mut current_settings_data = settings.read().clone();
                                current_settings_data.set_token_estimator(estimator.clone());
                                if let Err(e) = current_settings_data.save().await {
                                    log::error!("Failed to save settings: {}", e);
                                }
                                settings.set(current_settings_data);
                                // Recompute tokens with new estimator
                                if let Some(path) = current_workspace.read().clone() {
                                    match fs_utils::crawl(&path, &estimator, None).await {
                                        Ok(list) => files.set(list),
                                        Err(e) => log::error!("Failed to crawl workspace: {}", e),
                                    }
                                }
                            });
                        },
                        current_estimator: settings.read().get_token_estimator(),
                        selected_files: selected_files.clone(),
                    }
                    // File list scrollable area
                    div {
                        class: "flex-1 overflow-auto p-4",
                        FileTree {
                            all_files: files.read().clone(),
                            selected_paths: selected_files.clone(),
                            on_select_all: |_| {},
                            on_deselect_all: |_| {},
                            workspace_root: current_workspace.read().clone().expect("Workspace root must exist when FileTree is rendered")
                        }
                    }
                    Footer {
                        files: files.read().clone(),
                        selected_files: selected_files.clone(),
                        current_estimator: settings.read().get_token_estimator(),
                    }
                }
            } else {
                div {
                    class: "flex flex-col items-center justify-center h-full w-full",
                    // Welcome message removed
                    div {
                        class: "text-lg text-light-secondary-text",
                        "Open a workspace to get started"
                    }
                }
            }
        }
    }
}

fn main() {
    // Set up file logging
    let config_dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("context-loader");

    // Create config directory if it doesn't exist
    std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let _log_file_path = config_dir.join("app.log");
    let file_appender = tracing_appender::rolling::daily(&config_dir, "app.log");

    // Set up logging to both file and stdout with different filters
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("dioxus=debug".parse().unwrap()),
        )
        .init();

    // Load settings and create menu
    let settings_file = config_dir.join("settings.json");
    let settings = Settings::new(settings_file);
    let (menu, menu_ids) = create_menu(&settings);

    // Launch app with configuration
    let window = WindowBuilder::new()
        .with_title("Context Loader")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .with_resizable(true);

    let config = Config::default().with_window(window).with_menu(Some(menu));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(menu_ids)
        .launch(App);
}

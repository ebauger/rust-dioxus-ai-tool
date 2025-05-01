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
mod settings;
mod tokenizer;

use components::Toolbar;
use settings::Settings;

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

    // Create main menu
    let menu = muda::Menu::with_items(&[&file_submenu]).unwrap();

    // Platform specific menus
    #[cfg(target_os = "macos")]
    {
        let about_item = muda::PredefinedMenuItem::about(
            None,
            Some(muda::AboutMetadata {
                name: Some("Rust Dioxus AI Tool".into()),
                ..Default::default()
            }),
        );

        let app_submenu = muda::Submenu::with_items(
            "App",
            true,
            &[
                &about_item,
                &muda::PredefinedMenuItem::separator(),
                &muda::PredefinedMenuItem::services(None),
                &muda::PredefinedMenuItem::separator(),
                &muda::PredefinedMenuItem::hide(None),
                &muda::PredefinedMenuItem::hide_others(None),
                &muda::PredefinedMenuItem::show_all(None),
                &muda::PredefinedMenuItem::separator(),
                &muda::PredefinedMenuItem::quit(None),
            ],
        )
        .unwrap();

        menu.append(&app_submenu).unwrap();
    }

    // Windows-specific menu initialization
    #[cfg(target_os = "windows")]
    {
        let about_item = muda::PredefinedMenuItem::about(
            None,
            Some(muda::AboutMetadata {
                name: Some("Rust Dioxus AI Tool".into()),
                ..Default::default()
            }),
        );

        let help_submenu = muda::Submenu::with_items("Help", true, &[&about_item]).unwrap();

        menu.append(&help_submenu).unwrap();
    }

    // Linux-specific menu initialization
    #[cfg(target_os = "linux")]
    {
        let about_item = muda::PredefinedMenuItem::about(
            None,
            Some(muda::AboutMetadata {
                name: Some("Rust Dioxus AI Tool".into()),
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
    settings: Settings,
}

fn app() -> Element {
    let settings_file = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rust-dioxus-ai-tool")
        .join("settings.json");
    let settings = Settings::new(settings_file);
    let (_menu, menu_ids) = create_menu(&settings);

    let settings = use_signal(|| settings);
    let current_workspace = use_signal(|| None::<PathBuf>);
    let selected_files = use_signal(|| HashSet::new());

    // Handle menu events
    use_muda_event_handler(move |event| {
        let mut settings = settings.clone();
        let mut current_workspace = current_workspace.clone();
        if event.id == menu_ids.open {
            // Show file dialog to open workspace
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                // Handle opening workspace
                println!("Opening workspace: {:?}", path);
                current_workspace.set(Some(path.clone()));
                spawn(async move {
                    let mut current_settings = settings.read().clone();
                    current_settings.add_recent_workspace(path.clone());
                    if let Err(e) = current_settings.save().await {
                        log::error!("Failed to save settings: {}", e);
                    }
                    settings.set(current_settings);
                });
            }
        } else if menu_ids.recent_items.iter().any(|id| *id == event.id) {
            // Find the selected recent workspace
            let index = menu_ids
                .recent_items
                .iter()
                .position(|id| *id == event.id)
                .unwrap();
            let path = settings.read().get_recent_workspaces()[index].clone();
            println!("Opening recent workspace: {:?}", path);
            current_workspace.set(Some(path.clone()));
            spawn(async move {
                let mut current_settings = settings.read().clone();
                current_settings.add_recent_workspace(path.clone());
                if let Err(e) = current_settings.save().await {
                    log::error!("Failed to save settings: {}", e);
                }
                settings.set(current_settings);
            });
        } else if event.id == menu_ids.clear_recents {
            // Clear recent workspaces
            spawn(async move {
                let mut current_settings = settings.read().clone();
                current_settings.clear_recent_workspaces();
                if let Err(e) = current_settings.save().await {
                    log::error!("Failed to save settings: {}", e);
                }
                settings.set(current_settings);
            });
        }
    });

    rsx! {
        div {
            class: "flex flex-col h-screen",
            if let Some(_) = current_workspace.read().as_ref() {
                Toolbar {
                    has_files: true,
                    on_select_all: move |_| {
                        // TODO: Implement select all
                    },
                    on_deselect_all: move |_| {
                        // TODO: Implement deselect all
                    },
                    on_estimator_change: move |estimator| {
                        let mut settings = settings.clone();
                        spawn(async move {
                            let mut current_settings = settings.read().clone();
                            current_settings.set_token_estimator(estimator);
                            if let Err(e) = current_settings.save().await {
                                log::error!("Failed to save settings: {}", e);
                            }
                            settings.set(current_settings);
                        });
                    },
                    current_estimator: settings.read().get_token_estimator(),
                    selected_files: selected_files.clone(),
                }
            } else {
                div {
                    class: "flex flex-col items-center justify-center h-full",
                    div {
                        class: "text-2xl font-bold mb-4",
                        "Welcome to Rust Dioxus AI Tool"
                    }
                    div {
                        class: "text-lg text-gray-600",
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
        .join("rust-dioxus-ai-tool");

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

    // Launch app with configuration
    let window = WindowBuilder::new()
        .with_title("Rust Dioxus AI Tool")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .with_resizable(true);

    let config = Config::default()
        .with_window(window)
        .with_custom_head(format!(
            r#"<link rel="stylesheet" href="/{}"/>"#,
            asset!("/assets/main.css")
        ));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app);
}

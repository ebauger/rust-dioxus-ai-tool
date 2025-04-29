use crate::components::copy_button::CopyButton;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

#[test]
fn test_copy_button_disabled_when_no_files_selected() {
    let mut app = VirtualDom::new(|cx| {
        let selected_files = use_signal(cx, || HashSet::new());
        let on_copy = move |_| {};

        cx.render(rsx! {
            CopyButton {
                selected_files: selected_files,
                on_copy: on_copy,
            }
        })
    });

    let mut dom = app.rebuild();
    let button = dom.get_element_by_id("copy-button").unwrap();
    assert!(button.get_attribute("disabled").is_some());
}

#[test]
fn test_copy_button_enabled_when_files_selected() {
    let mut app = VirtualDom::new(|cx| {
        let mut selected_files = HashSet::new();
        selected_files.insert(PathBuf::from("test.txt"));
        let selected_files = use_signal(cx, || selected_files);
        let on_copy = move |_| {};

        cx.render(rsx! {
            CopyButton {
                selected_files: selected_files,
                on_copy: on_copy,
            }
        })
    });

    let mut dom = app.rebuild();
    let button = dom.get_element_by_id("copy-button").unwrap();
    assert!(button.get_attribute("disabled").is_none());
}

#[test]
fn test_copy_button_shows_loading_state() {
    let mut app = VirtualDom::new(|cx| {
        let mut selected_files = HashSet::new();
        selected_files.insert(PathBuf::from("test.txt"));
        let selected_files = use_signal(cx, || selected_files);
        let on_copy = move |_| {};

        cx.render(rsx! {
            CopyButton {
                selected_files: selected_files,
                on_copy: on_copy,
            }
        })
    });

    let mut dom = app.rebuild();
    let button = dom.get_element_by_id("copy-button").unwrap();
    button.click();
    assert_eq!(button.text_content(), "Copying...");
}

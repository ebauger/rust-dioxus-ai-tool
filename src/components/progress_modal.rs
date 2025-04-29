#![allow(non_snake_case)]

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ProgressModalProps {
    completed: usize,
    total: usize,
    message: String,
}

#[component]
pub fn ProgressModal(props: ProgressModalProps) -> Element {
    let ProgressModalProps {
        completed,
        total,
        message,
    } = props;

    let percentage = if total > 0 {
        (completed as f64 / total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div {
                class: "bg-white rounded-lg p-6 w-96",
                h2 {
                    class: "text-xl font-semibold mb-4",
                    "{message}"
                }
                div {
                    class: "w-full bg-gray-200 rounded-full h-2.5 mb-4",
                    div {
                        class: "bg-blue-600 h-2.5 rounded-full transition-all duration-300",
                        style: "width: {percentage}%",
                    }
                }
                p {
                    class: "text-sm text-gray-600 text-center",
                    "{completed} of {total} files processed"
                }
            }
        }
    }
}

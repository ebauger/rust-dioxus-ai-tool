#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::fmt::{self, Display};
use std::str::FromStr;

/// Represents the different types of file filtering options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Substring, // Simple substring match
    Extension, // File extension match (e.g., ".rs", ".txt")
    Regex,     // Regular expression match
}

impl Display for FilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterType::Substring => write!(f, "Substring"),
            FilterType::Extension => write!(f, "Extension"),
            FilterType::Regex => write!(f, "Regex"),
        }
    }
}

impl FromStr for FilterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Substring" => Ok(FilterType::Substring),
            "Extension" => Ok(FilterType::Extension),
            "Regex" => Ok(FilterType::Regex),
            _ => Err(format!("Unknown filter type: {}", s)),
        }
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct FilterInputProps {
    /// Current filter text
    filter_text: Signal<String>,
    /// Current filter type
    filter_type: Signal<FilterType>,
}

/// Component that renders a filter input with options for filter type
#[component]
pub fn FilterInput(props: FilterInputProps) -> Element {
    let FilterInputProps {
        filter_text,
        filter_type,
    } = props;

    let mut filter_text = filter_text.clone();
    let mut filter_type = filter_type.clone();

    let placeholder = match *filter_type.read() {
        FilterType::Substring => "Search by text...",
        FilterType::Extension => "Filter by extension (e.g. .rs)",
        FilterType::Regex => "Search with regex (e.g. .*\\.rs$)",
    };

    rsx! {
        div {
            class: "flex items-center space-x-2 mb-4",

            // Filter type selector
            select {
                class: "bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm",
                value: "{*filter_type.read()}",
                onchange: move |evt| {
                    if let Ok(new_filter_type) = evt.value().parse() {
                        filter_type.set(new_filter_type);
                    }
                },
                option { value: "{FilterType::Substring}", "Substring" }
                option { value: "{FilterType::Extension}", "Extension" }
                option { value: "{FilterType::Regex}", "Regex" }
            }

            // Filter input
            input {
                class: "flex-grow bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{filter_text.read()}",
                oninput: move |evt| {
                    filter_text.set(evt.value().clone());
                }
            }

            // Clear button
            button {
                class: "px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700",
                onclick: move |_| {
                    filter_text.set(String::new());
                },
                disabled: filter_text.read().is_empty(),
                "Clear"
            }
        }
    }
}

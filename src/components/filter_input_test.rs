#![cfg(test)]

use crate::components::filter_input::FilterType;
use std::str::FromStr;

#[test]
fn test_filter_type_from_str() {
    assert_eq!(
        "Substring".parse::<FilterType>().unwrap(),
        FilterType::Substring
    );
    assert_eq!(
        "Extension".parse::<FilterType>().unwrap(),
        FilterType::Extension
    );
    assert_eq!("Regex".parse::<FilterType>().unwrap(), FilterType::Regex);

    assert!("Unknown".parse::<FilterType>().is_err());
}

#[test]
fn test_filter_type_display() {
    assert_eq!(FilterType::Substring.to_string(), "Substring");
    assert_eq!(FilterType::Extension.to_string(), "Extension");
    assert_eq!(FilterType::Regex.to_string(), "Regex");
}

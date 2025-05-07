pub mod file_tree;
// mod file_list; // Keep this commented out or remove if FileList is truly gone
// mod file_list_test; // Keep this commented out or remove
pub mod copy_button;
mod copy_button_test;
mod file_tree_test;
pub mod footer;
mod footer_test;
pub mod toolbar;
// mod filter_input; // If FilterInput is unused, its module declaration can be removed too.
// mod filter_input_test; // Same for its test module.
// mod progress_modal; // If ProgressModal is unused, its module declaration can be removed.
// mod progress_modal_test; // Same for its test module.

pub use file_tree::FileTree;
// pub use file_list::FileList; // Keep commented
// pub use filter_input::{FilterInput, FilterType}; // Removed pub use
pub use footer::Footer;
// pub use progress_modal::ProgressModal; // Removed pub use
pub use copy_button::CopyButton;
pub use toolbar::Toolbar;

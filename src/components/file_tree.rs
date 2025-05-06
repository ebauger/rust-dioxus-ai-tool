use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeNodeType {
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeSelectionState {
    Selected,
    NotSelected,
    PartiallySelected,
}

#[derive(PartialEq)]
pub struct FileTreeNode {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub node_type: TreeNodeType,
    pub children: Vec<FileTreeNode>,
    pub is_expanded: Signal<bool>,
    pub selection_state: Signal<NodeSelectionState>,
    pub depth: usize,
}

// Required for Signal to work with FileTreeNode if it's used in a Signal directly.
// However, children is Vec<FileTreeNode>, not Signal<Vec<FileTreeNode>>.
// And the FileTreeNode itself is not directly in a signal in the requirements,
// rather its fields `is_expanded` and `selection_state` are signals.
// If we were to have `Signal<FileTreeNode>`, then `FileTreeNode` would need `impl PartialEq for FileTreeNode`.
// The derive(PartialEq) above should be sufficient for now. If comparing signals of FileTreeNode becomes necessary,
// we'd need to ensure the PartialEq impl is correct for Dioxus's reactivity.
// For now, deriving PartialEq is fine.
// Note: Dioxus signals typically require the contained type to be 'static + Clone + PartialEq.
// PathBuf is Clone but not Copy. String is Clone but not Copy. Vec is Clone but not Copy.
// Signal<bool> and Signal<NodeSelectionState> are fine.
// The struct FileTreeNode itself being stored in a list or as a field seems fine.
// If we were to put `Signal<Vec<FileTreeNode>>`, that would require `FileTreeNode: Clone + PartialEq`.
// The current `children: Vec<FileTreeNode>` is fine.

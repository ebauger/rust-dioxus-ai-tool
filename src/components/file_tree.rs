use crate::fs_utils::FileInfo;
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
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

// Helper to find or create a node in a list of children
fn find_or_create_node<'a>(
    children: &'a mut Vec<FileTreeNode>,
    name: &str,
    full_path: &PathBuf,
    node_type: TreeNodeType,
    current_id_counter: &mut usize,
    depth: usize,
    is_root_folder: bool, // To expand root folders by default
) -> &'a mut FileTreeNode {
    if let Some(pos) = children
        .iter()
        .position(|c| c.name == name && c.node_type == node_type)
    {
        &mut children[pos]
    } else {
        let id = *current_id_counter;
        *current_id_counter += 1;
        let new_node = FileTreeNode {
            id,
            name: name.to_string(),
            path: full_path.clone(),
            node_type,
            children: Vec::new(),
            is_expanded: Signal::new(if depth == 0 { true } else { is_root_folder }), // Expand root folders
            selection_state: Signal::new(NodeSelectionState::NotSelected), // Initial state
            depth,
        };
        children.push(new_node);
        children.last_mut().unwrap()
    }
}

pub fn build_tree_from_file_info(
    files: &[FileInfo],
    selected_paths: &HashSet<PathBuf>,
    // TODO: Consider adding a 'current_workspace_root: &Path' parameter
    // if paths in FileInfo are absolute and we need to determine the 'root'
    // of the tree structure shown to the user. For now, assuming paths are relative
    // or the structure naturally forms from common prefixes.
) -> Vec<FileTreeNode> {
    if files.is_empty() {
        return Vec::new();
    }

    let mut roots: Vec<FileTreeNode> = Vec::new();
    let mut id_counter = 0; // Simple ID counter

    // Sort files by path to process them in a somewhat predictable order,
    // which helps in constructing the tree layer by layer.
    let mut sorted_files = files.to_vec();
    sorted_files.sort_by_key(|f| f.path.clone());

    for file_info in &sorted_files {
        let path = &file_info.path;
        let mut current_children_list = &mut roots;
        let mut current_path = PathBuf::new();
        let components: Vec<_> = path.components().collect();

        for (idx, component) in components.iter().enumerate() {
            let component_name = component.as_os_str().to_string_lossy().into_owned();
            current_path.push(component_name.clone());

            let is_last_component = idx == components.len() - 1;

            if is_last_component {
                // This is the file itself or the last folder component if path ends with /
                let node_type = TreeNodeType::File; // Assuming FileInfo always refers to files
                let id = id_counter;
                id_counter += 1;

                let selection = if selected_paths.contains(&file_info.path) {
                    NodeSelectionState::Selected
                } else {
                    NodeSelectionState::NotSelected
                };

                let file_node = FileTreeNode {
                    id,
                    name: file_info.name.clone(), // Use name from FileInfo for files
                    path: file_info.path.clone(),
                    node_type,
                    children: Vec::new(),            // Files have no children
                    is_expanded: Signal::new(false), // Files cannot be expanded
                    selection_state: Signal::new(selection),
                    depth: idx,
                };
                current_children_list.push(file_node);
            } else {
                // This is a directory component
                let existing_node = current_children_list
                    .iter_mut()
                    .find(|n| n.name == component_name && n.node_type == TreeNodeType::Folder);

                if let Some(node) = existing_node {
                    // The folder already exists, descend into its children list
                    // Need to break the mutable borrow to re-borrow current_children_list
                    // This part is tricky. We need to get a mutable reference to the children of 'node'.
                    // Let's rethink the structure to avoid complex borrowing.
                    // A HashMap approach might be better or a recursive helper.

                    // For now, let's try a simpler iterative approach that might create duplicates
                    // or require a separate pass to merge.
                    // The find_or_create_node helper is intended to simplify this.
                    // current_children_list = &mut node.children; // This line causes borrow checker issues if not careful
                    // To avoid this, find_or_create_node returns a mutable ref to the node,
                    // and we then access its children.
                    let temp_node_path = current_path.clone(); // Path for the current folder component
                    let parent_node = find_or_create_node(
                        current_children_list,
                        &component_name,
                        &temp_node_path,
                        TreeNodeType::Folder,
                        &mut id_counter,
                        idx,
                        idx == 0, // Expand only the true root folders initially
                    );
                    current_children_list = &mut parent_node.children;
                } else {
                    // Folder does not exist, create it
                    let new_folder_id = id_counter;
                    id_counter += 1;
                    let new_folder_node = FileTreeNode {
                        id: new_folder_id,
                        name: component_name.clone(),
                        path: current_path.clone(),
                        node_type: TreeNodeType::Folder,
                        children: Vec::new(),
                        is_expanded: Signal::new(idx == 0), // Expand only the true root folders initially
                        selection_state: Signal::new(NodeSelectionState::NotSelected), // Will be updated
                        depth: idx,
                    };
                    current_children_list.push(new_folder_node);
                    current_children_list = &mut current_children_list.last_mut().unwrap().children;
                }
            }
        }
    }

    // Post-processing step: Merge duplicate folder entries if any were created due to the iterative approach.
    // This is a simplified tree builder. A more robust one would use a HashMap to track nodes by path.
    // For now, let's assume the current iterative approach with find_or_create_node (if correctly used) handles this.
    // The current implementation of find_or_create_node combined with iterating path components should build the hierarchy correctly
    // without creating duplicates *within the same level*.
    // The logic within the loop for handling folders needs to correctly use `find_or_create_node`.

    // Revised loop structure using find_or_create_node more directly:
    let mut final_roots: Vec<FileTreeNode> = Vec::new();
    let mut node_map: HashMap<PathBuf, usize> = HashMap::new(); // path -> id for quick lookups if needed, but not directly used in this simplified build
    let mut unique_id_counter = 0;

    for file_info in &sorted_files {
        let path = &file_info.path;
        let mut current_parent_children_list = &mut final_roots;
        let mut current_accumulated_path = PathBuf::new();

        let components: Vec<_> = path.components().collect();
        let num_components = components.len();

        for (idx, component_wrapper) in components.iter().enumerate() {
            let component_os_str = component_wrapper.as_os_str();
            let component_name = component_os_str.to_string_lossy().into_owned();
            current_accumulated_path.push(component_os_str);

            let is_last_component = idx == num_components - 1;

            if is_last_component {
                // It's a file
                if !current_parent_children_list
                    .iter()
                    .any(|n| n.path == *path && n.node_type == TreeNodeType::File)
                {
                    let id = unique_id_counter;
                    unique_id_counter += 1;
                    let selection = if selected_paths.contains(path) {
                        NodeSelectionState::Selected
                    } else {
                        NodeSelectionState::NotSelected
                    };
                    let file_node = FileTreeNode {
                        id,
                        name: file_info.name.clone(),
                        path: path.clone(),
                        node_type: TreeNodeType::File,
                        children: Vec::new(),
                        is_expanded: Signal::new(false),
                        selection_state: Signal::new(selection),
                        depth: idx,
                    };
                    current_parent_children_list.push(file_node);
                }
            } else {
                // It's a folder
                let folder_path_clone = current_accumulated_path.clone();
                let folder_node = find_or_create_node(
                    current_parent_children_list,
                    &component_name,
                    &folder_path_clone,
                    TreeNodeType::Folder,
                    &mut unique_id_counter,
                    idx,
                    idx == 0, // Only expand true top-level folders
                );
                current_parent_children_list = &mut folder_node.children;
            }
        }
    }

    // The selection state for folders will be calculated later based on children (Story 7 & 8)
    // For now, they are NotSelected unless explicitly part of selected_paths (which they usually aren't, files are)

    final_roots
}

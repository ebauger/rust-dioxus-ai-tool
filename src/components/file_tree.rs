use crate::fs_utils::FileInfo;
use dioxus::prelude::*;
use std::collections::HashSet;
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

#[derive(PartialEq, Clone)]
pub struct FileTreeNode {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub node_type: TreeNodeType,
    pub children: Vec<FileTreeNode>,
    pub is_expanded: bool,
    pub selection_state: NodeSelectionState,
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
            is_expanded: if depth == 0 { true } else { is_root_folder }, // plain bool
            selection_state: NodeSelectionState::NotSelected,            // plain enum
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

    // Sort files by path to process them in a somewhat predictable order,
    // which helps in constructing the tree layer by layer.
    let mut sorted_files = files.to_vec();
    sorted_files.sort_by_key(|f| f.path.clone());

    // Revised loop structure using find_or_create_node more directly:
    let mut final_roots: Vec<FileTreeNode> = Vec::new();
    // let mut node_map: HashMap<PathBuf, usize> = HashMap::new(); // Not used for now
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
                // Ensure file is not duplicated if path appears multiple times (though FileInfo should be unique by path)
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
                        is_expanded: false,         // plain bool
                        selection_state: selection, // plain enum
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

    final_roots
}

#[derive(Props, Clone, PartialEq)]
pub struct FileTreeProps {
    pub all_files: Vec<FileInfo>,
    pub selected_paths: Signal<HashSet<PathBuf>>,
    pub on_select_all: EventHandler<()>,
    pub on_deselect_all: EventHandler<()>,
}

#[allow(non_snake_case)]
pub fn FileTree(props: FileTreeProps) -> Element {
    let mut tree_nodes = use_signal(Vec::new);

    use_effect(move || {
        // This effect runs when props.all_files changes (component re-renders)
        // or when props.selected_paths signal changes.
        let new_tree = build_tree_from_file_info(&props.all_files, &props.selected_paths.read());
        tree_nodes.set(new_tree);
    });

    rsx! {
        div {
            class: "file-tree-container",
            // TODO: Add Select All / Deselect All buttons here later, using props.on_select_all and props.on_deselect_all
            ul {
                class: "file-tree-list",
                for node in tree_nodes.read().iter() {
                    li {
                        key: "{node.id}", // Unique key for Dioxus list rendering
                        // Basic rendering: checkbox, name, type
                        input {
                            r#type: "checkbox",
                            checked: node.selection_state == NodeSelectionState::Selected, // Direct access
                        }
                        span {
                            " {node.name} "
                        }
                        span {
                            match node.node_type {
                                TreeNodeType::File => "(File)",
                                TreeNodeType::Folder => "(Folder)",
                            }
                        }
                        // Recursive rendering will be handled in Story 4
                    }
                }
            }
        }
    }
}

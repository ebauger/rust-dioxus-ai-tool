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

// This is the blueprint struct with plain data, returned by build_tree_from_file_info
#[derive(PartialEq, Clone, Debug)] // Added Debug
pub struct FileTreeNodeBlueprint {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub node_type: TreeNodeType,
    pub children: Vec<FileTreeNodeBlueprint>, // Children are also blueprints
    pub is_expanded: bool,
    pub selection_state: NodeSelectionState,
    pub depth: usize,
}

// This is the struct used for display, containing Dioxus Signals
#[derive(PartialEq, Clone)] // Required by Dioxus for props if FileTreeNode is a prop, and for Signal<Vec<FileTreeNode>>
pub struct FileTreeNode {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub node_type: TreeNodeType,
    pub children: Vec<FileTreeNode>, // Children are also this signal-containing type
    pub is_expanded: Signal<bool>,
    pub selection_state: Signal<NodeSelectionState>,
    pub depth: usize,
}

// Helper to find or create a blueprint node in a list of children blueprints
fn find_or_create_blueprint_node<'a>(
    children: &'a mut Vec<FileTreeNodeBlueprint>,
    name: &str,
    full_path: &PathBuf,
    node_type: TreeNodeType,
    current_id_counter: &mut usize,
    depth: usize,
    is_root_folder: bool,
) -> &'a mut FileTreeNodeBlueprint {
    if let Some(pos) = children
        .iter()
        .position(|c| c.name == name && c.node_type == node_type)
    {
        &mut children[pos]
    } else {
        let id = *current_id_counter;
        *current_id_counter += 1;
        let new_node = FileTreeNodeBlueprint {
            id,
            name: name.to_string(),
            path: full_path.clone(),
            node_type,
            children: Vec::new(),
            is_expanded: if depth == 0 { true } else { is_root_folder },
            selection_state: NodeSelectionState::NotSelected,
            depth,
        };
        children.push(new_node);
        children.last_mut().unwrap()
    }
}

pub fn build_tree_from_file_info(
    files: &[FileInfo],
    selected_paths: &HashSet<PathBuf>,
) -> Vec<FileTreeNodeBlueprint> {
    // Returns blueprint
    if files.is_empty() {
        return Vec::new();
    }

    let mut sorted_files = files.to_vec();
    sorted_files.sort_by_key(|f| f.path.clone());

    let mut final_roots: Vec<FileTreeNodeBlueprint> = Vec::new();
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
                    let file_node = FileTreeNodeBlueprint {
                        // Create blueprint
                        id,
                        name: file_info.name.clone(),
                        path: path.clone(),
                        node_type: TreeNodeType::File,
                        children: Vec::new(),
                        is_expanded: false,
                        selection_state: selection,
                        depth: idx,
                    };
                    current_parent_children_list.push(file_node);
                }
            } else {
                // It's a folder
                let folder_path_clone = current_accumulated_path.clone();
                let folder_node = find_or_create_blueprint_node(
                    // Use blueprint helper
                    current_parent_children_list,
                    &component_name,
                    &folder_path_clone,
                    TreeNodeType::Folder,
                    &mut unique_id_counter,
                    idx,
                    idx == 0,
                );
                current_parent_children_list = &mut folder_node.children;
            }
        }
    }
    final_roots
}

// Recursive function to convert blueprints to signal-based FileTreeNodes
// This must be called within a Dioxus component/hook context for Signal::new to work.
fn convert_blueprint_to_file_tree_node_recursive(blueprint: FileTreeNodeBlueprint) -> FileTreeNode {
    let children: Vec<FileTreeNode> = blueprint
        .children
        .into_iter()
        .map(convert_blueprint_to_file_tree_node_recursive)
        .collect();

    FileTreeNode {
        id: blueprint.id,
        name: blueprint.name,
        path: blueprint.path,
        node_type: blueprint.node_type,
        children,
        is_expanded: Signal::new(blueprint.is_expanded),
        selection_state: Signal::new(blueprint.selection_state),
        depth: blueprint.depth,
    }
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
    let mut tree_display_nodes = use_signal(Vec::new);

    use_effect(move || {
        let blueprints = build_tree_from_file_info(&props.all_files, &props.selected_paths.read());

        let new_display_nodes: Vec<FileTreeNode> = blueprints
            .into_iter()
            .map(convert_blueprint_to_file_tree_node_recursive)
            .collect();

        tree_display_nodes.set(new_display_nodes);
    });

    rsx! {
        div {
            class: "file-tree-container",
            // TODO: Add Select All / Deselect All buttons here later
            ul {
                class: "file-tree-list p-0 m-0 list-none", // Added some basic list reset
                for node in tree_display_nodes.read().iter() {
                    FileTreeNodeDisplay {
                        key: "{node.id}", // Use node.id for the key directly
                        node: node.clone(), // Pass the FileTreeNode (signal-containing)
                        selected_paths: props.selected_paths,
                    }
                }
            }
        }
    }
}

// Story 4: Implement Recursive Rendering of Tree Nodes
// Task 4.1: Define FileTreeNodeDisplay component
#[derive(Props, Clone, PartialEq)]
pub struct FileTreeNodeDisplayProps {
    // node is the signal-containing FileTreeNode, not a Signal wrapping it.
    pub node: FileTreeNode,
    pub selected_paths: Signal<HashSet<PathBuf>>,
    // Add on_toggle_expanded: EventHandler<usize> later if needed for callbacks
    // Add on_toggle_selection: EventHandler<PathBuf> later
}

#[allow(non_snake_case)]
pub fn FileTreeNodeDisplay(props: FileTreeNodeDisplayProps) -> Element {
    // let node = props.node.clone(); // This clone is unused, props.node is used directly.

    // Task 4.2: Render checkbox, icon, name
    let icon = match props.node.node_type {
        TreeNodeType::File => "📄",
        TreeNodeType::Folder => {
            if *props.node.is_expanded.read() {
                "📂"
            } else {
                "📁"
            }
        }
    };

    // Task 4.3: Apply indentation
    let indent_style = format!("padding-left: {}px;", props.node.depth * 20);

    // Task 5.1 & 5.2: onclick handler for folder expansion/collapse
    let mut is_expanded_signal = props.node.is_expanded; // Get the signal to modify
    let node_type = props.node.node_type.clone(); // Clone for the closure

    rsx! {
        div { // Each node is a div for easier styling and click handling
            style: "{indent_style}",
            class: "file-tree-node-row flex items-center", // Added flex and items-center

            input {
                r#type: "checkbox",
                class: "mr-1", // Added margin
                checked: *props.node.selection_state.read() == NodeSelectionState::Selected,
            }

            span {
                class: "cursor-pointer", // Make it look clickable
                onclick: move |_| {
                    if node_type == TreeNodeType::Folder { // Task 5.3: Only for folders
                        is_expanded_signal.toggle(); // Corrected: Call toggle directly on the Signal
                    }
                },
                "{icon} {props.node.name}"
            }
        }

        if props.node.node_type == TreeNodeType::Folder && *props.node.is_expanded.read() {
            for child_node in props.node.children.iter() {
                FileTreeNodeDisplay {
                    node: child_node.clone(),
                    selected_paths: props.selected_paths,
                }
            }
        }
    }
}

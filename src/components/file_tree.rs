use crate::fs_utils::FileInfo;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use log;
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

impl FileTreeNode {
    // Helper function to collect all descendant file paths for a folder node
    pub fn collect_all_file_paths_recursive(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if self.node_type == TreeNodeType::File {
            // Although this function is intended for folders,
            // if called on a file, it should return its own path.
            paths.push(self.path.clone());
        } else {
            for child in &self.children {
                match child.node_type {
                    TreeNodeType::File => paths.push(child.path.clone()),
                    TreeNodeType::Folder => {
                        paths.extend(child.collect_all_file_paths_recursive());
                    }
                }
            }
        }
        paths
    }
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
    pub node: FileTreeNode,
    pub selected_paths: Signal<HashSet<PathBuf>>,
}

#[allow(non_snake_case)]
pub fn FileTreeNodeDisplay(props: FileTreeNodeDisplayProps) -> Element {
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
    let indent_style = format!("padding-left: {}px;", props.node.depth * 20);
    let mut is_expanded_signal = props.node.is_expanded;
    let node_type_for_click_logic = props.node.node_type.clone(); // Clone for the click handler

    let unique_checkbox_id = format!("ftn-checkbox-{}", props.node.id);

    let selection_state_for_effect = props.node.selection_state;
    let unique_checkbox_id_for_effect = unique_checkbox_id.clone();

    let window = use_window();

    // Clone necessary props for the oninput closure
    let node_for_input = props.node.clone();
    let mut selected_paths_signal = props.selected_paths; // This is already a Signal

    use_effect(move || {
        // Script to set indeterminate state
        let script = format!(
            r#"
            var checkbox = document.getElementById('{}');
            if (checkbox) {{
                checkbox.indeterminate = {};
            }}
            "#,
            unique_checkbox_id_for_effect,
            if *selection_state_for_effect.read() == NodeSelectionState::PartiallySelected {
                "true"
            } else {
                "false"
            }
        );
        if let Err(e) = window.webview.evaluate_script(&script) {
            log::error!("Failed to set indeterminate state: {}", e);
        }
    });

    rsx! {
        li {
            class: "file-tree-node",
            key: "{props.node.id}", // Key for Dioxus list rendering
            div {
                class: "node-row flex items-center hover:bg-gray-200 dark:hover:bg-gray-700 p-1 rounded",
                style: "{indent_style}",
                onclick: move |_| {
                    if node_type_for_click_logic == TreeNodeType::Folder {
                        let current_value = *is_expanded_signal.read();
                        is_expanded_signal.set(!current_value);
                    }
                },
                input {
                    id: "{unique_checkbox_id}",
                    "type": "checkbox",
                    class: "mr-2 form-checkbox rounded text-blue-500 focus:ring-blue-500 dark:text-blue-400 dark:focus:ring-blue-400 dark:bg-gray-700 dark:border-gray-600",
                    checked: props.node.selection_state.read().clone() == NodeSelectionState::Selected,
                    // Indeterminate state handled by use_effect above
                    oninput: move |event| {
                        let is_checked = event.value().parse::<bool>().unwrap_or_else(|_| {
                            log::warn!("Could not parse checkbox value: '{}', defaulting to false.", event.value());
                            false
                        });
                        let mut selected_paths_writer = selected_paths_signal.write();

                        match node_for_input.node_type {
                            TreeNodeType::File => {
                                if is_checked {
                                    selected_paths_writer.insert(node_for_input.path.clone());
                                } else {
                                    selected_paths_writer.remove(&node_for_input.path);
                                }
                            }
                            TreeNodeType::Folder => {
                                let descendant_file_paths = node_for_input.collect_all_file_paths_recursive();
                                for path in descendant_file_paths {
                                    if is_checked {
                                        selected_paths_writer.insert(path);
                                    } else {
                                        selected_paths_writer.remove(&path);
                                    }
                                }
                                // Also update the folder's own path if folders are directly selectable
                                // For now, assuming folders reflect state of children.
                                // If folders themselves are entities to be selected (e.g. to select the folder itself),
                                // this logic might need adjustment or the `collect_all_file_paths_recursive`
                                // might need to include the folder path if a folder node can be a "target" itself.
                                // Based on the task "A file node is Selected if its path is in selected_paths",
                                // it seems only files are directly tracked in `selected_paths`.
                            }
                        }
                    }
                },
                span {
                    class: "node-icon mr-1",
                    "{icon}"
                }
                span {
                    class: "node-name",
                    "{props.node.name}"
                }
            }
            if props.node.node_type == TreeNodeType::Folder && *props.node.is_expanded.read() {
                ul {
                    class: "list-none", // Ensure nested lists also don't have bullets
                    for child_node in &props.node.children {
                        FileTreeNodeDisplay {
                            key: "{child_node.id}",
                            node: child_node.clone(),
                            selected_paths: props.selected_paths,
                        }
                    }
                }
            }
        }
    }
}

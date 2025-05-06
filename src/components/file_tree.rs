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
// Making it pub(crate) for testing the full tree construction and update logic.
pub(crate) fn convert_blueprint_to_file_tree_node_recursive(
    blueprint: FileTreeNodeBlueprint,
    scope_id: ScopeId,
) -> FileTreeNode {
    let children: Vec<FileTreeNode> = blueprint
        .children
        .into_iter()
        .map(|b| convert_blueprint_to_file_tree_node_recursive(b, scope_id))
        .collect();

    FileTreeNode {
        id: blueprint.id,
        name: blueprint.name,
        path: blueprint.path,
        node_type: blueprint.node_type,
        children,
        is_expanded: Signal::new_in_scope(blueprint.is_expanded, scope_id),
        selection_state: Signal::new_in_scope(blueprint.selection_state, scope_id),
        depth: blueprint.depth,
    }
}

// --- Start of Story 8 Implementation ---

// Task 8.1: Helper function to calculate a folder's selection state based on its children.
pub fn calculate_folder_selection_state(folder_node: &FileTreeNode) -> NodeSelectionState {
    if folder_node.node_type == TreeNodeType::File {
        // This function is intended for folders. If called on a file,
        // it should ideally not happen, but return its own state as a fallback.
        return *folder_node.selection_state.read();
    }

    if folder_node.children.is_empty() {
        // An empty folder has no selected children, so it's NotSelected.
        return NodeSelectionState::NotSelected;
    }

    let mut all_children_selected = true;
    let mut any_child_selected = false;
    let mut any_child_partially_selected = false;

    for child in &folder_node.children {
        let child_state = *child.selection_state.read();
        match child_state {
            NodeSelectionState::Selected => {
                any_child_selected = true;
                // If even one child is not fully selected, the parent cannot be fully selected.
            }
            NodeSelectionState::NotSelected => {
                all_children_selected = false;
            }
            NodeSelectionState::PartiallySelected => {
                all_children_selected = false;
                any_child_partially_selected = true;
            }
        }
    }

    if all_children_selected {
        NodeSelectionState::Selected
    } else if any_child_selected || any_child_partially_selected {
        NodeSelectionState::PartiallySelected
    } else {
        NodeSelectionState::NotSelected
    }
}

// Task 8.2 (Helper): Recursively update folder selection states from bottom up.
// Making it pub(crate) for testing.
pub(crate) fn update_folder_selection_states_recursive(nodes: &[FileTreeNode]) {
    for node in nodes {
        if node.node_type == TreeNodeType::Folder {
            // First, ensure all children (and their descendants) are up-to-date.
            // This makes it a bottom-up update for the current node.
            if !node.children.is_empty() {
                update_folder_selection_states_recursive(&node.children);
            }

            // Now calculate this folder's state based on its (now updated) children.
            let new_calculated_state = calculate_folder_selection_state(node);

            // Only update the signal if the state has actually changed.
            // Signal::set takes &mut self, so the signal variable needs to be mutable.
            let mut selection_signal = node.selection_state;
            if *selection_signal.read() != new_calculated_state {
                selection_signal.set(new_calculated_state);
            }
        }
    }
}

// --- End of Story 8 Implementation ---

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

    let all_files_clone_for_buttons = props.all_files.clone();
    let selected_paths_for_buttons = props.selected_paths;

    use_effect(move || {
        // Correctly obtain scope_id for convert_blueprint_to_file_tree_node_recursive
        let effect_scope_id = current_scope_id().expect("use_effect must have a scope_id");

        let blueprints = build_tree_from_file_info(&props.all_files, &props.selected_paths.read());

        let new_display_nodes: Vec<FileTreeNode> = blueprints
            .into_iter()
            .map(|b| convert_blueprint_to_file_tree_node_recursive(b, effect_scope_id)) // Pass correct scope_id
            .collect();

        update_folder_selection_states_recursive(&new_display_nodes);

        tree_display_nodes.set(new_display_nodes);
    });

    rsx! {
        div {
            class: "file-tree-container",
            div {
                class: "file-tree-controls p-2 flex space-x-2",
                button {
                    class: "px-3 py-1 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-offset-gray-800",
                    onclick: move |_| {
                        let mut all_file_paths_hs = HashSet::new();
                        // Use the cloned all_files specific to this button
                        for file_info in &all_files_clone_for_buttons { // Use the clone
                            all_file_paths_hs.insert(file_info.path.clone());
                        }
                        // Signal is Copy, get a mutable copy for set()
                        let mut sp = selected_paths_for_buttons; // Use the copied signal
                        sp.set(all_file_paths_hs);
                    },
                    "Select All"
                }
                button {
                    class: "px-3 py-1 text-sm font-medium text-gray-700 bg-gray-200 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400 dark:text-gray-200 dark:bg-gray-600 dark:hover:bg-gray-500 dark:focus:ring-offset-gray-800",
                    onclick: move |_| {
                        // Signal is Copy, get a mutable copy for set()
                        let mut sp = selected_paths_for_buttons; // Use the copied signal
                        sp.set(HashSet::new());
                    },
                    "Deselect All"
                }
            }
            ul {
                class: "file-tree-list p-0 m-0 list-none",
                for node in tree_display_nodes.read().iter() {
                    FileTreeNodeDisplay {
                        key: "{node.id}",
                        node: node.clone(),
                        selected_paths: selected_paths_for_buttons, // Pass the copied signal to children if they also modify it, or original props.selected_paths
                                                                  // For now, FileTreeNodeDisplay takes Signal<HashSet<PathBuf>>, so selected_paths_for_buttons is fine.
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

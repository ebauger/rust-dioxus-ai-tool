use crate::fs_utils::FileInfo;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use log;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
    workspace_root: &Path,
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
        let absolute_file_path = &file_info.path;

        // Determine path components relative to the workspace root for tree structure
        let path_relative_to_workspace = match absolute_file_path.strip_prefix(workspace_root) {
            Ok(p) if p.components().next().is_some() => p.to_path_buf(), // Ensure it's not empty
            _ => {
                // If not under workspace or is the workspace root itself (as a file),
                // or strip_prefix results in an empty path.
                // Fallback: use only the file name as a single top-level node.
                // This makes it appear directly under an implicit root.
                PathBuf::from(file_info.name.clone())
            }
        };

        let mut current_parent_children_list = &mut final_roots;
        // This will track the absolute path for folders being created.
        // For files, we'll use absolute_file_path directly.
        let mut accumulated_absolute_folder_path = workspace_root.to_path_buf();

        let structural_components: Vec<_> = path_relative_to_workspace.components().collect();

        // If structural_components is empty (e.g. workspace_root is a file and file_info.path is that file)
        // this loop won't run, which is fine. The fallback above handles creating a node from file_info.name.
        // However, if the fallback results in a path with components (e.g. "file.txt"), this loop will run.

        let num_structural_components = structural_components.len();

        for (idx, component_wrapper) in structural_components.iter().enumerate() {
            let component_os_str = component_wrapper.as_os_str();
            let component_name = component_os_str.to_string_lossy().into_owned();

            // For folders, build their absolute path piece by piece from workspace_root
            // For the file itself (last component), we'll use its original absolute_file_path
            if idx < num_structural_components - 1 {
                // It's a folder component
                accumulated_absolute_folder_path.push(component_os_str);
            }

            let is_last_component = idx == num_structural_components - 1;

            if is_last_component {
                // It's a file node based on the original file_info
                // Check if it already exists (e.g. if fallback created single component for it)
                if !current_parent_children_list
                    .iter()
                    .any(|n| n.path == *absolute_file_path && n.node_type == TreeNodeType::File)
                {
                    let id = unique_id_counter;
                    unique_id_counter += 1;
                    let selection = if selected_paths.contains(absolute_file_path) {
                        NodeSelectionState::Selected
                    } else {
                        NodeSelectionState::NotSelected
                    };
                    let file_node = FileTreeNodeBlueprint {
                        id,
                        // Use component_name for consistency if path_relative_to_workspace was just the filename
                        // Or, use file_info.name if preferred. Let's use component_name for now.
                        name: component_name.clone(), // component_name is from the structural component
                        path: absolute_file_path.clone(), // Crucially, store the original absolute path
                        node_type: TreeNodeType::File,
                        children: Vec::new(),
                        is_expanded: false,
                        selection_state: selection,
                        depth: idx, // Depth is based on iteration over relative components
                    };
                    current_parent_children_list.push(file_node);
                }
            } else {
                // It's an intermediate folder
                let folder_path_for_blueprint = accumulated_absolute_folder_path.clone();
                let folder_node = find_or_create_blueprint_node(
                    current_parent_children_list,
                    &component_name,
                    &folder_path_for_blueprint, // This is the absolute path for the folder
                    TreeNodeType::Folder,
                    &mut unique_id_counter,
                    idx,      // Depth relative to workspace root
                    idx == 0, // is_expanded for top-level folders relative to workspace
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
    // First, convert all children blueprints into actual FileTreeNodes.
    // These children will have their selection_state signals correctly initialized
    // because this function determines state bottom-up.
    let children_nodes: Vec<FileTreeNode> = blueprint
        .children
        .into_iter()
        .map(|b| convert_blueprint_to_file_tree_node_recursive(b, scope_id))
        .collect();

    // Now, determine the selection_state for the current node.
    let current_node_selection_state: NodeSelectionState;
    if blueprint.node_type == TreeNodeType::File {
        // For files, the blueprint's selection_state is authoritative (derived from selected_paths).
        current_node_selection_state = blueprint.selection_state;
    } else {
        // It's a Folder
        if children_nodes.is_empty() {
            // An empty folder has no selected children, so it's NotSelected.
            // Or, if folders themselves can be selected, this might need parent's `selected_paths` check.
            // For now, assuming empty folders are NotSelected unless `selected_paths` implies otherwise
            // (which build_tree_from_file_info doesn't currently do for folders directly).
            current_node_selection_state = NodeSelectionState::NotSelected;
        } else {
            let mut all_children_selected = true;
            let mut any_child_selected = false;
            let mut any_child_partially_selected = false;

            for child_node in &children_nodes {
                // Iterate over the newly created child FileTreeNodes
                let child_state = *child_node.selection_state.read(); // Read from the child's signal
                match child_state {
                    NodeSelectionState::Selected => {
                        any_child_selected = true;
                        // all_children_selected remains true unless a non-selected child is found
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
                current_node_selection_state = NodeSelectionState::Selected;
            } else if any_child_selected || any_child_partially_selected {
                current_node_selection_state = NodeSelectionState::PartiallySelected;
            } else {
                current_node_selection_state = NodeSelectionState::NotSelected;
            }
        }
    }

    FileTreeNode {
        id: blueprint.id,
        name: blueprint.name,
        path: blueprint.path,
        node_type: blueprint.node_type,
        children: children_nodes, // Use the converted children
        is_expanded: Signal::new_in_scope(blueprint.is_expanded, scope_id),
        // Initialize the signal directly with the calculated state.
        selection_state: Signal::new_in_scope(current_node_selection_state, scope_id),
        depth: blueprint.depth,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FileTreeProps {
    pub all_files: Vec<FileInfo>,
    pub selected_paths: Signal<HashSet<PathBuf>>,
    pub on_select_all: EventHandler<()>,
    pub on_deselect_all: EventHandler<()>,
    pub workspace_root: PathBuf,
}

#[allow(non_snake_case)]
pub fn FileTree(props: FileTreeProps) -> Element {
    let all_files_clone_for_buttons = props.all_files.clone();
    let selected_paths_for_buttons = props.selected_paths;

    let tree_nodes_memo = use_memo(move || {
        let current_scope_id =
            current_scope_id().expect("use_memo running outside of a Dioxus scope");

        // Explicitly use the captured props fields for clarity and to ensure
        // the memo is sensitive to their changes when the component re-renders.
        let current_all_files = &props.all_files;
        let current_workspace_root = &props.workspace_root;
        let current_selected_paths = props.selected_paths.read().clone(); // Signal read

        log::debug!(
            "FileTree use_memo: Recomputing tree_nodes. Files: {}, Selected: {}, WS Root: {}",
            current_all_files.len(),
            current_selected_paths.len(),
            current_workspace_root.display()
        );

        // Initial tree construction
        let new_tree_blueprints = build_tree_from_file_info(
            current_all_files,       // Use the reference from captured props
            &current_selected_paths, // Use the cloned signal value
            current_workspace_root,  // Use the reference from captured props
        );

        // Convert blueprints to FileTreeNodes
        let new_tree_nodes: Vec<FileTreeNode> = new_tree_blueprints
            .into_iter()
            .map(|bp| convert_blueprint_to_file_tree_node_recursive(bp, current_scope_id))
            .collect();

        new_tree_nodes // This Vec<FileTreeNode> is the value of the memo
    });

    rsx! {
        div {
            class: "file-tree-container",
            div {
                class: "file-tree-controls p-2 flex space-x-2 border-b border-light-border",
                button {
                    class: "px-3 py-1 text-sm font-medium text-white bg-light-primary rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500",
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
                    class: "px-3 py-1 text-sm font-medium text-light-foreground bg-gray-200 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400",
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
                for node in tree_nodes_memo.read().iter() {
                    FileTreeNodeDisplay {
                        key: "{node.id}",
                        node: node.clone(),
                        selected_paths: selected_paths_for_buttons,
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
                class: "node-row flex items-center hover:bg-gray-100 p-1 rounded",
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
                    class: "mr-2 form-checkbox rounded text-blue-500 focus:ring-blue-500 bg-light-background border-light-border",
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

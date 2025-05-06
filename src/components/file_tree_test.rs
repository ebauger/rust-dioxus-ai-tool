#![cfg(test)]
use crate::components::file_tree::{
    build_tree_from_file_info, FileTreeNodeBlueprint, NodeSelectionState, TreeNodeType,
};
use crate::fs_utils::FileInfo;
use dioxus::prelude::*;
use dioxus_core;
use dioxus_core::ScopeId;
use std::collections::HashSet;
use std::path::PathBuf;

fn create_file_info(path_str: &str) -> FileInfo {
    let path = PathBuf::from(path_str);
    FileInfo {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        path,
        size: 0,        // Not relevant for tree structure
        token_count: 0, // Not relevant for tree structure
    }
}

#[test]
fn test_build_tree_empty() {
    let files = Vec::new();
    let selected_paths = HashSet::new();
    let tree = build_tree_from_file_info(&files, &selected_paths);
    assert!(tree.is_empty(), "Tree should be empty for no input files");
}

#[test]
fn test_build_tree_flat_list() {
    let files = vec![
        create_file_info("file1.txt"),
        create_file_info("file2.rs"),
        create_file_info("file3.md"),
    ];
    let mut selected_paths = HashSet::new();
    selected_paths.insert(PathBuf::from("file2.rs"));

    let tree = build_tree_from_file_info(&files, &selected_paths);

    assert_eq!(tree.len(), 3, "Should have 3 root nodes");

    // file1.txt
    let node1 = tree
        .iter()
        .find(|n| n.name == "file1.txt")
        .expect("file1.txt not found");
    assert_eq!(node1.path, PathBuf::from("file1.txt"));
    assert_eq!(node1.node_type, TreeNodeType::File);
    assert_eq!(node1.depth, 0);
    assert!(!node1.is_expanded, "File node should not be expanded");
    assert_eq!(node1.selection_state, NodeSelectionState::NotSelected);
    assert!(node1.children.is_empty());

    // file2.rs (selected)
    let node2 = tree
        .iter()
        .find(|n| n.name == "file2.rs")
        .expect("file2.rs not found");
    assert_eq!(node2.path, PathBuf::from("file2.rs"));
    assert_eq!(node2.node_type, TreeNodeType::File);
    assert_eq!(node2.depth, 0);
    assert_eq!(node2.selection_state, NodeSelectionState::Selected);
    assert!(node2.children.is_empty());

    // file3.md
    let node3 = tree
        .iter()
        .find(|n| n.name == "file3.md")
        .expect("file3.md not found");
    assert_eq!(node3.path, PathBuf::from("file3.md"));
    assert_eq!(node3.node_type, TreeNodeType::File);
    assert_eq!(node3.depth, 0);
    assert_eq!(node3.selection_state, NodeSelectionState::NotSelected);
    assert!(node3.children.is_empty());

    // Check IDs are unique (simple check, assumes they are 0, 1, 2 in some order)
    let mut ids = tree.iter().map(|n| n.id).collect::<Vec<_>>();
    ids.sort();
    assert_eq!(
        ids,
        vec![0, 1, 2],
        "IDs should be unique and sequential for flat list"
    );
}

#[test]
fn test_build_tree_nested_structure() {
    let files = vec![
        create_file_info("src/main.rs"),
        create_file_info("src/components/button.rs"),
        create_file_info("README.md"),
        create_file_info("src/components/mod.rs"),
    ];
    let mut selected_paths = HashSet::new();
    selected_paths.insert(PathBuf::from("src/main.rs"));
    selected_paths.insert(PathBuf::from("src/components/button.rs"));

    // Expected structure:
    // - README.md (depth 0)
    // - src (depth 0)
    //   - main.rs (depth 1, selected)
    //   - components (depth 1)
    //     - button.rs (depth 2, selected)
    //     - mod.rs (depth 2)

    let tree = build_tree_from_file_info(&files, &selected_paths);

    assert_eq!(tree.len(), 2, "Should have 2 root nodes (README.md, src)");

    // README.md
    let readme_node = tree
        .iter()
        .find(|n| n.name == "README.md")
        .expect("README.md not found");
    assert_eq!(readme_node.node_type, TreeNodeType::File);
    assert_eq!(readme_node.path, PathBuf::from("README.md"));
    assert_eq!(readme_node.depth, 0);
    assert!(!readme_node.is_expanded);
    assert_eq!(readme_node.selection_state, NodeSelectionState::NotSelected);
    assert!(readme_node.children.is_empty());

    // src folder
    let src_node = tree
        .iter()
        .find(|n| n.name == "src" && n.node_type == TreeNodeType::Folder)
        .expect("src folder not found");
    assert_eq!(src_node.path, PathBuf::from("src"));
    assert_eq!(src_node.depth, 0);
    assert!(
        src_node.is_expanded,
        "Root folder 'src' should be expanded by default"
    );
    // Folder selection state will be handled later, for now it's NotSelected
    assert_eq!(src_node.selection_state, NodeSelectionState::NotSelected);
    assert_eq!(
        src_node.children.len(),
        2,
        "'src' folder should have 2 children (main.rs, components)"
    );

    // src/main.rs
    let main_rs_node = src_node
        .children
        .iter()
        .find(|n| n.name == "main.rs")
        .expect("src/main.rs not found");
    assert_eq!(main_rs_node.node_type, TreeNodeType::File);
    assert_eq!(main_rs_node.path, PathBuf::from("src/main.rs"));
    assert_eq!(main_rs_node.depth, 1);
    assert!(!main_rs_node.is_expanded);
    assert_eq!(main_rs_node.selection_state, NodeSelectionState::Selected);
    assert!(main_rs_node.children.is_empty());

    // src/components folder
    let components_node = src_node
        .children
        .iter()
        .find(|n| n.name == "components" && n.node_type == TreeNodeType::Folder)
        .expect("src/components folder not found");
    assert_eq!(components_node.path, PathBuf::from("src/components"));
    assert_eq!(components_node.depth, 1);
    // Non-root folders are collapsed by default
    assert!(
        !components_node.is_expanded,
        "Folder 'src/components' should be collapsed by default"
    );
    assert_eq!(
        components_node.selection_state,
        NodeSelectionState::NotSelected
    );
    assert_eq!(
        components_node.children.len(),
        2,
        "'components' folder should have 2 children (button.rs, mod.rs)"
    );

    // src/components/button.rs
    let button_rs_node = components_node
        .children
        .iter()
        .find(|n| n.name == "button.rs")
        .expect("src/components/button.rs not found");
    assert_eq!(button_rs_node.node_type, TreeNodeType::File);
    assert_eq!(
        button_rs_node.path,
        PathBuf::from("src/components/button.rs")
    );
    assert_eq!(button_rs_node.depth, 2);
    assert_eq!(button_rs_node.selection_state, NodeSelectionState::Selected);
    assert!(button_rs_node.children.is_empty());

    // src/components/mod.rs
    let mod_rs_node = components_node
        .children
        .iter()
        .find(|n| n.name == "mod.rs")
        .expect("src/components/mod.rs not found");
    assert_eq!(mod_rs_node.node_type, TreeNodeType::File);
    assert_eq!(mod_rs_node.path, PathBuf::from("src/components/mod.rs"));
    assert_eq!(mod_rs_node.depth, 2);
    assert_eq!(mod_rs_node.selection_state, NodeSelectionState::NotSelected);
    assert!(mod_rs_node.children.is_empty());

    // Check IDs are unique across the tree
    let mut ids = Vec::new();
    collect_ids(&tree, &mut ids);
    ids.sort();
    let expected_ids: Vec<usize> = (0..ids.len()).collect(); // Expect 0, 1, 2, 3, 4, 5
    assert_eq!(
        ids, expected_ids,
        "IDs should be unique and sequential across the tree"
    );
}

#[test]
fn test_build_tree_paths_with_common_prefix_but_different_roots() {
    let files = vec![
        create_file_info("project_a/src/main.rs"),
        create_file_info("project_b/src/main.rs"),
    ];
    let selected_paths = HashSet::new();
    let tree = build_tree_from_file_info(&files, &selected_paths);

    assert_eq!(
        tree.len(),
        2,
        "Should have two root folders: project_a and project_b"
    );

    let project_a_node = tree
        .iter()
        .find(|n| n.name == "project_a")
        .expect("project_a not found");
    assert_eq!(project_a_node.depth, 0);
    assert_eq!(project_a_node.node_type, TreeNodeType::Folder);
    assert!(project_a_node.is_expanded);
    assert_eq!(project_a_node.children.len(), 1); // src

    let project_b_node = tree
        .iter()
        .find(|n| n.name == "project_b")
        .expect("project_b not found");
    assert_eq!(project_b_node.depth, 0);
    assert_eq!(project_b_node.node_type, TreeNodeType::Folder);
    assert!(project_b_node.is_expanded);
    assert_eq!(project_b_node.children.len(), 1); // src
}

#[test]
fn test_build_tree_file_and_folder_same_name_at_root() {
    // Edge case: a file "foo" and a folder "foo" at the same level (root)
    let files_data = vec![
        create_file_info("foo"),         // File "foo"
        create_file_info("foo/bar.txt"), // File "foo/bar.txt", implies folder "foo"
    ];
    let selected_paths = HashSet::new();
    let tree = build_tree_from_file_info(&files_data, &selected_paths);

    // Current behavior: creates two root nodes: File("foo") and Folder("foo")
    assert_eq!(
        tree.len(),
        2,
        "Should have File('foo') and Folder('foo') at root"
    );

    let foo_file_node = tree
        .iter()
        .find(|n| n.name == "foo" && n.node_type == TreeNodeType::File)
        .expect("File 'foo' not found at root");
    assert_eq!(foo_file_node.depth, 0);

    let foo_folder_node = tree
        .iter()
        .find(|n| n.name == "foo" && n.node_type == TreeNodeType::Folder)
        .expect("Folder 'foo' not found at root");
    assert_eq!(foo_folder_node.depth, 0);
    assert_eq!(
        foo_folder_node.children.len(),
        1,
        "Folder 'foo' should contain 'bar.txt'"
    );
    assert!(foo_folder_node.is_expanded);

    let bar_txt_node = foo_folder_node
        .children
        .iter()
        .find(|n| n.name == "bar.txt")
        .expect("bar.txt not found");
    assert_eq!(bar_txt_node.node_type, TreeNodeType::File);
    assert_eq!(bar_txt_node.depth, 1);
}

#[test]
fn test_build_tree_id_uniqueness_and_order() {
    let files = vec![
        create_file_info("b/c.txt"), // folder b, file c.txt
        create_file_info("a.txt"),   // file a.txt
    ];
    let selected_paths = HashSet::new();
    let tree = build_tree_from_file_info(&files, &selected_paths);

    // Expected:
    // a.txt (id=0, depth=0)
    // b (id=1, depth=0)
    //   c.txt (id=2, depth=1)
    // Order of roots might vary due to HashMap nature if paths weren't sorted, but sorting `files` helps.
    // The final_roots are populated by iterating through sorted_files.
    // So, "a.txt" related nodes should get ids before "b/c.txt" related nodes IF "a.txt" path < "b/c.txt" path.
    // PathBuf::from("a.txt") < PathBuf::from("b") is true.

    let mut ids = Vec::new();
    collect_ids(&tree, &mut ids);
    ids.sort(); // Sort to ensure consistent comparison regardless of internal HashMap iteration order

    let mut names_and_depths = Vec::new();
    collect_names_and_depths(&tree, &mut names_and_depths);

    // Verify structure and IDs
    // Since files are sorted by path before processing:
    // 1. "a.txt" is processed.
    //    - Node "a.txt" (File, depth 0, id 0)
    // 2. "b/c.txt" is processed.
    //    - Node "b" (Folder, depth 0, id 1)
    //    - Node "c.txt" (File, depth 1, id 2)

    assert_eq!(ids, vec![0, 1, 2], "IDs should be 0, 1, 2 after sorting");

    let a_txt_node = tree
        .iter()
        .find(|n| n.name == "a.txt")
        .expect("a.txt not found");
    assert_eq!(a_txt_node.id, 0);
    assert_eq!(a_txt_node.depth, 0);

    let b_folder_node = tree
        .iter()
        .find(|n| n.name == "b" && n.node_type == TreeNodeType::Folder)
        .expect("folder b not found");
    assert_eq!(b_folder_node.id, 1);
    assert_eq!(b_folder_node.depth, 0);
    assert_eq!(b_folder_node.children.len(), 1);

    let c_txt_node = b_folder_node.children.first().expect("c.txt not found");
    assert_eq!(c_txt_node.name, "c.txt");
    assert_eq!(c_txt_node.id, 2);
    assert_eq!(c_txt_node.depth, 1);
}

// Helper to collect all node IDs from a tree for uniqueness checks
fn collect_ids(nodes: &[FileTreeNodeBlueprint], ids: &mut Vec<usize>) {
    for node in nodes {
        ids.push(node.id);
        collect_ids(&node.children, ids);
    }
}

// Helper to collect names and depths for structure checks
fn collect_names_and_depths(
    nodes: &[FileTreeNodeBlueprint],
    names_depths: &mut Vec<(String, usize)>,
) {
    for node in nodes {
        names_depths.push((node.name.clone(), node.depth));
        if !node.children.is_empty() {
            collect_names_and_depths(&node.children, names_depths);
        }
    }
}

// --- Tests for Story 10 ---
#[cfg(test)]
mod story_10_tests {
    use dioxus::prelude::*;
    use std::{collections::HashSet, path::PathBuf};

    use crate::components::file_tree::{
        build_tree_from_file_info, calculate_folder_selection_state,
        convert_blueprint_to_file_tree_node_recursive, update_folder_selection_states_recursive,
        FileTreeNode, NodeSelectionState, TreeNodeType,
    };
    use crate::fs_utils::FileInfo;

    // Helper to create a FileTreeNode (File type) for testing
    fn create_test_file_node(
        scope_id: ScopeId,
        id: usize,
        name: &str,
        path_str: &str,
        selection_state: NodeSelectionState,
        depth: usize,
    ) -> FileTreeNode {
        FileTreeNode {
            id,
            name: name.to_string(),
            path: PathBuf::from(path_str),
            node_type: TreeNodeType::File,
            children: Vec::new(),
            is_expanded: Signal::new_in_scope(false, scope_id),
            selection_state: Signal::new_in_scope(selection_state, scope_id),
            depth,
        }
    }

    // Helper to create a FileTreeNode (Folder type) for testing
    fn create_test_folder_node(
        scope_id: ScopeId,
        id: usize,
        name: &str,
        path_str: &str,
        children: Vec<FileTreeNode>,
        is_expanded: bool,
        selection_state: NodeSelectionState,
        depth: usize,
    ) -> FileTreeNode {
        FileTreeNode {
            id,
            name: name.to_string(),
            path: PathBuf::from(path_str),
            node_type: TreeNodeType::Folder,
            children,
            is_expanded: Signal::new_in_scope(is_expanded, scope_id),
            selection_state: Signal::new_in_scope(selection_state, scope_id),
            depth,
        }
    }

    #[test]
    fn test_collect_all_file_paths_recursive() {
        #[allow(non_snake_case)]
        fn TestComponent() -> Element {
            let scope_id = current_scope_id()
                .expect("TestComponent must run in a Dioxus scope to get scope_id");

            let file1_path = PathBuf::from("dir1/file1.txt");
            let file2_path = PathBuf::from("dir1/subdir1/file2.txt");
            let file3_path = PathBuf::from("file3.txt");

            let file1 = create_test_file_node(
                scope_id,
                0,
                "file1.txt",
                "dir1/file1.txt",
                NodeSelectionState::NotSelected,
                1,
            );
            let file2 = create_test_file_node(
                scope_id,
                1,
                "file2.txt",
                "dir1/subdir1/file2.txt",
                NodeSelectionState::NotSelected,
                2,
            );
            let file3 = create_test_file_node(
                scope_id,
                3,
                "file3.txt",
                "file3.txt",
                NodeSelectionState::NotSelected,
                0,
            );

            // Test on a single file node
            let collected_paths_file1 = file1.collect_all_file_paths_recursive();
            assert_eq!(collected_paths_file1.len(), 1);
            assert!(collected_paths_file1.contains(&file1_path));

            // Test on an empty folder
            let empty_folder = create_test_folder_node(
                scope_id,
                10,
                "empty_dir",
                "empty_dir",
                vec![],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert!(empty_folder.collect_all_file_paths_recursive().is_empty());

            // Test on a folder with only files
            let folder_with_files = create_test_folder_node(
                scope_id,
                20,
                "dir_files_only",
                "dir_files_only",
                vec![file1.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            let collected_folder_files_only = folder_with_files.collect_all_file_paths_recursive();
            assert_eq!(collected_folder_files_only.len(), 1);
            assert!(collected_folder_files_only.contains(&file1_path));

            // Test on a nested structure
            let subdir1 = create_test_folder_node(
                scope_id,
                30,
                "subdir1",
                "dir1/subdir1",
                vec![file2.clone()],
                false,
                NodeSelectionState::NotSelected,
                1,
            );
            let dir1 = create_test_folder_node(
                scope_id,
                40,
                "dir1",
                "dir1",
                vec![file1.clone(), subdir1],
                false,
                NodeSelectionState::NotSelected,
                0,
            );

            let root_node_list = vec![dir1.clone(), file3.clone()]; // Simulating a root level structure

            // Collect from dir1
            let collected_dir1 = dir1.collect_all_file_paths_recursive();
            assert_eq!(
                collected_dir1.len(),
                2,
                "Dir1 should contain file1.txt and file2.txt"
            );
            assert!(collected_dir1.contains(&file1_path));
            assert!(collected_dir1.contains(&file2_path));

            // If we had a "root" conceptual folder containing dir1 and file3
            let mut all_paths_from_root_structure = Vec::new();
            for node in root_node_list {
                all_paths_from_root_structure.extend(node.collect_all_file_paths_recursive());
            }
            all_paths_from_root_structure.sort();
            let mut expected_paths =
                vec![file1_path.clone(), file2_path.clone(), file3_path.clone()];
            expected_paths.sort();
            assert_eq!(
                all_paths_from_root_structure, expected_paths,
                "All paths from structure should be collected"
            );

            rsx! { div {} } // Dummy element
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }

    #[test]
    fn test_calculate_folder_selection_state() {
        #[allow(non_snake_case)]
        fn TestComponent() -> Element {
            let scope_id = current_scope_id()
                .expect("TestComponent must run in a Dioxus scope to get scope_id");

            let empty_folder = create_test_folder_node(
                scope_id,
                0,
                "empty",
                "empty",
                vec![],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&empty_folder),
                NodeSelectionState::NotSelected
            );

            // All children selected
            let file1_sel =
                create_test_file_node(scope_id, 1, "f1", "f1", NodeSelectionState::Selected, 1);
            let file2_sel =
                create_test_file_node(scope_id, 2, "f2", "f2", NodeSelectionState::Selected, 1);
            let folder_all_sel = create_test_folder_node(
                scope_id,
                3,
                "all_sel",
                "all_sel",
                vec![file1_sel.clone(), file2_sel.clone()],
                false,
                NodeSelectionState::NotSelected, // Initial state of folder itself, calculation is tested
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&folder_all_sel),
                NodeSelectionState::Selected
            );

            // All children not selected
            let file1_not_sel = create_test_file_node(
                scope_id,
                4,
                "f1ns",
                "f1ns",
                NodeSelectionState::NotSelected,
                1,
            );
            let file2_not_sel = create_test_file_node(
                scope_id,
                5,
                "f2ns",
                "f2ns",
                NodeSelectionState::NotSelected,
                1,
            );
            let folder_all_not_sel = create_test_folder_node(
                scope_id,
                6,
                "all_not_sel",
                "all_not_sel",
                vec![file1_not_sel.clone(), file2_not_sel.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&folder_all_not_sel),
                NodeSelectionState::NotSelected
            );

            // Mixed: some selected, some not selected (direct children files)
            let folder_mixed_direct = create_test_folder_node(
                scope_id,
                7,
                "mixed_direct",
                "mixed_direct",
                vec![file1_sel.clone(), file2_not_sel.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&folder_mixed_direct),
                NodeSelectionState::PartiallySelected
            );

            // Child is PartiallySelected folder
            let subfolder_partial_child_file_sel = create_test_file_node(
                scope_id,
                8,
                "sf_f_sel",
                "sf_f_sel",
                NodeSelectionState::Selected,
                2,
            );
            let subfolder_partial_child_file_not_sel = create_test_file_node(
                scope_id,
                9,
                "sf_f_not_sel",
                "sf_f_not_sel",
                NodeSelectionState::NotSelected,
                2,
            );
            // For this subfolder, its *own* selection_state signal must be correctly pre-set for the test.
            // The create_test_folder_node initializes its selection_state Signal based on the passed argument.
            // So, if we want to test how calculate_folder_selection_state behaves with a child that *is* PartiallySelected,
            // we must ensure that child node's Signal<NodeSelectionState> *reads* as PartiallySelected.
            let subfolder_partial = create_test_folder_node(
                scope_id,
                10,
                "sub_partial",
                "sub_partial",
                vec![
                    subfolder_partial_child_file_sel, // These define what subfolder_partial *would* calculate to
                    subfolder_partial_child_file_not_sel,
                ],
                false,
                NodeSelectionState::PartiallySelected, // We explicitly set its Signal to this state
                1,
            );

            let folder_with_partial_sub = create_test_folder_node(
                scope_id,
                11,
                "with_partial_sub",
                "with_partial_sub",
                vec![subfolder_partial.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&folder_with_partial_sub),
                NodeSelectionState::PartiallySelected
            );

            // Mixed: one child folder Selected, one child file NotSelected
            let subfolder_all_sel_child1 = create_test_file_node(
                scope_id,
                12,
                "sf_all_c1",
                "sf_all_c1",
                NodeSelectionState::Selected,
                2,
            );
            let subfolder_all_sel_child2 = create_test_file_node(
                scope_id,
                13,
                "sf_all_c2",
                "sf_all_c2",
                NodeSelectionState::Selected,
                2,
            );
            let subfolder_fully_sel = create_test_folder_node(
                scope_id,
                14,
                "sub_full_sel",
                "sub_full_sel",
                vec![subfolder_all_sel_child1, subfolder_all_sel_child2],
                false,
                NodeSelectionState::Selected, // Explicitly set its Signal to Selected
                1,
            );

            let folder_mixed_types = create_test_folder_node(
                scope_id,
                15,
                "mixed_types",
                "mixed_types",
                vec![subfolder_fully_sel.clone(), file2_not_sel.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&folder_mixed_types),
                NodeSelectionState::PartiallySelected
            );

            // All children (files and folders) are selected
            let folder_all_children_types_sel = create_test_folder_node(
                scope_id,
                16,
                "all_children_types_sel",
                "all_children_types_sel",
                vec![subfolder_fully_sel.clone(), file1_sel.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            assert_eq!(
                calculate_folder_selection_state(&folder_all_children_types_sel),
                NodeSelectionState::Selected
            );

            // File node passed to calculate_folder_selection_state (should return its own state)
            assert_eq!(
                calculate_folder_selection_state(&file1_sel),
                NodeSelectionState::Selected
            );

            rsx! { div {} } // Dummy element
        }
        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }

    fn create_file_info_for_test(path_str: &str) -> FileInfo {
        let path = PathBuf::from(path_str);
        FileInfo {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            path,
            size: 0,
            token_count: 0,
        }
    }

    // Helper to find a node by path in a Vec<FileTreeNode>
    fn find_node_by_path(nodes: &[FileTreeNode], path_to_find: &PathBuf) -> Option<FileTreeNode> {
        for node in nodes {
            if node.path == *path_to_find {
                return Some(node.clone());
            }
            if !node.children.is_empty() {
                if let Some(found_child) = find_node_by_path(&node.children, path_to_find) {
                    return Some(found_child);
                }
            }
        }
        None
    }

    #[test]
    fn test_selection_state_logic_full_pipeline() {
        #[allow(non_snake_case)]
        fn TestComponent() -> Element {
            let scope_id = current_scope_id().expect("TestComponent must run in a Dioxus scope");

            let all_files = vec![
                create_file_info_for_test("src/main.rs"),
                create_file_info_for_test("src/components/button.rs"),
                create_file_info_for_test("src/components/mod.rs"),
                create_file_info_for_test("README.md"),
            ];

            // Scenario 1: Select src/components/button.rs
            let mut selected_paths_set = HashSet::new();
            selected_paths_set.insert(PathBuf::from("src/components/button.rs"));

            let blueprints = build_tree_from_file_info(&all_files, &selected_paths_set);
            let mut tree_nodes: Vec<FileTreeNode> = blueprints
                .into_iter()
                .map(|b| convert_blueprint_to_file_tree_node_recursive(b, scope_id))
                .collect();
            update_folder_selection_states_recursive(&mut tree_nodes);

            // Assertions for Scenario 1
            let button_rs =
                find_node_by_path(&tree_nodes, &PathBuf::from("src/components/button.rs"))
                    .expect("button.rs not found");
            assert_eq!(
                *button_rs.selection_state.read(),
                NodeSelectionState::Selected,
                "button.rs should be Selected"
            );

            let components_folder =
                find_node_by_path(&tree_nodes, &PathBuf::from("src/components"))
                    .expect("components folder not found");
            assert_eq!(
                *components_folder.selection_state.read(),
                NodeSelectionState::PartiallySelected,
                "components folder should be PartiallySelected"
            );

            let src_folder = find_node_by_path(&tree_nodes, &PathBuf::from("src"))
                .expect("src folder not found");
            assert_eq!(
                *src_folder.selection_state.read(),
                NodeSelectionState::PartiallySelected,
                "src folder should be PartiallySelected"
            );

            let readme = find_node_by_path(&tree_nodes, &PathBuf::from("README.md"))
                .expect("README.md not found");
            assert_eq!(
                *readme.selection_state.read(),
                NodeSelectionState::NotSelected,
                "README.md should be NotSelected"
            );

            // Scenario 2: Select all files in src/components/
            selected_paths_set.clear();
            selected_paths_set.insert(PathBuf::from("src/components/button.rs"));
            selected_paths_set.insert(PathBuf::from("src/components/mod.rs"));

            let blueprints_2 = build_tree_from_file_info(&all_files, &selected_paths_set);
            let mut tree_nodes_2: Vec<FileTreeNode> = blueprints_2
                .into_iter()
                .map(|b| convert_blueprint_to_file_tree_node_recursive(b, scope_id))
                .collect();
            update_folder_selection_states_recursive(&mut tree_nodes_2);

            // Assertions for Scenario 2
            let components_folder_2 =
                find_node_by_path(&tree_nodes_2, &PathBuf::from("src/components"))
                    .expect("components folder S2 not found");
            assert_eq!(
                *components_folder_2.selection_state.read(),
                NodeSelectionState::Selected,
                "components folder S2 should be Selected"
            );

            let src_folder_2 = find_node_by_path(&tree_nodes_2, &PathBuf::from("src"))
                .expect("src folder S2 not found");
            assert_eq!(
                *src_folder_2.selection_state.read(),
                NodeSelectionState::PartiallySelected,
                "src folder S2 should be PartiallySelected (main.rs not selected)"
            );

            // Scenario 3: Select all files
            selected_paths_set.clear();
            for file_info in &all_files {
                selected_paths_set.insert(file_info.path.clone());
            }

            let blueprints_3 = build_tree_from_file_info(&all_files, &selected_paths_set);
            let mut tree_nodes_3: Vec<FileTreeNode> = blueprints_3
                .into_iter()
                .map(|b| convert_blueprint_to_file_tree_node_recursive(b, scope_id))
                .collect();
            update_folder_selection_states_recursive(&mut tree_nodes_3);

            // Assertions for Scenario 3
            let readme_3 = find_node_by_path(&tree_nodes_3, &PathBuf::from("README.md"))
                .expect("README.md S3 not found");
            assert_eq!(
                *readme_3.selection_state.read(),
                NodeSelectionState::Selected,
                "README.md S3 should be Selected"
            );

            let src_folder_3 = find_node_by_path(&tree_nodes_3, &PathBuf::from("src"))
                .expect("src folder S3 not found");
            assert_eq!(
                *src_folder_3.selection_state.read(),
                NodeSelectionState::Selected,
                "src folder S3 should be Selected"
            );

            let components_folder_3 =
                find_node_by_path(&tree_nodes_3, &PathBuf::from("src/components"))
                    .expect("components folder S3 not found");
            assert_eq!(
                *components_folder_3.selection_state.read(),
                NodeSelectionState::Selected,
                "components folder S3 should be Selected"
            );

            rsx! { div {} }
        }
        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }

    #[test]
    fn test_folder_expansion_signal() {
        #[allow(non_snake_case)]
        fn TestComponent() -> Element {
            let scope_id = current_scope_id().expect("TestComponent must run in a Dioxus scope");

            let folder = create_test_folder_node(
                scope_id,
                100,
                "test_folder",
                "test_folder_path",
                vec![],
                false, // Initially not expanded
                NodeSelectionState::NotSelected,
                0,
            );

            assert_eq!(
                *folder.is_expanded.read(),
                false,
                "Folder should initially be collapsed"
            );

            // Simulate toggling the signal correctly
            let mut signal_mut = folder.is_expanded;
            let current_val_1 = *signal_mut.read();
            signal_mut.set(!current_val_1);
            assert_eq!(
                *folder.is_expanded.read(),
                true,
                "Folder should now be expanded"
            );

            let current_val_2 = *signal_mut.read();
            signal_mut.set(!current_val_2);
            assert_eq!(
                *folder.is_expanded.read(),
                false,
                "Folder should be collapsed again"
            );

            rsx! { div {} }
        }
        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }
}

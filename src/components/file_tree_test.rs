#![cfg(test)]
use crate::components::file_tree::{
    build_tree_from_file_info, convert_blueprint_to_file_tree_node_recursive, FileTreeNode,
    FileTreeNodeBlueprint, NodeSelectionState, TreeNodeType,
};
use crate::fs_utils::FileInfo;
use dioxus::prelude::*;
use futures_util::FutureExt;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

fn create_file_info(path_str: &str, workspace_root_for_test: &Path) -> FileInfo {
    let relative_path = PathBuf::from(path_str);
    let absolute_path = workspace_root_for_test.join(relative_path);
    FileInfo {
        name: absolute_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        path: absolute_path,
        size: 0,        // Not relevant for tree structure
        token_count: 0, // Not relevant for tree structure
    }
}

#[test]
fn test_build_tree_empty() {
    let files = Vec::new();
    let selected_paths = HashSet::new();
    let workspace_root = Path::new("/test_ws");
    let tree = build_tree_from_file_info(&files, &selected_paths, workspace_root);
    assert!(tree.is_empty(), "Tree should be empty for no input files");
}

#[test]
fn test_build_tree_flat_list() {
    let workspace_root = Path::new("/test_ws");
    let files = vec![
        create_file_info("file1.txt", workspace_root),
        create_file_info("file2.rs", workspace_root),
        create_file_info("file3.md", workspace_root),
    ];
    let mut selected_paths = HashSet::new();
    selected_paths.insert(workspace_root.join("file2.rs"));

    let tree = build_tree_from_file_info(&files, &selected_paths, workspace_root);

    assert_eq!(tree.len(), 3, "Should have 3 root nodes");

    // file1.txt
    let node1 = tree
        .iter()
        .find(|n| n.name == "file1.txt")
        .expect("file1.txt not found");
    assert_eq!(node1.path, workspace_root.join("file1.txt"));
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
    assert_eq!(node2.path, workspace_root.join("file2.rs"));
    assert_eq!(node2.node_type, TreeNodeType::File);
    assert_eq!(node2.depth, 0);
    assert_eq!(node2.selection_state, NodeSelectionState::Selected);
    assert!(node2.children.is_empty());

    // file3.md
    let node3 = tree
        .iter()
        .find(|n| n.name == "file3.md")
        .expect("file3.md not found");
    assert_eq!(node3.path, workspace_root.join("file3.md"));
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
    let workspace_root = Path::new("/test_ws");
    let files = vec![
        create_file_info("src/main.rs", workspace_root),
        create_file_info("src/components/button.rs", workspace_root),
        create_file_info("README.md", workspace_root),
        create_file_info("src/components/mod.rs", workspace_root),
    ];
    let mut selected_paths = HashSet::new();
    selected_paths.insert(workspace_root.join("src/main.rs"));
    selected_paths.insert(workspace_root.join("src/components/button.rs"));

    // Expected structure:
    // - README.md (depth 0)
    // - src (depth 0)
    //   - main.rs (depth 1, selected)
    //   - components (depth 1)
    //     - button.rs (depth 2, selected)
    //     - mod.rs (depth 2)

    let tree = build_tree_from_file_info(&files, &selected_paths, workspace_root);

    assert_eq!(tree.len(), 2, "Should have 2 root nodes (README.md, src)");

    // README.md
    let readme_node = tree
        .iter()
        .find(|n| n.name == "README.md")
        .expect("README.md not found");
    assert_eq!(readme_node.node_type, TreeNodeType::File);
    assert_eq!(readme_node.path, workspace_root.join("README.md"));
    assert_eq!(readme_node.depth, 0);
    assert!(!readme_node.is_expanded);
    assert_eq!(readme_node.selection_state, NodeSelectionState::NotSelected);
    assert!(readme_node.children.is_empty());

    // src folder
    let src_node = tree
        .iter()
        .find(|n| n.name == "src" && n.node_type == TreeNodeType::Folder)
        .expect("src folder not found");
    assert_eq!(src_node.path, workspace_root.join("src"));
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
    assert_eq!(main_rs_node.path, workspace_root.join("src/main.rs"));
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
    assert_eq!(components_node.path, workspace_root.join("src/components"));
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
        workspace_root.join("src/components/button.rs")
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
    assert_eq!(
        mod_rs_node.path,
        workspace_root.join("src/components/mod.rs")
    );
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
    let workspace_root = Path::new("/test_ws");
    let files = vec![
        create_file_info("project_a/src/main.rs", workspace_root),
        create_file_info("project_b/src/main.rs", workspace_root),
    ];
    let selected_paths = HashSet::new();
    let tree = build_tree_from_file_info(&files, &selected_paths, workspace_root);

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
    assert_eq!(project_a_node.path, workspace_root.join("project_a"));

    let project_a_src_node = project_a_node
        .children
        .iter()
        .find(|n| n.name == "src")
        .expect("project_a/src not found");
    assert_eq!(project_a_src_node.depth, 1);
    assert_eq!(project_a_src_node.node_type, TreeNodeType::Folder);
    assert!(!project_a_src_node.is_expanded); // Not a root folder in its own right
    assert_eq!(project_a_src_node.children.len(), 1); // main.rs
    assert_eq!(
        project_a_src_node.path,
        workspace_root.join("project_a/src")
    );

    let project_a_main_rs_node = project_a_src_node
        .children
        .iter()
        .find(|n| n.name == "main.rs")
        .expect("project_a/src/main.rs not found");
    assert_eq!(project_a_main_rs_node.depth, 2);
    assert_eq!(project_a_main_rs_node.node_type, TreeNodeType::File);
    assert_eq!(
        project_a_main_rs_node.path,
        workspace_root.join("project_a/src/main.rs")
    );

    let project_b_node = tree
        .iter()
        .find(|n| n.name == "project_b")
        .expect("project_b not found");
    assert_eq!(project_b_node.depth, 0);
    assert_eq!(project_b_node.node_type, TreeNodeType::Folder);
    assert!(project_b_node.is_expanded);
    assert_eq!(project_b_node.children.len(), 1); // src
    assert_eq!(project_b_node.path, workspace_root.join("project_b"));

    let project_b_src_node = project_b_node
        .children
        .iter()
        .find(|n| n.name == "src")
        .expect("project_b/src not found");
    assert_eq!(project_b_src_node.depth, 1);
    assert_eq!(project_b_src_node.node_type, TreeNodeType::Folder);
    assert!(!project_b_src_node.is_expanded);
    assert_eq!(project_b_src_node.children.len(), 1); // main.rs
    assert_eq!(
        project_b_src_node.path,
        workspace_root.join("project_b/src")
    );

    let project_b_main_rs_node = project_b_src_node
        .children
        .iter()
        .find(|n| n.name == "main.rs")
        .expect("project_b/src/main.rs not found");
    assert_eq!(project_b_main_rs_node.depth, 2);
    assert_eq!(project_b_main_rs_node.node_type, TreeNodeType::File);
    assert_eq!(
        project_b_main_rs_node.path,
        workspace_root.join("project_b/src/main.rs")
    );
}

#[test]
fn test_build_tree_file_and_folder_same_name_at_root() {
    let workspace_root = Path::new("/test_ws");
    let files = vec![
        create_file_info("foo", workspace_root),         // File "foo"
        create_file_info("foo/bar.txt", workspace_root), // File "foo/bar.txt", implies folder "foo"
    ];
    let selected_paths = HashSet::new();
    let tree = build_tree_from_file_info(&files, &selected_paths, workspace_root);

    // Expected:
    // - foo (file, depth 0)
    // - foo (folder, depth 0)
    //   - bar.txt (file, depth 1)
    assert_eq!(
        tree.len(),
        2,
        "Should have two root items: file 'foo' and folder 'foo'"
    );

    let file_foo = tree
        .iter()
        .find(|n| n.name == "foo" && n.node_type == TreeNodeType::File)
        .expect("File 'foo' not found at root");
    assert_eq!(file_foo.depth, 0);
    assert_eq!(file_foo.path, workspace_root.join("foo"));
    assert!(file_foo.children.is_empty());

    let folder_foo = tree
        .iter()
        .find(|n| n.name == "foo" && n.node_type == TreeNodeType::Folder)
        .expect("Folder 'foo' not found at root");
    assert_eq!(folder_foo.depth, 0);
    assert_eq!(folder_foo.path, workspace_root.join("foo"));
    assert!(folder_foo.is_expanded); // Root folders are expanded
    assert_eq!(
        folder_foo.children.len(),
        1,
        "Folder 'foo' should contain 'bar.txt'"
    );

    let bar_txt_node = folder_foo
        .children
        .iter()
        .find(|n| n.name == "bar.txt")
        .expect("foo/bar.txt not found");
    assert_eq!(bar_txt_node.node_type, TreeNodeType::File);
    assert_eq!(bar_txt_node.depth, 1);
    assert_eq!(bar_txt_node.path, workspace_root.join("foo/bar.txt"));
}

#[test]
fn test_build_tree_id_uniqueness_and_order() {
    let workspace_root = Path::new("/test_ws");
    let files = vec![
        create_file_info("a.txt", workspace_root),
        create_file_info("b/c.txt", workspace_root), // folder b, file c.txt
        create_file_info("d.txt", workspace_root),
    ];
    let selected_paths = HashSet::new();

    // Expected structure for clarity (IDs are assigned during build):
    // - a.txt (file, id=0)
    // - b (folder, id=1)
    //   - c.txt (file, id=2)
    // - d.txt (file, id=3)
    // Total 4 nodes if folders are counted.
    // The test currently checks for 3 IDs: [0, 1, 2]. This implies it expects 3 nodes.
    // Let's adjust the expectation if the logic correctly creates folder nodes.
    // If `b/c.txt` creates a folder `b` and a file `c.txt` within it, plus `a.txt` and `d.txt`,
    // that's 4 nodes: `a.txt`, `b` (folder), `c.txt` (inside `b`), `d.txt`.
    // So IDs should be 0, 1, 2, 3.

    let tree = build_tree_from_file_info(&files, &selected_paths, workspace_root);

    // Verify structure first to understand what IDs to expect
    // tree should contain: a.txt, b (folder), d.txt at root
    assert_eq!(tree.len(), 3, "Root should contain a.txt, folder b, d.txt");

    let mut actual_names_at_root = tree.iter().map(|n| n.name.clone()).collect::<Vec<_>>();
    actual_names_at_root.sort();
    assert_eq!(actual_names_at_root, vec!["a.txt", "b", "d.txt"]);

    let b_folder = tree
        .iter()
        .find(|n| n.name == "b" && n.node_type == TreeNodeType::Folder)
        .expect("Folder 'b' not found");
    assert_eq!(
        b_folder.children.len(),
        1,
        "Folder 'b' should have one child"
    );
    assert_eq!(
        b_folder.children[0].name, "c.txt",
        "Child of 'b' should be 'c.txt'"
    );
    assert_eq!(b_folder.children[0].node_type, TreeNodeType::File);

    let mut ids = Vec::new();
    collect_ids(&tree, &mut ids);
    ids.sort();

    // There should be 4 nodes in total:
    // 1. a.txt (file)
    // 2. b (folder)
    // 3. c.txt (file, child of b)
    // 4. d.txt (file)
    // So we expect IDs [0, 1, 2, 3]
    assert_eq!(
        ids,
        vec![0, 1, 2, 3],
        "IDs should be 0, 1, 2, 3 after sorting for all created nodes"
    );
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
    use super::create_file_info;
    use dioxus::prelude::*;
    use dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
    use futures_util::FutureExt;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    use crate::components::file_tree::{
        build_tree_from_file_info, convert_blueprint_to_file_tree_node_recursive, FileTreeNode,
        NodeSelectionState, TreeNodeType,
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
        fn app_collect_paths() -> Element {
            rsx! { div {} }
        }
        let mut vdom = VirtualDom::new(app_collect_paths);
        vdom.rebuild_in_place();

        vdom.in_runtime(|| {
            let root_scope_id = ScopeId::ROOT; // Dioxus 0.6: Get the root scope ID from the VDOM
                                               // Or more robustly: let root_scope_id = vdom.base_scope().id();
                                               // However, for simple tests, ScopeId::ROOT inside in_runtime often works.

            let workspace_root = Path::new("/test_ws_recursive");

            let file1 = create_test_file_node(
                root_scope_id,
                0,
                "file1.txt",
                "/test_ws_recursive/file1.txt",
                NodeSelectionState::NotSelected,
                0,
            );
            let file2 = create_test_file_node(
                root_scope_id,
                1,
                "file2.txt",
                "/test_ws_recursive/sub/file2.txt",
                NodeSelectionState::NotSelected,
                1,
            );
            let file3 = create_test_file_node(
                root_scope_id,
                2,
                "file3.txt",
                "/test_ws_recursive/sub/another/file3.txt",
                NodeSelectionState::NotSelected,
                2,
            );
            let file4 = create_test_file_node(
                root_scope_id,
                3,
                "file4.txt",
                "/test_ws_recursive/sub2/file4.txt",
                NodeSelectionState::NotSelected,
                1,
            );
            let file_only_node = create_test_file_node(
                root_scope_id,
                7,
                "singlefile.txt",
                "/test_ws_recursive/singlefile.txt",
                NodeSelectionState::NotSelected,
                0,
            );

            let another_folder = create_test_folder_node(
                root_scope_id,
                4,
                "another",
                "/test_ws_recursive/sub/another",
                vec![file3.clone()],
                false,
                NodeSelectionState::NotSelected,
                1,
            );
            let sub_folder = create_test_folder_node(
                root_scope_id,
                5,
                "sub",
                "/test_ws_recursive/sub",
                vec![file2.clone(), another_folder.clone()],
                true,
                NodeSelectionState::NotSelected,
                0,
            );
            let sub2_folder = create_test_folder_node(
                root_scope_id,
                6,
                "sub2",
                "/test_ws_recursive/sub2",
                vec![file4.clone()],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            let empty_folder = create_test_folder_node(
                root_scope_id,
                8,
                "empty",
                "/test_ws_recursive/empty",
                vec![],
                false,
                NodeSelectionState::NotSelected,
                0,
            );
            let root_folder_children = vec![file1.clone(), sub_folder.clone(), sub2_folder.clone()];
            let root_folder = create_test_folder_node(
                root_scope_id,
                9,
                "test_ws_recursive",
                "/test_ws_recursive",
                root_folder_children,
                true,
                NodeSelectionState::NotSelected,
                0,
            );

            // Test case 1: Collect from a folder with nested children
            let paths_from_sub = sub_folder.collect_all_file_paths_recursive();
            let mut expected_paths_from_sub = vec![
                workspace_root.join("sub/file2.txt"),
                workspace_root.join("sub/another/file3.txt"),
            ];
            expected_paths_from_sub.sort();
            let mut actual_paths_from_sub = paths_from_sub;
            actual_paths_from_sub.sort();
            assert_eq!(actual_paths_from_sub, expected_paths_from_sub);

            // Test case 2: Collect from a folder with direct children only
            let paths_from_sub2 = sub2_folder.collect_all_file_paths_recursive();
            assert_eq!(paths_from_sub2, vec![workspace_root.join("sub2/file4.txt")]);

            // Test case 3: Collect from an empty folder
            let paths_from_empty = empty_folder.collect_all_file_paths_recursive();
            assert!(paths_from_empty.is_empty());

            // Test case 4: Collect from a file node (should return its own path)
            let paths_from_file = file1.collect_all_file_paths_recursive();
            assert_eq!(paths_from_file, vec![workspace_root.join("file1.txt")]);

            // Test case 5: Collect from a file node (using single_file_node)
            let paths_from_single_file = file_only_node.collect_all_file_paths_recursive();
            assert_eq!(
                paths_from_single_file,
                vec![workspace_root.join("singlefile.txt")]
            );

            // Test for root_folder containing other folders and files
            let paths_from_root = root_folder.collect_all_file_paths_recursive();
            let mut expected_paths_from_root = vec![
                workspace_root.join("file1.txt"),
                workspace_root.join("sub/file2.txt"),
                workspace_root.join("sub/another/file3.txt"),
                workspace_root.join("sub2/file4.txt"),
            ];
            expected_paths_from_root.sort();
            let mut actual_paths_from_root = paths_from_root;
            actual_paths_from_root.sort();
            assert_eq!(actual_paths_from_root, expected_paths_from_root);
        });
    }

    // This is a copy from the outer scope, adjusted for Story 10 paths
    // If it's identical, it can be removed, but good to have a local version for story 10 if it diverges.
    // For now, let's assume it might be specific or call the outer one.
    // The original create_file_info is not in scope here directly without super::
    // fn create_file_info_for_test(path_str: &str) -> FileInfo { ... }
    // Using the new `create_file_info_for_story10` defined above.

    // Utility to find a node by its absolute path in a Vec<FileTreeNode>
    fn find_node_by_path(nodes: &[FileTreeNode], path_to_find: &Path) -> Option<FileTreeNode> {
        for node in nodes {
            if node.path == path_to_find {
                return Some(node.clone());
            }
            if node.node_type == TreeNodeType::Folder {
                if let Some(found_in_child) = find_node_by_path(&node.children, path_to_find) {
                    return Some(found_in_child);
                }
            }
        }
        None
    }

    #[test]
    fn test_selection_state_logic_full_pipeline() {
        #[component]
        fn SelectionPipelineTestComponent() -> Element {
            let current_scope_id = ScopeId::ROOT; // Still need a ScopeId for recursive conversion

            let workspace_root_path = use_hook(|| PathBuf::from("/test_ws_select"));
            let workspace_root_path_clone = workspace_root_path.clone();
            let all_files_data = use_hook(|| {
                vec![
                    create_file_info("README.md", &workspace_root_path),
                    create_file_info("src/main.rs", &workspace_root_path),
                    create_file_info("src/utils.rs", &workspace_root_path),
                    create_file_info("src/components/comp1.rs", &workspace_root_path),
                    create_file_info("src/components/comp2.rs", &workspace_root_path),
                    create_file_info("data/file1.json", &workspace_root_path),
                ]
            });

            let mut selected_paths_signal = use_signal(|| {
                let mut hs = HashSet::new();
                hs.insert(workspace_root_path.join("src/main.rs"));
                hs.insert(workspace_root_path.join("src/components/comp1.rs"));
                hs
            });

            let tree_blueprints = use_memo(move || {
                build_tree_from_file_info(
                    &all_files_data,
                    &selected_paths_signal.read(),
                    &workspace_root_path_clone,
                )
            });

            let tree_nodes = use_memo(move || {
                tree_blueprints
                    .read()
                    .clone()
                    .into_iter()
                    .map(|bp| convert_blueprint_to_file_tree_node_recursive(bp, current_scope_id))
                    .collect::<Vec<FileTreeNode>>()
            });

            // Perform assertions - these run on every render
            // Initial state check
            let nodes_read = tree_nodes.read();
            let readme_node =
                find_node_by_path(&nodes_read, &workspace_root_path.join("README.md"))
                    .expect("README.md not found");
            assert_eq!(
                *readme_node.selection_state.read(),
                NodeSelectionState::NotSelected
            );
            let main_rs_node =
                find_node_by_path(&nodes_read, &workspace_root_path.join("src/main.rs"))
                    .expect("src/main.rs not found");
            assert_eq!(
                *main_rs_node.selection_state.read(),
                NodeSelectionState::Selected
            );
            // ... (assert all other initial states)
            let utils_rs_node =
                find_node_by_path(&nodes_read, &workspace_root_path.join("src/utils.rs"))
                    .expect("src/utils.rs not found");
            assert_eq!(
                *utils_rs_node.selection_state.read(),
                NodeSelectionState::NotSelected
            );
            let comp1_rs_node = find_node_by_path(
                &nodes_read,
                &workspace_root_path.join("src/components/comp1.rs"),
            )
            .expect("src/components/comp1.rs not found");
            assert_eq!(
                *comp1_rs_node.selection_state.read(),
                NodeSelectionState::Selected
            );
            let comp2_rs_node = find_node_by_path(
                &nodes_read,
                &workspace_root_path.join("src/components/comp2.rs"),
            )
            .expect("src/components/comp2.rs not found");
            assert_eq!(
                *comp2_rs_node.selection_state.read(),
                NodeSelectionState::NotSelected
            );
            let data_file1_node =
                find_node_by_path(&nodes_read, &workspace_root_path.join("data/file1.json"))
                    .expect("data/file1.json not found");
            assert_eq!(
                *data_file1_node.selection_state.read(),
                NodeSelectionState::NotSelected
            );
            let components_folder_node =
                find_node_by_path(&nodes_read, &workspace_root_path.join("src/components"))
                    .expect("src/components folder not found");
            assert_eq!(components_folder_node.node_type, TreeNodeType::Folder);
            assert_eq!(
                *components_folder_node.selection_state.read(),
                NodeSelectionState::PartiallySelected
            );
            let src_folder_node = find_node_by_path(&nodes_read, &workspace_root_path.join("src"))
                .expect("src folder not found");
            assert_eq!(src_folder_node.node_type, TreeNodeType::Folder);
            assert_eq!(
                *src_folder_node.selection_state.read(),
                NodeSelectionState::PartiallySelected
            );
            let data_folder_node =
                find_node_by_path(&nodes_read, &workspace_root_path.join("data"))
                    .expect("data folder not found");
            assert_eq!(data_folder_node.node_type, TreeNodeType::Folder);
            assert_eq!(
                *data_folder_node.selection_state.read(),
                NodeSelectionState::NotSelected
            );

            // Use an effect to simulate state changes and re-assertions
            let mut test_phase = use_signal(|| 0); // Made mutable

            use_effect(move || {
                if *test_phase.read() == 0 {
                    // Need to clone workspace_root_path if used inside effect closures too
                    let root_clone = workspace_root_path.clone();
                    selected_paths_signal.write().clear();
                    selected_paths_signal
                        .write()
                        .insert(root_clone.join("src/components/comp1.rs"));
                    selected_paths_signal
                        .write()
                        .insert(root_clone.join("src/components/comp2.rs"));
                    test_phase.set(1);
                } else if *test_phase.read() == 1 {
                    let root_clone = workspace_root_path.clone(); // Clone again for this branch
                    let nodes_read_phase1 = tree_nodes.read();
                    let comp1_rs_node_2 = find_node_by_path(
                        &nodes_read_phase1,
                        &root_clone.join("src/components/comp1.rs"),
                    )
                    .expect("src/components/comp1.rs not found (phase 1)");
                    assert_eq!(
                        *comp1_rs_node_2.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let comp2_rs_node_2 = find_node_by_path(
                        &nodes_read_phase1,
                        &root_clone.join("src/components/comp2.rs"),
                    )
                    .expect("src/components/comp2.rs not found (phase 1)");
                    assert_eq!(
                        *comp2_rs_node_2.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let components_folder_node_2 =
                        find_node_by_path(&nodes_read_phase1, &root_clone.join("src/components"))
                            .expect("src/components folder not found (phase 1)");
                    assert_eq!(
                        *components_folder_node_2.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let src_folder_node_2 =
                        find_node_by_path(&nodes_read_phase1, &root_clone.join("src"))
                            .expect("src folder not found (phase 1)");
                    assert_eq!(
                        *src_folder_node_2.selection_state.read(),
                        NodeSelectionState::PartiallySelected
                    );

                    // Transition to phase 2: Deselect all
                    selected_paths_signal.write().clear();
                    test_phase.set(2);
                } else if *test_phase.read() == 2 {
                    let root_clone = workspace_root_path.clone(); // Clone again
                    let nodes_read_phase2 = tree_nodes.read();
                    let components_folder_node_3 =
                        find_node_by_path(&nodes_read_phase2, &root_clone.join("src/components"))
                            .expect("src/components folder not found (phase 2)");
                    assert_eq!(
                        *components_folder_node_3.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );
                    let src_folder_node_3 =
                        find_node_by_path(&nodes_read_phase2, &root_clone.join("src"))
                            .expect("src folder not found (phase 2)");
                    assert_eq!(
                        *src_folder_node_3.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );
                    let main_rs_node_3 =
                        find_node_by_path(&nodes_read_phase2, &root_clone.join("src/main.rs"))
                            .expect("src/main.rs not found (phase 2)");
                    assert_eq!(
                        *main_rs_node_3.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );

                    // Test finished
                    test_phase.set(3);
                }
            });

            rsx! { div { "Phase: {test_phase}" } }
        }

        let mut vdom = VirtualDom::new(SelectionPipelineTestComponent);
        vdom.rebuild_in_place();
        // Run the event loop until the test completes (phase becomes 3)
        // This requires a way to check the test_phase state from outside or run for a fixed number of ticks.
        // A simple approach for testing might be to run the loop a few times.
        for _ in 0..10 {
            // Arbitrary number of ticks, might need adjustment
            vdom.wait_for_work().now_or_never();
            vdom.process_events();
            vdom.render_immediate(&mut NoOpMutations);
        }
        // Final assertion can be tricky without direct access to component state after VDOM stops.
        // The assertions inside the effect are the primary validation.
    }

    #[test]
    fn test_folder_expansion_signal() {
        #[component]
        fn ExpansionTestComponent() -> Element {
            let workspace_root = use_hook(|| PathBuf::from("/test_ws_expand"));
            let selected_paths_signal = use_signal(HashSet::new);

            let blueprints = use_hook(|| {
                let file_infos = vec![
                    create_file_info("folder1/file1.txt", &workspace_root),
                    create_file_info("folder1/subfolder/file2.txt", &workspace_root),
                    create_file_info("folder2/file3.txt", &workspace_root),
                ];
                build_tree_from_file_info(
                    &file_infos,
                    &selected_paths_signal.read(),
                    &workspace_root,
                )
            });

            let tree_nodes = use_hook(|| {
                blueprints
                    .clone()
                    .into_iter()
                    .map(|bp| convert_blueprint_to_file_tree_node_recursive(bp, ScopeId::ROOT)) // Use ScopeId::ROOT if vdom manages it, or cx.scope_id() if available
                    .collect::<Vec<FileTreeNode>>()
            });

            let folder1_node = use_hook(|| {
                find_node_by_path(&tree_nodes, &workspace_root.join("folder1"))
                    .expect("folder1 not found")
            });
            assert!(folder1_node.is_expanded.read().clone());

            let subfolder_node = use_hook(|| {
                find_node_by_path(&tree_nodes, &workspace_root.join("folder1/subfolder"))
                    .expect("folder1/subfolder not found")
            });
            assert!(!subfolder_node.is_expanded.read().clone());

            use_effect(move || {
                // Clone the signal handle and make it mutable
                let mut is_expanded_signal = subfolder_node.is_expanded;
                is_expanded_signal.toggle();
            });

            assert!(
                subfolder_node.is_expanded.read().clone(),
                "Subfolder should be expanded after toggle"
            );

            let empty_selected_paths_for_file_check = use_signal(HashSet::new);
            let file1_node_bp = use_hook(|| {
                build_tree_from_file_info(
                    &[create_file_info("file.txt", &workspace_root)],
                    &empty_selected_paths_for_file_check.read(),
                    &workspace_root,
                )
            });
            assert!(!file1_node_bp[0].is_expanded);

            rsx! { div { "Test completed" } }
        }

        let mut vdom = VirtualDom::new(ExpansionTestComponent);
        vdom.rebuild_in_place();
        // We might need to run the VDOM event loop briefly to allow effects to run
        // vdom.wait_for_idle().await; // If async context is available
        // Or just run immediate work might be enough for simple cases
        vdom.process_events();
    }

    #[test]
    fn test_select_all_deselect_all_logic() {
        #[component]
        fn SelectAllTestComponent() -> Element {
            let current_scope_id = ScopeId::ROOT;
            let workspace_root = use_hook(|| PathBuf::from("/test_ws_select_all"));

            let all_files_prop = use_signal(|| {
                vec![
                    create_file_info("file1.txt", &workspace_root),
                    create_file_info("src/main.rs", &workspace_root),
                    create_file_info("src/components/button.rs", &workspace_root),
                ]
            });
            let mut selected_paths_prop = use_signal(HashSet::new);
            let mut test_phase = use_signal(|| 0); // 0: initial, 1: select all done, 2: deselect all done

            use_effect(move || {
                if *test_phase.read() == 0 {
                    // Simulate "Select All"
                    let mut all_file_paths_for_selection = HashSet::new();
                    for file_info in all_files_prop.read().iter() {
                        all_file_paths_for_selection.insert(file_info.path.clone());
                    }
                    selected_paths_prop.set(all_file_paths_for_selection);
                    test_phase.set(1);
                } else if *test_phase.read() == 1 {
                    // Assertions after Select All
                    let tree_blueprints_selected_all = build_tree_from_file_info(
                        &all_files_prop.read(),
                        &selected_paths_prop.read(),
                        &workspace_root,
                    );
                    let tree_nodes_selected_all: Vec<FileTreeNode> = tree_blueprints_selected_all
                        .into_iter()
                        .map(|bp| {
                            convert_blueprint_to_file_tree_node_recursive(bp, current_scope_id)
                        })
                        .collect();

                    let file1_node = find_node_by_path(
                        &tree_nodes_selected_all,
                        &workspace_root.join("file1.txt"),
                    )
                    .unwrap();
                    assert_eq!(
                        *file1_node.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let main_rs_node = find_node_by_path(
                        &tree_nodes_selected_all,
                        &workspace_root.join("src/main.rs"),
                    )
                    .unwrap();
                    assert_eq!(
                        *main_rs_node.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let button_rs_node = find_node_by_path(
                        &tree_nodes_selected_all,
                        &workspace_root.join("src/components/button.rs"),
                    )
                    .unwrap();
                    assert_eq!(
                        *button_rs_node.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let components_folder = find_node_by_path(
                        &tree_nodes_selected_all,
                        &workspace_root.join("src/components"),
                    )
                    .unwrap();
                    assert_eq!(
                        *components_folder.selection_state.read(),
                        NodeSelectionState::Selected
                    );
                    let src_folder =
                        find_node_by_path(&tree_nodes_selected_all, &workspace_root.join("src"))
                            .unwrap();
                    assert_eq!(
                        *src_folder.selection_state.read(),
                        NodeSelectionState::Selected
                    );

                    // Simulate "Deselect All"
                    selected_paths_prop.set(HashSet::new());
                    test_phase.set(2);
                } else if *test_phase.read() == 2 {
                    // Assertions after Deselect All
                    let tree_blueprints_deselected_all = build_tree_from_file_info(
                        &all_files_prop.read(),
                        &selected_paths_prop.read(),
                        &workspace_root,
                    );
                    let tree_nodes_deselected_all: Vec<FileTreeNode> =
                        tree_blueprints_deselected_all
                            .into_iter()
                            .map(|bp| {
                                convert_blueprint_to_file_tree_node_recursive(bp, current_scope_id)
                            })
                            .collect();

                    let file1_node_deselected = find_node_by_path(
                        &tree_nodes_deselected_all,
                        &workspace_root.join("file1.txt"),
                    )
                    .unwrap();
                    assert_eq!(
                        *file1_node_deselected.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );
                    let main_rs_node_deselected = find_node_by_path(
                        &tree_nodes_deselected_all,
                        &workspace_root.join("src/main.rs"),
                    )
                    .unwrap();
                    assert_eq!(
                        *main_rs_node_deselected.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );
                    let button_rs_node_deselected = find_node_by_path(
                        &tree_nodes_deselected_all,
                        &workspace_root.join("src/components/button.rs"),
                    )
                    .unwrap();
                    assert_eq!(
                        *button_rs_node_deselected.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );
                    let components_folder_deselected = find_node_by_path(
                        &tree_nodes_deselected_all,
                        &workspace_root.join("src/components"),
                    )
                    .unwrap();
                    assert_eq!(
                        *components_folder_deselected.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );
                    let src_folder_deselected =
                        find_node_by_path(&tree_nodes_deselected_all, &workspace_root.join("src"))
                            .unwrap();
                    assert_eq!(
                        *src_folder_deselected.selection_state.read(),
                        NodeSelectionState::NotSelected
                    );

                    test_phase.set(3); // Mark test as finished
                }
            });

            rsx! { div { "Phase: {test_phase}" } }
        }

        let mut vdom = VirtualDom::new(SelectAllTestComponent);
        vdom.rebuild_in_place();
        // Run VDOM until test completes
        for _ in 0..10 {
            // Arbitrary ticks, might need adjustment
            vdom.wait_for_work().now_or_never();
            vdom.process_events();
            vdom.render_immediate(&mut NoOpMutations);
        }
    }
}

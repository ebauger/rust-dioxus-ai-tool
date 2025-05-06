#![cfg(test)]
use crate::components::file_tree::{
    build_tree_from_file_info, FileTreeNodeBlueprint, NodeSelectionState, TreeNodeType,
};
use crate::fs_utils::FileInfo;
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
        collect_names_and_depths(&node.children, names_depths);
    }
}

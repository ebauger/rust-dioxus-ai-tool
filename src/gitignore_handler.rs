// src/gitignore_handler.rs
use std::path::{Path, PathBuf};

/// Checks for a .gitignore file in the given workspace root path.
///
/// # Arguments
/// * `workspace_root_path`: The absolute path to the root of the workspace.
///
/// # Returns
/// * `Some(PathBuf)` containing the absolute path to `.gitignore` if it exists.
/// * `None` if `.gitignore` does not exist at the root of the workspace.
pub fn check_for_gitignore(workspace_root_path: &Path) -> Option<PathBuf> {
    let gitignore_path = workspace_root_path.join(".gitignore");
    if gitignore_path.is_file() {
        Some(gitignore_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_gitignore_exists() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path();
        File::create(workspace_path.join(".gitignore")).unwrap();

        let result = check_for_gitignore(workspace_path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), workspace_path.join(".gitignore"));
    }

    #[test]
    fn test_gitignore_does_not_exist() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path();

        let result = check_for_gitignore(workspace_path);
        assert!(result.is_none());
    }

    #[test]
    fn test_gitignore_is_a_directory() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path();
        fs::create_dir(workspace_path.join(".gitignore")).unwrap();

        let result = check_for_gitignore(workspace_path);
        assert!(result.is_none()); // .is_file() should be false for a directory
    }
}

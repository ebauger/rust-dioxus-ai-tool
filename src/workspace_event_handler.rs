use std::collections::HashSet;
use std::error::Error;
use std::path::Path; // For Box<dyn Error>

// Import necessary functions
use crate::fs_utils::get_all_workspace_files;
use crate::gitignore_handler::{
    check_for_gitignore, is_file_ignored, preprocess_gitignore_lines, read_gitignore_patterns,
};

pub fn handle_workspace_opened(
    workspace_path_str: String,
) -> Result<HashSet<String>, Box<dyn Error>> {
    println!(
        "[INFO] Workspace opened event triggered for path: {}",
        workspace_path_str
    );

    let workspace_root = Path::new(&workspace_path_str);
    let mut final_selected_files = HashSet::new();

    // Get all files first (excluding .git)
    let all_files = get_all_workspace_files(workspace_root)?; // Propagate IO errors
    println!("[INFO] Found {} files initially.", all_files.len());

    // Check for .gitignore
    if let Some(gitignore_path) = check_for_gitignore(workspace_root) {
        println!("[INFO] Found .gitignore at: {}", gitignore_path.display());

        // Try reading and processing .gitignore
        match read_gitignore_patterns(&gitignore_path) {
            Ok(raw_patterns) => {
                let processed_patterns = preprocess_gitignore_lines(raw_patterns);
                println!(
                    "[INFO] Loaded {} effective patterns from .gitignore.",
                    processed_patterns.len()
                );

                // Filter files based on patterns
                for file_path in all_files {
                    if !is_file_ignored(&file_path, &processed_patterns, workspace_root) {
                        final_selected_files.insert(file_path);
                    }
                }
                println!(
                    "[INFO] Selected {} files after applying .gitignore rules.",
                    final_selected_files.len()
                );
            }
            Err(e) => {
                eprintln!(
                    "[ERROR] Failed to read .gitignore file at {}: {}. Returning error.",
                    gitignore_path.display(),
                    e
                );
                // Return the error if .gitignore exists but is unreadable
                return Err(Box::new(e));
            }
        }
    } else {
        // No .gitignore found. Spec says deselect all files.
        println!("[INFO] No .gitignore found. Deselecting all files.");
        // final_selected_files remains empty, which is correct.
    }

    Ok(final_selected_files)
}

// Placeholder for where this function might be called from, // Modified placeholder
// e.g., your main application loop or an event subscription mechanism.
#[cfg(test)] // Added cfg(test) for example usage as a test
mod tests {
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_handle_workspace_opened_scenario() -> Result<(), Box<dyn Error>> {
        // Setup a temporary workspace
        let dir = tempdir()?;
        let root = dir.path();

        // Create some files
        create_dir_all(root.join("src"))?;
        File::create(root.join("src/main.rs"))?.write_all(b"fn main() {}")?;
        File::create(root.join("README.md"))?.write_all(b"# Test Project")?;
        create_dir_all(root.join("target"))?;
        File::create(root.join("target/debug.log"))?.write_all(b"debug info")?;
        create_dir_all(root.join(".git"))?;
        File::create(root.join(".git/config"))?.write_all(b"[core]")?;

        // Case 1: No .gitignore
        println!("\n--- Testing without .gitignore ---");
        let selected_none = handle_workspace_opened(root.to_str().unwrap().to_string())?;
        assert!(
            selected_none.is_empty(),
            "Expected empty set without .gitignore"
        );

        // Case 2: With .gitignore
        println!("\n--- Testing with .gitignore ---");
        let gitignore_path = root.join(".gitignore");
        let mut gitignore_file = File::create(&gitignore_path)?;
        writeln!(gitignore_file, "target/")?;
        writeln!(gitignore_file, "*.log")?;
        drop(gitignore_file);

        let selected_with = handle_workspace_opened(root.to_str().unwrap().to_string())?;

        let expected_files: HashSet<String> = [
            "src/main.rs".to_string(),
            "README.md".to_string(),
            // ".gitignore" should not be listed by get_all_workspace_files if it includes itself
            // get_all_workspace_files excludes .git/* but not necessarily .gitignore
            // Let's assume get_all_workspace_files doesn't list .gitignore itself (or handle it)
        ]
        .iter()
        .cloned()
        .collect();

        // Check if .gitignore is listed by get_all_workspace_files
        // If it is, it should be selected unless explicitly ignored
        let all_files_for_check = get_all_workspace_files(root)?;
        if all_files_for_check.contains(&".gitignore".to_string()) {
            println!("WARN: get_all_workspace_files includes .gitignore");
            // If .gitignore is listed, it should NOT be ignored by default patterns
            // unless the .gitignore itself contains a pattern like ".gitignore"
        }

        assert_eq!(selected_with.len(), 2, "Expected 2 files selected");
        assert!(selected_with.contains("src/main.rs"));
        assert!(selected_with.contains("README.md"));
        assert!(!selected_with.contains("target/debug.log")); // Ignored by target/
        assert!(!selected_with.contains(".git/config")); // Excluded by get_all_workspace_files

        Ok(())
    }
}

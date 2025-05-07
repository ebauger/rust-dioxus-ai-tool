// src/gitignore_handler.rs
use ignore::gitignore::GitignoreBuilder;
use std::io;
use std::path::{Path, PathBuf}; // Added for io::Result // Added for GitignoreBuilder

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

/// Reads a .gitignore file and returns its content as a list of raw lines.
///
/// # Arguments
/// * `gitignore_file_path`: The absolute path to the .gitignore file.
///
/// # Returns
/// * `Ok(Vec<String>)` containing each line from the .gitignore file.
/// * `Err(io::Error)` if the file cannot be read.
pub fn read_gitignore_patterns(gitignore_file_path: &Path) -> io::Result<Vec<String>> {
    let content = std::fs::read_to_string(gitignore_file_path)?;
    let lines = content.lines().map(String::from).collect();
    Ok(lines)
}

/// Pre-processes raw lines from a .gitignore file.
/// - Trims whitespace from each line.
/// - Removes empty lines.
/// - Removes comment lines (starting with '#').
///
/// # Arguments
/// * `raw_lines`: A vector of strings, where each string is a raw line from .gitignore.
///
/// # Returns
/// * A new vector of strings containing only the effective pattern strings.
pub fn preprocess_gitignore_lines(raw_lines: Vec<String>) -> Vec<String> {
    raw_lines
        .into_iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Checks if a relative file path is ignored based on processed .gitignore patterns.
///
/// # Arguments
/// * `relative_file_path`: The path of the file, relative to the workspace root.
/// * `processed_patterns`: A slice of effective pattern strings from .gitignore.
/// * `workspace_root`: The absolute path to the root of the workspace.
///
/// # Returns
/// * `true` if the file should be excluded (ignored), `false` otherwise.
///   If building the ignore rules fails, it logs an error and defaults to `false`.
pub fn is_file_ignored(
    relative_file_path: &str,
    processed_patterns: &[String],
    workspace_root: &Path,
) -> bool {
    let mut builder = GitignoreBuilder::new(workspace_root);
    for pattern_str in processed_patterns {
        // Using add_line(None, ...) treats patterns as if they are from a .gitignore
        // file at the workspace_root.
        if let Err(e) = builder.add_line(None, pattern_str) {
            // This error path for add_line is less common with `None` base,
            // but good to acknowledge. `build()` is more likely to error on bad globs.
            eprintln!(
                "Error adding gitignore pattern '{}': {}. File will not be ignored by this pattern.",
                pattern_str,
                e
            );
            // Continue adding other patterns
        }
    }

    match builder.build() {
        Ok(gitignore) => {
            let path_to_check = workspace_root.join(relative_file_path);

            // Assuming relative_file_path always refers to a file, so is_dir = false.
            // Use matched_path_or_any_parents to check the file and its ancestors.
            let match_result = gitignore.matched_path_or_any_parents(&path_to_check, false);
            match_result.is_ignore()
        }
        Err(e) => {
            eprintln!(
                "Error building gitignore rules: {}. Assuming file is not ignored.",
                e
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir; // Added for new tests

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

    // New tests for read_gitignore_patterns
    #[test]
    fn test_read_gitignore_patterns_valid_file() -> io::Result<()> {
        let dir = tempdir()?;
        let gitignore_path = dir.path().join(".gitignore_test_read");
        let mut file = File::create(&gitignore_path)?;
        writeln!(file, "node_modules/")?;
        writeln!(file, "*.log")?;
        writeln!(file, "")?;
        writeln!(file, " # A comment")?;
        writeln!(file, "build/")?;
        drop(file);

        let patterns = read_gitignore_patterns(&gitignore_path)?;
        assert_eq!(patterns.len(), 5);
        assert_eq!(patterns[0], "node_modules/");
        assert_eq!(patterns[1], "*.log");
        assert_eq!(patterns[2], "");
        assert_eq!(patterns[3], " # A comment");
        assert_eq!(patterns[4], "build/");
        Ok(())
    }

    #[test]
    fn test_read_gitignore_patterns_empty_file() -> io::Result<()> {
        let dir = tempdir()?;
        let gitignore_path = dir.path().join(".gitignore_test_empty");
        // Create a truly empty 0-byte file
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&gitignore_path)?;

        let patterns = read_gitignore_patterns(&gitignore_path)?;
        assert!(patterns.is_empty());
        Ok(())
    }

    #[test]
    fn test_read_gitignore_patterns_file_with_final_newline() -> io::Result<()> {
        let dir = tempdir()?;
        let gitignore_path = dir.path().join(".gitignore_test_final_nl");
        let mut file = File::create(&gitignore_path)?;
        writeln!(file, "line1")?;
        write!(file, "line2\n")?;
        drop(file);

        let patterns = read_gitignore_patterns(&gitignore_path)?;
        assert_eq!(patterns, vec!["line1".to_string(), "line2".to_string()]);
        Ok(())
    }

    #[test]
    fn test_read_gitignore_patterns_file_with_only_newline() -> io::Result<()> {
        let dir = tempdir()?;
        let gitignore_path = dir.path().join(".gitignore_test_only_nl");
        let mut file = File::create(&gitignore_path)?;
        writeln!(file)?;
        drop(file);

        let patterns = read_gitignore_patterns(&gitignore_path)?;
        assert_eq!(patterns, vec!["".to_string()]);
        Ok(())
    }

    #[test]
    fn test_read_gitignore_patterns_non_existent_file() -> io::Result<()> {
        let dir = tempdir()?;
        let gitignore_path = dir.path().join(".gitignore_does_not_exist");

        let result = read_gitignore_patterns(&gitignore_path);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        } else {
            panic!("Expected an error for non-existent file");
        }
        Ok(())
    }

    // New tests for preprocess_gitignore_lines
    #[test]
    fn test_preprocess_gitignore_lines_basic() {
        let raw_lines = vec![
            "  node_modules/  ".to_string(),
            "*.log".to_string(),
            " ".to_string(), // Whitespace only line
            "# This is a comment".to_string(),
            "build/".to_string(),
            "   # Another comment with leading spaces".to_string(),
            "another/pattern # with trailing comment".to_string(),
            "!important_file.txt".to_string(),
            "trailing_space   ".to_string(),
        ];
        let processed = preprocess_gitignore_lines(raw_lines);
        assert_eq!(
            processed,
            vec![
                "node_modules/".to_string(),
                "*.log".to_string(),
                "build/".to_string(),
                "another/pattern # with trailing comment".to_string(),
                "!important_file.txt".to_string(),
                "trailing_space".to_string(),
            ]
        );
    }

    #[test]
    fn test_preprocess_gitignore_lines_empty_input() {
        let raw_lines: Vec<String> = vec![];
        let processed = preprocess_gitignore_lines(raw_lines);
        assert!(processed.is_empty());
    }

    #[test]
    fn test_preprocess_gitignore_lines_all_comments_or_empty() {
        let raw_lines = vec![
            "# comment 1".to_string(),
            "   ".to_string(),
            " # comment 2".to_string(),
            "\t# comment 3 with tab".to_string(),
        ];
        let processed = preprocess_gitignore_lines(raw_lines);
        assert!(processed.is_empty());
    }

    #[test]
    fn test_preprocess_gitignore_lines_no_changes_needed() {
        let raw_lines = vec!["src/".to_string(), "*.o".to_string(), "a/b/c".to_string()];
        let processed = preprocess_gitignore_lines(raw_lines.clone());
        assert_eq!(processed, raw_lines);
    }

    #[test]
    fn test_preprocess_gitignore_line_with_hash_not_at_start() {
        let raw_lines = vec!["file#withhash.txt".to_string()];
        let processed = preprocess_gitignore_lines(raw_lines);
        assert_eq!(processed, vec!["file#withhash.txt".to_string()]);
    }

    // Helper for is_file_ignored tests
    fn test_is_ignored_case(
        path_str: &str,
        patterns: &[&str],
        expected: bool,
        case_name: &str, // Changed parameter name for clarity
    ) {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();
        // It's good practice to create the file for is_dir to be accurate if the library relied on it,
        // but the `ignore` crate primarily uses the boolean flag and path string.
        // For these tests, `is_dir` is hardcoded to false in `is_file_ignored` call to `matched`.
        // Ensure the workspace_root exists as a directory.
        std::fs::create_dir_all(
            workspace_root.join(Path::new(path_str).parent().unwrap_or(Path::new(""))),
        )
        .unwrap();
        if !path_str.ends_with('/') {
            // Don't try to create a file if path_str is meant to be a dir pattern test target
            File::create(workspace_root.join(path_str)).unwrap();
        }

        let processed_patterns_vec: Vec<String> = patterns.iter().map(|s| s.to_string()).collect();

        let actual = is_file_ignored(path_str, &processed_patterns_vec, workspace_root);
        assert_eq!(
            actual, expected,
            "Test failed for [{}]: path '{}' with patterns {:?}. Expected {}, got {}",
            case_name, path_str, patterns, expected, actual
        );
    }

    #[test]
    fn test_is_file_ignored_simple_file_match() {
        test_is_ignored_case("file.log", &["*.log"], true, "simple_log");
        test_is_ignored_case("file.txt", &["*.log"], false, "simple_txt_no_match");
        test_is_ignored_case("file.log", &["file.log"], true, "exact_file_log");
        test_is_ignored_case(
            "sub/file.log",
            &["file.log"],
            true,
            "exact_file_log_in_subdir",
        );
        test_is_ignored_case(
            "sub/file.log",
            &["sub/file.log"],
            true,
            "exact_path_in_subdir",
        );
    }

    #[test]
    fn test_is_file_ignored_directory_match() {
        test_is_ignored_case(
            "build/output.txt",
            &["build/"],
            true,
            "dir_match_file_inside",
        );
        test_is_ignored_case("logs/errors.txt", &["logs/"], true, "logs_dir_file_inside");
        test_is_ignored_case("src/main.rs", &["build/"], false, "dir_no_match");
        test_is_ignored_case(
            "output/file.txt",
            &["output"],
            true,
            "implicit_dir_match_output",
        );
        test_is_ignored_case(
            "other_output/file.txt",
            &["output"],
            false,
            "implicit_dir_no_match_other",
        );
    }

    #[test]
    fn test_is_file_ignored_wildcard() {
        test_is_ignored_case("temp.tmp", &["*.tmp"], true, "wildcard_tmp");
        test_is_ignored_case("src/temp.tmp", &["*.tmp"], true, "wildcard_in_subdir_tmp");
        test_is_ignored_case("data.txt", &["d*a.txt"], true, "wildcard_middle");
        test_is_ignored_case("src/data.txt", &["d*a.txt"], true, "wildcard_middle_subdir");
    }

    #[test]
    fn test_is_file_ignored_anchored() {
        test_is_ignored_case("root.file", &["/root.file"], true, "anchored_match");
        test_is_ignored_case(
            "src/root.file",
            &["/root.file"],
            false,
            "anchored_no_match_subdir",
        );
        test_is_ignored_case(
            "src/another.file",
            &["another.file"],
            true,
            "non_anchored_match_subdir",
        );
        test_is_ignored_case(
            "another.file",
            &["another.file"],
            true,
            "non_anchored_match_root",
        );
    }

    #[test]
    fn test_is_file_ignored_path_specific() {
        test_is_ignored_case(
            "docs/README.md",
            &["docs/README.md"],
            true,
            "path_specific_match",
        );
        test_is_ignored_case(
            "README.md",
            &["docs/README.md"],
            false,
            "path_specific_no_match_root",
        );
        test_is_ignored_case(
            "other/docs/README.md",
            &["docs/README.md"],
            false,
            "path_specific_no_match_elsewhere",
        );
    }

    #[test]
    fn test_is_file_ignored_negation() {
        test_is_ignored_case(
            "important.md",
            &["*.md", "!important.md"],
            false,
            "negation_target",
        );
        test_is_ignored_case(
            "other.md",
            &["*.md", "!important.md"],
            true,
            "negation_other_md",
        );
        test_is_ignored_case(
            "data.txt",
            &["*.md", "!important.md"],
            false,
            "negation_no_match_txt",
        );
        test_is_ignored_case(
            "important.md",
            &["!important.md", "*.md"],
            true,
            "negation_order_matters1",
        );
        test_is_ignored_case(
            "foo/file.txt",
            &["foo/", "!foo/file.txt"],
            false,
            "negation_specific_file_in_ignored_dir",
        );
        test_is_ignored_case(
            "foo/other.txt",
            &["foo/", "!foo/file.txt"],
            true,
            "negation_other_file_in_ignored_dir",
        );
    }

    #[test]
    fn test_is_file_ignored_globstar() {
        test_is_ignored_case("foo/bar.txt", &["foo/**/bar.txt"], true, "globstar_simple");
        test_is_ignored_case(
            "foo/a/b/bar.txt",
            &["foo/**/bar.txt"],
            true,
            "globstar_deep",
        );
        test_is_ignored_case(
            "foo/a/b/other.txt",
            &["foo/**/bar.txt"],
            false,
            "globstar_no_match",
        );
        test_is_ignored_case(
            "foo/baz/bar.config",
            &["foo/**/bar.*"],
            true,
            "globstar_with_wildcard_ext",
        );
        test_is_ignored_case(
            "deep/logs/error.log",
            &["**/logs"],
            true,
            "globstar_dir_match1",
        );
        test_is_ignored_case(
            "deep/logs/error.log",
            &["**/logs/"],
            true,
            "globstar_dir_match2",
        );
        test_is_ignored_case(
            "other/file.txt",
            &["**/logs"],
            false,
            "globstar_dir_no_match",
        );
        test_is_ignored_case(
            "logs/error.log",
            &["**/logs/"],
            true,
            "globstar_dir_match_root_logs",
        );
    }

    #[test]
    fn test_is_file_ignored_precedence() {
        test_is_ignored_case(
            "debug.log",
            &["*.log", "!debug.log"],
            false,
            "precedence_negate",
        );
        test_is_ignored_case(
            "debug.log",
            &["!debug.log", "*.log"],
            true,
            "precedence_ignore_after_negate",
        );
        test_is_ignored_case(
            "foo/debug.log",
            &["*.log", "!foo/debug.log", "foo/*"],
            true,
            "precedence_complex1",
        );
        test_is_ignored_case(
            "foo/debug.log",
            &["foo/*", "!foo/debug.log"],
            false,
            "precedence_complex2",
        );
    }

    #[test]
    fn test_is_file_ignored_files_in_ignored_dir() {
        test_is_ignored_case("build/app.exe", &["build/"], true, "file_in_ignored_dir");
        test_is_ignored_case(
            "build/subdir/data",
            &["build/"],
            true,
            "nested_file_in_ignored_dir",
        );
    }

    #[test]
    fn test_is_file_ignored_negated_file_in_ignored_dir() {
        test_is_ignored_case(
            "build/special.dll",
            &["build/", "!build/special.dll"],
            false,
            "negated_file_in_ignored_dir",
        );
        test_is_ignored_case(
            "build/other.dll",
            &["build/", "!build/special.dll"],
            true,
            "other_file_in_ignored_dir_still_ignored",
        );
    }

    #[test]
    fn test_is_file_ignored_unicode_paths_and_patterns() {
        test_is_ignored_case("résumé.pdf", &["*.pdf"], true, "unicode_filename_pdf");
        test_is_ignored_case("Фото/image.jpg", &["Фото/"], true, "unicode_dirname_photo");
        test_is_ignored_case(
            "你好世界.txt",
            &["你好世界.txt"],
            true,
            "unicode_exact_match",
        );
        test_is_ignored_case(
            "café/menu.txt",
            &["café/*"],
            true,
            "unicode_pattern_wildcard",
        );
    }
}

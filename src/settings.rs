use dirs_next::config_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "repo_prompt_clone";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub recent_workspaces: Vec<PathBuf>,
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

impl Settings {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Settings::default()
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let default_path = Self::config_path();
        let config_path = self.config_path.as_ref().unwrap_or(&default_path);
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, contents)
    }

    pub fn add_recent_workspace(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_workspaces.retain(|p| p != &path);
        // Add to front
        self.recent_workspaces.insert(0, path);
        // Keep only last 5
        if self.recent_workspaces.len() > 5 {
            self.recent_workspaces.truncate(5);
        }
    }

    pub fn get_recent_workspaces(&self) -> &[PathBuf] {
        &self.recent_workspaces
    }

    fn config_path() -> PathBuf {
        config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_NAME)
            .join(SETTINGS_FILE)
    }

    #[cfg(test)]
    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_settings_load_save() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("settings.json");

        // Create test settings
        let mut settings = Settings::default().with_config_path(config_path.clone());
        settings.add_recent_workspace(PathBuf::from("/test/path"));

        // Save settings
        settings.save().unwrap();

        // Load settings
        let loaded = Settings::default().with_config_path(config_path);
        let contents = fs::read_to_string(&loaded.config_path.unwrap()).unwrap();
        let loaded = serde_json::from_str::<Settings>(&contents).unwrap();

        assert_eq!(loaded.recent_workspaces, settings.recent_workspaces);
    }

    #[test]
    fn test_recent_workspaces_limit() {
        let mut settings = Settings::default();

        // Add more than 5 workspaces
        for i in 0..10 {
            settings.add_recent_workspace(PathBuf::from(format!("/path/{}", i)));
        }

        // Should only keep the last 5
        assert_eq!(settings.recent_workspaces.len(), 5);
        assert_eq!(settings.recent_workspaces[0], PathBuf::from("/path/9"));
        assert_eq!(settings.recent_workspaces[4], PathBuf::from("/path/5"));
    }

    #[test]
    fn test_recent_workspaces_no_duplicates() {
        let mut settings = Settings::default();
        let path = PathBuf::from("/test/path");

        // Add same path multiple times
        settings.add_recent_workspace(path.clone());
        settings.add_recent_workspace(path.clone());
        settings.add_recent_workspace(path.clone());

        // Should only appear once
        assert_eq!(settings.recent_workspaces.len(), 1);
        assert_eq!(settings.recent_workspaces[0], path);
    }
}

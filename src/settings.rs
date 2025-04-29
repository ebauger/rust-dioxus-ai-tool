use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::tokenizer::TokenEstimator;
use dirs_next::config_dir;
use std::sync::Arc;

const APP_NAME: &str = "repo_prompt_clone";
const SETTINGS_FILE: &str = "settings.json";
const MAX_RECENT_WORKSPACES: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub recent_workspaces: Vec<PathBuf>,
    pub token_estimator: TokenEstimator,
    pub config_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recent_workspaces: Vec::new(),
            token_estimator: TokenEstimator::default(),
            config_path: None,
        }
    }
}

impl Settings {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            recent_workspaces: Vec::new(),
            token_estimator: TokenEstimator::default(),
            config_path: Some(config_path),
        }
    }

    pub fn add_recent_workspace(&mut self, path: PathBuf) {
        // Remove if already exists
        if let Some(pos) = self.recent_workspaces.iter().position(|x| x == &path) {
            self.recent_workspaces.remove(pos);
        }
        // Insert at the beginning
        self.recent_workspaces.insert(0, path);
        // Keep only the last 5
        if self.recent_workspaces.len() > 5 {
            self.recent_workspaces.pop();
        }
    }

    pub fn set_token_estimator(&mut self, estimator: TokenEstimator) {
        self.token_estimator = estimator;
    }

    pub fn get_token_estimator(&self) -> TokenEstimator {
        self.token_estimator.clone()
    }

    pub async fn save(&self) -> std::io::Result<()> {
        if let Some(path) = &self.config_path {
            let json = serde_json::to_string_pretty(self)?;
            tokio::fs::write(path, json).await?;
        }
        Ok(())
    }

    pub async fn load(path: &PathBuf) -> std::io::Result<Self> {
        if path.exists() {
            let json = tokio::fs::read_to_string(path).await?;
            let mut settings: Self = serde_json::from_str(&json)?;
            settings.config_path = Some(path.clone());
            Ok(settings)
        } else {
            Ok(Self::new(path.clone()))
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_settings_save_load() {
        let temp_dir = tempdir().unwrap();
        let settings_file = temp_dir.path().join("settings.json");
        let mut settings = Settings::new(settings_file.clone());

        // Add some recent workspaces
        settings.add_recent_workspace(PathBuf::from("/path/to/workspace1"));
        settings.add_recent_workspace(PathBuf::from("/path/to/workspace2"));

        // Save settings
        settings.save().await.unwrap();

        // Load settings
        let loaded_settings = Settings::load(&settings_file).await.unwrap();

        // Verify loaded settings
        assert_eq!(loaded_settings.recent_workspaces.len(), 2);
        assert_eq!(
            loaded_settings.recent_workspaces[0],
            PathBuf::from("/path/to/workspace2")
        );
        assert_eq!(
            loaded_settings.recent_workspaces[1],
            PathBuf::from("/path/to/workspace1")
        );
    }
}

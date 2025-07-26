#[cfg(feature = "config")]
pub mod settings;

#[cfg(feature = "config")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "config")]
use std::path::PathBuf;

/// Configuration options for file search operations
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct Config {
    /// Maximum depth to traverse in directory tree (None for unlimited)
    pub max_depth: Option<usize>,
    /// Whether to ignore hidden files and directories
    pub ignore_hidden: bool,
    /// Glob patterns to ignore during search
    pub ignore_patterns: Vec<String>,
    /// Whether search should be case-sensitive
    pub case_sensitive: bool,
    /// Maximum file size to consider (None for no limit)
    pub max_file_size: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_depth: None,
            ignore_hidden: true,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ],
            case_sensitive: false,
            max_file_size: None,
        }
    }
}

impl Config {
    #[cfg(feature = "config")]
    pub fn load_from_file(path: &PathBuf) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config =
            serde_json::from_str(&content).map_err(|e| crate::FileSearchError::InvalidConfig {
                reason: format!("Config serialize error: {}", e),
            })?;
        Ok(config)
    }

    #[cfg(feature = "config")]
    pub fn save_to_file(&self, path: &PathBuf) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            crate::FileSearchError::InvalidConfig {
                reason: format!("Config serialize error: {}", e),
            }
        })?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

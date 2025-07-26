#[cfg(feature = "config")]
pub mod settings;

#[cfg(feature = "config")]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct Config {
    pub max_depth: Option<usize>,
    pub ignore_hidden: bool,
    pub ignore_patterns: Vec<String>,
    pub case_sensitive: bool,
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
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| crate::FileSearchError::InvalidQuery(format!("Config parse error: {}", e)))?;
        Ok(config)
    }

    #[cfg(feature = "config")]
    pub fn save_to_file(&self, path: &PathBuf) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::FileSearchError::InvalidQuery(format!("Config serialize error: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
use super::Config;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
    config: Config,
}

impl ConfigManager {
    #[cfg(feature = "config")]
    pub fn new() -> crate::Result<Self> {
        let config_path = Self::default_config_path()?;
        let config = if config_path.exists() {
            Config::load_from_file(&config_path)?
        } else {
            Config::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    #[cfg(feature = "config")]
    pub fn save(&self) -> crate::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.config.save_to_file(&self.config_path)
    }

    #[cfg(feature = "config")]
    fn default_config_path() -> crate::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::error::FileSearchError::invalid_config("Could not determine config directory"))?;
        Ok(config_dir.join("file-search").join("config.json"))
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config_path: PathBuf::from("config.json"),
            config: Config::default(),
        })
    }
}
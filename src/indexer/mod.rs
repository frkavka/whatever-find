/// File system walker implementation
pub mod file_walker;

use crate::config::Config;
use crate::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// File index mapping filenames to their full paths
pub type FileIndex = HashMap<String, Vec<PathBuf>>;

/// File system indexer that builds searchable indexes of files
pub struct FileIndexer {
    config: Config,
}

impl FileIndexer {
    /// Create a new file indexer with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Build a complete file index from the given root path
    pub fn build_index(&mut self, root_path: &str) -> Result<FileIndex> {
        let mut index = HashMap::new();
        let walker = file_walker::FileWalker::new(&self.config);

        let entries = walker.walk(root_path)?;
        for entry_result in entries {
            let entry = entry_result?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    let key = if self.config.case_sensitive {
                        filename.to_string()
                    } else {
                        filename.to_lowercase()
                    };

                    index
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(path.to_path_buf());
                }
            }
        }

        Ok(index)
    }

    /// Check if a path should be ignored based on configuration
    pub fn should_ignore(&self, path: &Path) -> bool {
        if self.config.ignore_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    return true;
                }
            }
        }

        for pattern in &self.config.ignore_patterns {
            if self.matches_pattern(path, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, path: &Path, pattern: &str) -> bool {
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if pattern.contains('*') {
                let regex_pattern = pattern.replace("*", ".*");
                if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                    return regex.is_match(filename);
                }
            } else {
                return filename == pattern || path.to_string_lossy().contains(pattern);
            }
        }
        false
    }
}

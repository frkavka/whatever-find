use crate::config::Config;
use crate::Result;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// File system walker that respects configuration settings
pub struct FileWalker {
    config: Config,
}

impl FileWalker {
    /// Create a new file walker with the given configuration
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Walk the file system starting from `root_path`, respecting configuration
    pub fn walk(&self, root_path: &str) -> Result<Vec<walkdir::Result<DirEntry>>> {
        let mut walker = WalkDir::new(root_path);

        if let Some(max_depth) = self.config.max_depth {
            walker = walker.max_depth(max_depth);
        }

        let config = self.config.clone();
        let entries: Vec<_> = walker
            .into_iter()
            .filter_entry(move |e| !Self::should_skip_entry_with_config(e, &config))
            .collect();

        Ok(entries)
    }

    fn should_skip_entry_with_config(entry: &DirEntry, config: &Config) -> bool {
        let path = entry.path();

        if config.ignore_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    return true;
                }
            }
        }

        for pattern in &config.ignore_patterns {
            if Self::matches_pattern(path, pattern) {
                return true;
            }
        }

        if let Some(max_size) = config.max_file_size {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() > max_size {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn matches_pattern(path: &Path, pattern: &str) -> bool {
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

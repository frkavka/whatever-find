//! # File Search Library
//!
//! A fast, cross-platform file search library with smart pattern detection.
//!
//! ## Features
//!
//! - **Smart Pattern Detection**: Automatically detects glob, regex, or substring patterns
//! - **Multiple Search Modes**: Substring, glob, regex, and fuzzy matching
//! - **Cross-Platform**: Works on Windows, macOS, and Linux
//! - **High Performance**: Efficient file indexing and searching
//! - **Configurable**: Extensive configuration options for search behavior
//! - **Async Support**: Optional async operations with the `async` feature
//!
//! ## Quick Start
//!
//! ```rust
//! use file_search::{FileSearcher, SearchMode};
//! use std::path::Path;
//!
//! let searcher = FileSearcher::new();
//! let results = searcher.search_auto(Path::new("."), "*.rs").unwrap();
//! 
//! for file in results {
//!     println!("{}", file.display());
//! }
//! ```
//!
//! ## Search Modes
//!
//! The library supports several search modes:
//!
//! - **Auto**: Automatically detects the best search mode based on the pattern
//! - **Substring**: Simple substring matching
//! - **Glob**: Shell-style wildcards (`*`, `?`)
//! - **Regex**: Full regular expression support
//! - **Fuzzy**: Typo-tolerant fuzzy matching
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use file_search::FileSearcher;
//! use std::path::Path;
//!
//! let searcher = FileSearcher::new();
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Auto-detection (recommended)
//! let rs_files = searcher.search_auto(Path::new("."), "*.rs")?;
//! let test_files = searcher.search_auto(Path::new("."), "test_*")?;
//! let regex_files = searcher.search_auto(Path::new("."), r"\d{4}")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Manual Mode Selection
//!
//! ```rust
//! use file_search::{FileSearcher, SearchMode};
//! use std::path::Path;
//!
//! let searcher = FileSearcher::new();
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Force specific search modes
//! let files = searcher.search(Path::new("."), "config", SearchMode::Substring)?;
//! let files = searcher.search(Path::new("."), "*.txt", SearchMode::Glob)?;
//! let files = searcher.search(Path::new("."), r"\.rs$", SearchMode::Regex)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Fuzzy Search
//!
//! ```rust
//! use file_search::{FileSearcher, SearchMode};
//! use std::path::Path;
//!
//! let searcher = FileSearcher::new();
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Fuzzy search returns scored results
//! let scored_results = searcher.search_fuzzy(Path::new("."), "mian")?; // finds "main"
//! 
//! for (file, score) in scored_results {
//!     println!("{} (score: {:.2})", file.display(), score);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Builder Pattern
//!
//! ```rust
//! use file_search::FileSearcher;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let searcher = FileSearcher::builder()
//!     .max_depth(3)
//!     .ignore_hidden(false)
//!     .case_sensitive(true)
//!     .ignore_pattern("*.tmp")
//!     .ignore_pattern("target")
//!     .max_file_size(1024 * 1024) // 1MB
//!     .build()?;
//!
//! let results = searcher.search_auto(Path::new("."), "*.rs")?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Configuration management for file search operations
pub mod config;
/// File system indexing functionality
pub mod indexer;
/// Search engine implementation with various modes
pub mod search;
/// Error types and handling
pub mod error;

use std::path::{Path, PathBuf};

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, crate::error::FileSearchError>;

/// Builder pattern for creating `FileSearcher` instances with custom configuration
///
/// This builder provides a fluent interface for configuring search behavior.
///
/// # Examples
///
/// ```rust
/// use file_search::FileSearcherBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let searcher = FileSearcherBuilder::new()
///     .max_depth(3)
///     .ignore_hidden(false)
///     .case_sensitive(true)
///     .ignore_pattern("*.tmp")
///     .ignore_pattern("target")
///     .max_file_size(1024 * 1024) // 1MB
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileSearcherBuilder {
    config: crate::config::Config,
}

impl Default for FileSearcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcherBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: crate::config::Config::default(),
        }
    }

    /// Set the maximum depth for directory traversal
    ///
    /// # Arguments
    /// * `depth` - Maximum depth to traverse. `None` means unlimited depth.
    ///
    /// # Examples
    /// ```rust
    /// use file_search::FileSearcherBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let searcher = FileSearcherBuilder::new()
    ///     .max_depth(3)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = Some(depth);
        self
    }

    /// Set unlimited depth for directory traversal
    pub fn unlimited_depth(mut self) -> Self {
        self.config.max_depth = None;
        self
    }

    /// Set whether to ignore hidden files and directories
    ///
    /// # Arguments
    /// * `ignore` - If `true`, hidden files and directories will be ignored
    pub fn ignore_hidden(mut self, ignore: bool) -> Self {
        self.config.ignore_hidden = ignore;
        self
    }

    /// Set whether search should be case-sensitive
    ///
    /// # Arguments
    /// * `sensitive` - If `true`, search will be case-sensitive
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.config.case_sensitive = sensitive;
        self
    }

    /// Add a pattern to ignore during search
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern to ignore (e.g., "*.tmp", "target", ".git")
    ///
    /// # Examples
    /// ```rust
    /// use file_search::FileSearcherBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let searcher = FileSearcherBuilder::new()
    ///     .ignore_pattern("*.tmp")
    ///     .ignore_pattern("target")
    ///     .ignore_pattern(".git")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn ignore_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.config.ignore_patterns.push(pattern.into());
        self
    }

    /// Set multiple patterns to ignore during search
    ///
    /// # Arguments
    /// * `patterns` - Iterator of glob patterns to ignore
    pub fn ignore_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.ignore_patterns.extend(patterns.into_iter().map(Into::into));
        self
    }

    /// Clear all ignore patterns
    pub fn clear_ignore_patterns(mut self) -> Self {
        self.config.ignore_patterns.clear();
        self
    }

    /// Set the maximum file size to consider during search
    ///
    /// # Arguments
    /// * `size` - Maximum file size in bytes. `None` means no limit.
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.config.max_file_size = Some(size);
        self
    }

    /// Remove file size limit
    pub fn unlimited_file_size(mut self) -> Self {
        self.config.max_file_size = None;
        self
    }

    /// Set the configuration directly
    ///
    /// This overwrites any previously configured settings.
    pub fn config(mut self, config: crate::config::Config) -> Self {
        self.config = config;
        self
    }

    /// Validate the configuration and build the `FileSearcher`
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid
    pub fn build(self) -> Result<FileSearcher> {
        // Validate configuration
        if let Some(depth) = self.config.max_depth {
            if depth == 0 {
                return Err(crate::error::FileSearchError::invalid_config(
                    "max_depth cannot be 0. Use unlimited_depth() for no limit or set a positive value."
                ));
            }
        }

        if let Some(size) = self.config.max_file_size {
            if size == 0 {
                return Err(crate::error::FileSearchError::invalid_config(
                    "max_file_size cannot be 0. Use unlimited_file_size() for no limit or set a positive value."
                ));
            }
        }

        // Validate ignore patterns
        for pattern in &self.config.ignore_patterns {
            if pattern.is_empty() {
                return Err(crate::error::FileSearchError::invalid_config(
                    "ignore patterns cannot be empty"
                ));
            }
        }

        Ok(FileSearcher {
            config: self.config,
        })
    }

    /// Build the `FileSearcher` without validation
    ///
    /// This method skips configuration validation and should only be used
    /// when you're certain the configuration is valid.
    pub fn build_unchecked(self) -> FileSearcher {
        FileSearcher {
            config: self.config,
        }
    }
}

/// Main entry point for the file search library
///
/// This struct provides a high-level interface for searching files with various patterns.
/// It handles indexing, pattern detection, and search execution.
#[derive(Debug)]
pub struct FileSearcher {
    config: crate::config::Config,
}

impl Default for FileSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher {
    /// Creates a new `FileSearcher` with default configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::FileSearcher;
    ///
    /// let searcher = FileSearcher::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: crate::config::Config::default(),
        }
    }

    /// Creates a new `FileSearcherBuilder` for fluent configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::FileSearcher;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let searcher = FileSearcher::builder()
    ///     .max_depth(3)
    ///     .ignore_hidden(false)
    ///     .case_sensitive(true)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> FileSearcherBuilder {
        FileSearcherBuilder::new()
    }

    /// Creates a new `FileSearcher` with custom configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::{FileSearcher, Config};
    ///
    /// let config = Config {
    ///     case_sensitive: true,
    ///     max_depth: Some(3),
    ///     ..Default::default()
    /// };
    /// let searcher = FileSearcher::with_config(config);
    /// ```
    pub fn with_config(config: crate::config::Config) -> Self {
        Self { config }
    }

    /// Searches for files using automatic pattern detection
    ///
    /// This method automatically detects whether the query is a glob pattern,
    /// regular expression, or simple substring and uses the appropriate search mode.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The directory to search in
    /// * `query` - The search pattern
    ///
    /// # Returns
    ///
    /// A vector of matching file paths
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or if the pattern is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::FileSearcher;
    /// use std::path::Path;
    ///
    /// let searcher = FileSearcher::new();
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = searcher.search_auto(Path::new("."), "*.rs")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_auto(&self, root_path: &Path, query: &str) -> Result<Vec<PathBuf>> {
        let mut indexer = crate::indexer::FileIndexer::new(self.config.clone());
        let index = indexer.build_index(root_path.to_str().ok_or_else(|| crate::error::FileSearchError::invalid_path(root_path, "Contains invalid UTF-8"))?)?;
        
        let search_engine = crate::search::SearchEngine::new(self.config.clone());
        search_engine.search_auto(&index, query)
    }

    /// Searches for files using automatic pattern detection, returning the detected mode
    ///
    /// Similar to `search_auto`, but also returns information about which search mode
    /// was automatically selected.
    ///
    /// # Returns
    ///
    /// A tuple containing the matching file paths and the detected search mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::{FileSearcher, SearchMode};
    /// use std::path::Path;
    ///
    /// let searcher = FileSearcher::new();
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (results, mode) = searcher.search_auto_with_mode(Path::new("."), "*.rs")?;
    /// assert_eq!(mode, SearchMode::Glob);
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_auto_with_mode(&self, root_path: &Path, query: &str) -> Result<(Vec<PathBuf>, crate::search::SearchMode)> {
        let mut indexer = crate::indexer::FileIndexer::new(self.config.clone());
        let index = indexer.build_index(root_path.to_str().ok_or_else(|| crate::error::FileSearchError::invalid_path(root_path, "Contains invalid UTF-8"))?)?;
        
        let search_engine = crate::search::SearchEngine::new(self.config.clone());
        search_engine.search_auto_with_mode(&index, query)
    }

    /// Searches for files using a specific search mode
    ///
    /// This method allows you to force a specific search mode, bypassing automatic detection.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The directory to search in
    /// * `query` - The search pattern
    /// * `mode` - The search mode to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::{FileSearcher, SearchMode};
    /// use std::path::Path;
    ///
    /// let searcher = FileSearcher::new();
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = searcher.search(Path::new("."), "test", SearchMode::Substring)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn search(&self, root_path: &Path, query: &str, mode: crate::search::SearchMode) -> Result<Vec<PathBuf>> {
        let mut indexer = crate::indexer::FileIndexer::new(self.config.clone());
        let index = indexer.build_index(root_path.to_str().ok_or_else(|| crate::error::FileSearchError::invalid_path(root_path, "Contains invalid UTF-8"))?)?;
        
        let search_engine = crate::search::SearchEngine::new(self.config.clone());
        
        match mode {
            crate::search::SearchMode::Substring => Ok(search_engine.search_substring(&index, query)),
            crate::search::SearchMode::Glob => search_engine.search_glob(&index, query),
            crate::search::SearchMode::Regex => search_engine.search_regex(&index, query),
            crate::search::SearchMode::Fuzzy => Ok(search_engine.search_fuzzy(&index, query).into_iter().map(|(path, _)| path).collect()),
        }
    }

    /// Performs fuzzy search and returns scored results
    ///
    /// Fuzzy search is tolerant of typos and returns results ranked by relevance score.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing file paths and their relevance scores (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_search::FileSearcher;
    /// use std::path::Path;
    ///
    /// let searcher = FileSearcher::new();
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = searcher.search_fuzzy(Path::new("."), "mian")?; // finds "main"
    /// 
    /// for (file, score) in results {
    ///     println!("{} (score: {:.2})", file.display(), score);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_fuzzy(&self, root_path: &Path, query: &str) -> Result<Vec<(PathBuf, f64)>> {
        let mut indexer = crate::indexer::FileIndexer::new(self.config.clone());
        let index = indexer.build_index(root_path.to_str().ok_or_else(|| crate::error::FileSearchError::invalid_path(root_path, "Contains invalid UTF-8"))?)?;
        
        let search_engine = crate::search::SearchEngine::new(self.config.clone());
        Ok(search_engine.search_fuzzy(&index, query))
    }

    /// Gets the current configuration
    pub fn config(&self) -> &crate::config::Config {
        &self.config
    }

    /// Updates the configuration
    pub fn set_config(&mut self, config: crate::config::Config) {
        self.config = config;
    }

    /// Asynchronous version of `search_auto`
    ///
    /// This method runs the search operation on a background thread to avoid blocking
    /// the current thread. Requires the `async` feature to be enabled.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use file_search::FileSearcher;
    /// use std::path::Path;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let searcher = FileSearcher::new();
    ///     let results = searcher.search_auto_async(Path::new("."), "*.rs").await?;
    ///     println!("Found {} files", results.len());
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "async")]
    pub async fn search_auto_async(&self, root_path: &Path, query: &str) -> Result<Vec<PathBuf>> {
        let searcher = self.clone();
        let root_path = root_path.to_path_buf();
        let query = query.to_string();
        
        tokio::task::spawn_blocking(move || {
            searcher.search_auto(&root_path, &query)
        }).await.map_err(|e| crate::error::FileSearchError::invalid_config(
            format!("Async task failed: {}", e)
        ))?
    }

    /// Asynchronous version of `search_auto_with_mode`
    #[cfg(feature = "async")]
    pub async fn search_auto_with_mode_async(&self, root_path: &Path, query: &str) -> Result<(Vec<PathBuf>, crate::search::SearchMode)> {
        let searcher = self.clone();
        let root_path = root_path.to_path_buf();
        let query = query.to_string();
        
        tokio::task::spawn_blocking(move || {
            searcher.search_auto_with_mode(&root_path, &query)
        }).await.map_err(|e| crate::error::FileSearchError::invalid_config(
            format!("Async task failed: {}", e)
        ))?
    }

    /// Asynchronous version of `search`
    #[cfg(feature = "async")]
    pub async fn search_async(&self, root_path: &Path, query: &str, mode: crate::search::SearchMode) -> Result<Vec<PathBuf>> {
        let searcher = self.clone();
        let root_path = root_path.to_path_buf();
        let query = query.to_string();
        
        tokio::task::spawn_blocking(move || {
            searcher.search(&root_path, &query, mode)
        }).await.map_err(|e| crate::error::FileSearchError::invalid_config(
            format!("Async task failed: {}", e)
        ))?
    }

    /// Asynchronous version of `search_fuzzy`
    #[cfg(feature = "async")]
    pub async fn search_fuzzy_async(&self, root_path: &Path, query: &str) -> Result<Vec<(PathBuf, f64)>> {
        let searcher = self.clone();
        let root_path = root_path.to_path_buf();
        let query = query.to_string();
        
        tokio::task::spawn_blocking(move || {
            searcher.search_fuzzy(&root_path, &query)
        }).await.map_err(|e| crate::error::FileSearchError::invalid_config(
            format!("Async task failed: {}", e)
        ))?
    }
}

// Clone implementation needed for async support
impl Clone for FileSearcher {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

// Re-export commonly used types
pub use crate::indexer::FileIndex;
pub use crate::search::SearchMode;
pub use crate::config::Config;
pub use crate::error::FileSearchError;

// FileSearcherBuilder is already defined in this module, no need to re-export

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("lib.rs"), "pub mod lib;").unwrap();
        fs::write(root.join("config.toml"), "[config]").unwrap();
        fs::write(root.join("README.md"), "# Test").unwrap();
        
        // Create subdirectory
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src").join("test.rs"), "test code").unwrap();
        fs::write(root.join("src").join("helper.rs"), "helper code").unwrap();
        
        // Create hidden file
        fs::write(root.join(".hidden"), "hidden content").unwrap();

        temp_dir
    }

    fn test_config() -> crate::config::Config {
        crate::config::Config {
            ignore_hidden: false,
            ignore_patterns: vec![], // Clear all ignore patterns for testing
            case_sensitive: false,
            max_depth: None,
            max_file_size: None,
        }
    }

    #[test]
    fn test_basic_search() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        let results = searcher.search_auto(temp_dir.path(), "*.rs").unwrap();
        // Should find main.rs, lib.rs, src/test.rs, src/helper.rs
        assert!(results.len() >= 4, "Expected at least 4 .rs files, found {}", results.len());
    }

    #[test]
    fn test_substring_search() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        let results = searcher.search(temp_dir.path(), "main", SearchMode::Substring).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].file_name().unwrap().to_str().unwrap().contains("main"));
    }

    #[test]
    fn test_glob_search() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        let results = searcher.search(temp_dir.path(), "*.rs", SearchMode::Glob).unwrap();
        assert!(results.len() >= 4);
    }

    #[test]
    fn test_regex_search() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        let results = searcher.search(temp_dir.path(), r".*\.rs$", SearchMode::Regex).unwrap();
        assert!(results.len() >= 4);
    }

    #[test]
    fn test_fuzzy_search() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        let results = searcher.search_fuzzy(temp_dir.path(), "man").unwrap(); // should find "main"
        assert!(!results.is_empty());
        
        // Check that results are scored
        for (_, score) in &results {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
        
        // Verify we found main.rs
        let found_main = results.iter().any(|(path, _)| {
            path.file_name().unwrap().to_str().unwrap() == "main.rs"
        });
        assert!(found_main, "Should find main.rs with fuzzy search 'man'");
    }

    #[test]
    fn test_auto_detection() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        // Should detect as glob
        let (results, mode) = searcher.search_auto_with_mode(temp_dir.path(), "*.rs").unwrap();
        assert_eq!(mode, SearchMode::Glob);
        assert!(results.len() >= 4);
        
        // Should detect as regex
        let (results, mode) = searcher.search_auto_with_mode(temp_dir.path(), r"\.rs$").unwrap();
        assert_eq!(mode, SearchMode::Regex);
        assert!(results.len() >= 4);
        
        // Should detect as substring
        let (results, mode) = searcher.search_auto_with_mode(temp_dir.path(), "main").unwrap();
        assert_eq!(mode, SearchMode::Substring);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_builder_pattern() {
        let temp_dir = create_test_structure();
        
        // Test that the builder pattern works
        let searcher = FileSearcher::builder()
            .ignore_hidden(false)
            .clear_ignore_patterns() // Clear defaults first
            .case_sensitive(false)
            .build()
            .unwrap();
        
        let results = searcher.search_auto(temp_dir.path(), "*.rs").unwrap();
        // Should find all .rs files with builder configuration
        assert!(results.len() >= 4, "Builder pattern should work correctly");
    }

    #[test]
    fn test_ignore_patterns() {
        let temp_dir = create_test_structure();
        
        let searcher = FileSearcher::builder()
            .ignore_hidden(false)
            .clear_ignore_patterns() // Clear defaults first
            .ignore_pattern("*.md")
            .build()
            .unwrap();
        
        let results = searcher.search_auto(temp_dir.path(), "*").unwrap();
        // Should not include README.md
        assert!(!results.iter().any(|p| p.file_name().unwrap().to_str().unwrap().ends_with(".md")));
    }

    #[test]
    fn test_case_sensitivity() {
        let temp_dir = create_test_structure();
        
        // Case insensitive (default)
        let searcher = FileSearcher::with_config(test_config());
        let results = searcher.search(temp_dir.path(), "MAIN", SearchMode::Substring).unwrap();
        assert_eq!(results.len(), 1);
        
        // Case sensitive
        let searcher = FileSearcher::builder()
            .ignore_hidden(false)
            .clear_ignore_patterns() // Clear defaults first
            .case_sensitive(true)
            .build()
            .unwrap();
        let results = searcher.search(temp_dir.path(), "MAIN", SearchMode::Substring).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_builder_validation() {
        // Test invalid max_depth
        let result = FileSearcher::builder()
            .max_depth(0)
            .build();
        assert!(result.is_err());
        
        // Test invalid max_file_size
        let result = FileSearcher::builder()
            .max_file_size(0)
            .build();
        assert!(result.is_err());
        
        // Test empty ignore pattern
        let mut builder = FileSearcher::builder();
        builder.config.ignore_patterns.push(String::new());
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling() {
        let searcher = FileSearcher::with_config(test_config());
        
        // Test with non-existent path
        let result = searcher.search_auto(Path::new("/non/existent/path"), "*.rs");
        assert!(result.is_err());
        
        // Test with invalid regex
        let temp_dir = create_test_structure();
        let result = searcher.search(temp_dir.path(), "[invalid", SearchMode::Regex);
        assert!(result.is_err());
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_search() {
        let temp_dir = create_test_structure();
        let searcher = FileSearcher::with_config(test_config());
        
        let results = searcher.search_auto_async(temp_dir.path(), "*.rs").await.unwrap();
        assert!(results.len() >= 4);
    }
}
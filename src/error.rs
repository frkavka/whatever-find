//! Error types for the file search library

use std::fmt;
use std::path::PathBuf;

/// Main error type for the file search library
#[derive(Debug)]
pub enum FileSearchError {
    /// IO error occurred during file operations
    Io {
        /// The IO error that occurred
        source: std::io::Error,
        /// Context about what operation failed
        context: String,
        /// Optional path where the error occurred
        path: Option<PathBuf>,
    },
    /// Invalid regular expression pattern
    InvalidRegex {
        /// The regex error
        source: regex::Error,
        /// The pattern that failed to compile
        pattern: String,
    },
    /// Invalid glob pattern
    InvalidGlob {
        /// The glob pattern error
        source: glob::PatternError,
        /// The pattern that failed to compile
        pattern: String,
    },
    /// WalkDir error during directory traversal
    WalkDir {
        /// The walkdir error
        source: walkdir::Error,
        /// The root path being traversed
        root_path: PathBuf,
    },
    /// Index is empty or invalid
    EmptyIndex {
        /// The path that was indexed
        path: PathBuf,
    },
    /// Search query is empty or invalid
    InvalidQuery {
        /// Description of why the query is invalid
        reason: String,
        /// The invalid query
        query: String,
    },
    /// Path conversion error
    InvalidPath {
        /// The path that couldn't be converted
        path: PathBuf,
        /// Reason for the failure
        reason: String,
    },
    /// Configuration error
    InvalidConfig {
        /// Description of the configuration issue
        reason: String,
    },
}

impl fmt::Display for FileSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { source, context, path } => {
                if let Some(path) = path {
                    write!(f, "IO error in {}: {} (path: {})", context, source, path.display())
                } else {
                    write!(f, "IO error in {}: {}", context, source)
                }
            }
            Self::InvalidRegex { source, pattern } => {
                write!(f, "Invalid regex pattern '{}': {}", pattern, source)
            }
            Self::InvalidGlob { source, pattern } => {
                write!(f, "Invalid glob pattern '{}': {}", pattern, source)
            }
            Self::WalkDir { source, root_path } => {
                write!(f, "Directory traversal error in '{}': {}", root_path.display(), source)
            }
            Self::EmptyIndex { path } => {
                write!(f, "Search index is empty for path: {}", path.display())
            }
            Self::InvalidQuery { reason, query } => {
                write!(f, "Invalid search query '{}': {}", query, reason)
            }
            Self::InvalidPath { path, reason } => {
                write!(f, "Invalid path '{}': {}", path.display(), reason)
            }
            Self::InvalidConfig { reason } => {
                write!(f, "Invalid configuration: {}", reason)
            }
        }
    }
}

impl std::error::Error for FileSearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidRegex { source, .. } => Some(source),
            Self::InvalidGlob { source, .. } => Some(source),
            Self::WalkDir { source, .. } => Some(source),
            Self::EmptyIndex { .. } 
            | Self::InvalidQuery { .. } 
            | Self::InvalidPath { .. }
            | Self::InvalidConfig { .. } => None,
        }
    }
}

// Helper methods for creating errors with context
impl FileSearchError {
    /// Create an IO error with context
    pub fn io_error<S: Into<String>>(source: std::io::Error, context: S) -> Self {
        Self::Io {
            source,
            context: context.into(),
            path: None,
        }
    }

    /// Create an IO error with context and path
    pub fn io_error_with_path<S: Into<String>, P: Into<PathBuf>>(
        source: std::io::Error,
        context: S,
        path: P,
    ) -> Self {
        Self::Io {
            source,
            context: context.into(),
            path: Some(path.into()),
        }
    }

    /// Create a regex error with pattern
    pub fn regex_error<S: Into<String>>(source: regex::Error, pattern: S) -> Self {
        Self::InvalidRegex {
            source,
            pattern: pattern.into(),
        }
    }

    /// Create a glob error with pattern
    pub fn glob_error<S: Into<String>>(source: glob::PatternError, pattern: S) -> Self {
        Self::InvalidGlob {
            source,
            pattern: pattern.into(),
        }
    }

    /// Create a walkdir error with root path
    pub fn walkdir_error<P: Into<PathBuf>>(source: walkdir::Error, root_path: P) -> Self {
        Self::WalkDir {
            source,
            root_path: root_path.into(),
        }
    }

    /// Create an empty index error
    pub fn empty_index<P: Into<PathBuf>>(path: P) -> Self {
        Self::EmptyIndex {
            path: path.into(),
        }
    }

    /// Create an invalid query error
    pub fn invalid_query<R: Into<String>, Q: Into<String>>(reason: R, query: Q) -> Self {
        Self::InvalidQuery {
            reason: reason.into(),
            query: query.into(),
        }
    }

    /// Create an invalid path error
    pub fn invalid_path<P: Into<PathBuf>, R: Into<String>>(path: P, reason: R) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid config error
    pub fn invalid_config<R: Into<String>>(reason: R) -> Self {
        Self::InvalidConfig {
            reason: reason.into(),
        }
    }
}

// Keep simple From implementations for backward compatibility
impl From<std::io::Error> for FileSearchError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(err, "IO operation")
    }
}

impl From<regex::Error> for FileSearchError {
    fn from(err: regex::Error) -> Self {
        Self::regex_error(err, "<unknown pattern>")
    }
}

impl From<glob::PatternError> for FileSearchError {
    fn from(err: glob::PatternError) -> Self {
        Self::glob_error(err, "<unknown pattern>")
    }
}

impl From<walkdir::Error> for FileSearchError {
    fn from(err: walkdir::Error) -> Self {
        Self::walkdir_error(err, "<unknown path>")
    }
}
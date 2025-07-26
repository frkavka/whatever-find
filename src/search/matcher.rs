use regex::Regex;
use std::path::Path;

/// Types of pattern matching supported
pub enum MatchType {
    /// Exact string matching
    Exact,
    /// Substring matching
    Substring,
    /// Regular expression matching
    Regex,
    /// Fuzzy matching with typo tolerance
    Fuzzy,
}

/// Pattern matcher with configurable matching behavior
pub struct Matcher {
    match_type: MatchType,
    case_sensitive: bool,
    compiled_regex: Option<Regex>,
}

impl Matcher {
    /// Create a new matcher with the specified type and case sensitivity
    pub fn new(match_type: MatchType, case_sensitive: bool) -> Self {
        Self {
            match_type,
            case_sensitive,
            compiled_regex: None,
        }
    }

    /// Create a new regex matcher with a pre-compiled pattern
    pub fn with_regex(pattern: &str, case_sensitive: bool) -> Result<Self, regex::Error> {
        let flags = if case_sensitive { "" } else { "(?i)" };
        let full_pattern = format!("{}{}", flags, pattern);
        let regex = Regex::new(&full_pattern)?;

        Ok(Self {
            match_type: MatchType::Regex,
            case_sensitive,
            compiled_regex: Some(regex),
        })
    }

    /// Check if the filename matches the query using the configured match type
    pub fn matches(&self, filename: &str, query: &str) -> bool {
        match self.match_type {
            MatchType::Exact => self.exact_match(filename, query),
            MatchType::Substring => self.substring_match(filename, query),
            MatchType::Regex => self.regex_match(filename),
            MatchType::Fuzzy => self.fuzzy_match(filename, query) > 0.0,
        }
    }

    /// Calculate fuzzy matching score (0.0 to 1.0, higher is better)
    pub fn fuzzy_score(&self, filename: &str, query: &str) -> f64 {
        if matches!(self.match_type, MatchType::Fuzzy) {
            self.fuzzy_match(filename, query)
        } else if self.matches(filename, query) {
            1.0
        } else {
            0.0
        }
    }

    fn exact_match(&self, filename: &str, query: &str) -> bool {
        if self.case_sensitive {
            filename == query
        } else {
            filename.to_lowercase() == query.to_lowercase()
        }
    }

    fn substring_match(&self, filename: &str, query: &str) -> bool {
        if self.case_sensitive {
            filename.contains(query)
        } else {
            filename.to_lowercase().contains(&query.to_lowercase())
        }
    }

    fn regex_match(&self, filename: &str) -> bool {
        if let Some(ref regex) = self.compiled_regex {
            regex.is_match(filename)
        } else {
            false
        }
    }

    fn fuzzy_match(&self, filename: &str, query: &str) -> f64 {
        let filename = if self.case_sensitive {
            filename.to_string()
        } else {
            filename.to_lowercase()
        };

        let query = if self.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        if filename == query {
            return 1.0;
        }

        if filename.contains(&query) {
            return 0.8;
        }

        let mut score = 0.0;
        let query_chars: Vec<char> = query.chars().collect();
        let filename_chars: Vec<char> = filename.chars().collect();

        let mut query_idx = 0;
        let mut consecutive = 0;

        for &ch in filename_chars.iter() {
            if query_idx < query_chars.len() && ch == query_chars[query_idx] {
                query_idx += 1;
                consecutive += 1;
                score += 0.1 + (consecutive as f64 * 0.05);
            } else {
                consecutive = 0;
            }
        }

        if query_idx == query_chars.len() {
            score / filename_chars.len() as f64
        } else {
            0.0
        }
    }
}

/// Utility function to match a path against a pattern (glob or substring)
pub fn matches_path_pattern(path: &Path, pattern: &str) -> bool {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if pattern.contains('*') || pattern.contains('?') {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                return glob.matches(filename);
            }
        }
        filename.contains(pattern)
    } else {
        false
    }
}

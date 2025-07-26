/// Pattern matching implementations
pub mod matcher;

use crate::config::Config;
use crate::indexer::FileIndex;
use crate::Result;
use glob::Pattern;
use regex::Regex;
use std::path::PathBuf;

/// Search modes supported by the search engine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    /// Simple substring matching
    Substring,
    /// Shell-style glob patterns with wildcards
    Glob,
    /// Full regular expression support
    Regex,
    /// Fuzzy matching with typo tolerance
    Fuzzy,
}

/// Search engine that supports multiple search modes and automatic pattern detection
pub struct SearchEngine {
    config: Config,
}

impl SearchEngine {
    /// Create a new search engine with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Auto-detect the best search mode based on the query pattern
    pub fn detect_search_mode(&self, query: &str) -> SearchMode {
        // Check for regex patterns first (more specific)
        if self.looks_like_regex(query) {
            return SearchMode::Regex;
        }

        // Check for glob patterns
        if self.looks_like_glob(query) {
            return SearchMode::Glob;
        }

        // Default to substring for simple queries
        SearchMode::Substring
    }

    fn looks_like_regex(&self, query: &str) -> bool {
        // Common regex metacharacters that are unlikely to be in normal filenames
        let regex_indicators = [
            r"\d", r"\w", r"\s", r"\.", r"\^", r"\$", // Escape sequences
            "^", "$", // Anchors
            "[", "]", // Character classes
            "{", "}", // Quantifiers (only if containing digits)
            "+", // One or more (when not at start)
            "?", // Zero or one (when not simple glob)
            "|", // Alternation
            "(", ")", // Groups
        ];

        // Check for escape sequences
        if query.contains('\\') {
            for indicator in &regex_indicators[0..6] {
                if query.contains(indicator) {
                    return true;
                }
            }
        }

        // Check for anchors
        if query.starts_with('^') || query.ends_with('$') {
            return true;
        }

        // Check for character classes
        if query.contains('[') && query.contains(']') {
            return true;
        }

        // Check for quantifiers with braces
        if query.contains('{') && query.contains('}') && query.chars().any(|c| c.is_ascii_digit()) {
            return true;
        }

        // Check for alternation
        if query.contains('|') {
            return true;
        }

        // Check for groups
        if query.contains('(') && query.contains(')') {
            return true;
        }

        // Check for + quantifier (but not at the start where it might be a filename)
        if query.len() > 1 && query[1..].contains('+') {
            return true;
        }

        false
    }

    fn looks_like_glob(&self, query: &str) -> bool {
        // Glob patterns contain * or ? but don't look like regex
        if !query.contains('*') && !query.contains('?') {
            return false;
        }

        // If it looks like regex, prefer regex over glob
        if self.looks_like_regex(query) {
            return false;
        }

        // Simple heuristics for glob vs regex with wildcards
        // Globs usually have simpler patterns
        let has_glob_chars = query.contains('*') || query.contains('?');
        let has_complex_regex = query.contains('[')
            || query.contains('(')
            || query.contains('\\')
            || query.contains('|');

        has_glob_chars && !has_complex_regex
    }

    /// Smart search that auto-detects the pattern type
    pub fn search_auto(&self, index: &FileIndex, query: &str) -> Result<Vec<PathBuf>> {
        let mode = self.detect_search_mode(query);

        match mode {
            SearchMode::Regex => self.search_regex(index, query),
            SearchMode::Glob => self.search_glob(index, query),
            SearchMode::Substring => Ok(self.search_substring(index, query)),
            SearchMode::Fuzzy => Ok(self
                .search_fuzzy(index, query)
                .into_iter()
                .map(|(path, _)| path)
                .collect()),
        }
    }

    /// Smart search with mode information returned
    pub fn search_auto_with_mode(
        &self,
        index: &FileIndex,
        query: &str,
    ) -> Result<(Vec<PathBuf>, SearchMode)> {
        let mode = self.detect_search_mode(query);
        let results = match mode {
            SearchMode::Regex => self.search_regex(index, query)?,
            SearchMode::Glob => self.search_glob(index, query)?,
            SearchMode::Substring => self.search_substring(index, query),
            SearchMode::Fuzzy => self
                .search_fuzzy(index, query)
                .into_iter()
                .map(|(path, _)| path)
                .collect(),
        };

        Ok((results, mode))
    }

    /// Search using substring matching
    pub fn search_substring(&self, index: &FileIndex, query: &str) -> Vec<PathBuf> {
        let search_query = if self.config.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut results = Vec::new();

        for (filename, paths) in index {
            let search_target = if self.config.case_sensitive {
                filename.clone()
            } else {
                filename.to_lowercase()
            };

            if search_target.contains(&search_query) {
                results.extend(paths.clone());
            }
        }

        results.sort();
        results
    }

    /// Search using regular expressions
    pub fn search_regex(&self, index: &FileIndex, pattern: &str) -> Result<Vec<PathBuf>> {
        let flags = if self.config.case_sensitive {
            ""
        } else {
            "(?i)"
        };

        let full_pattern = format!("{}{}", flags, pattern);
        let regex = Regex::new(&full_pattern)?;

        let mut results = Vec::new();

        for (filename, paths) in index {
            if regex.is_match(filename) {
                results.extend(paths.clone());
            }
        }

        results.sort();
        Ok(results)
    }

    /// Search using glob patterns
    pub fn search_glob(&self, index: &FileIndex, pattern: &str) -> Result<Vec<PathBuf>> {
        let glob_pattern = if self.config.case_sensitive {
            Pattern::new(pattern)?
        } else {
            // For case-insensitive matching, we'll need to check both the pattern and filenames
            Pattern::new(&pattern.to_lowercase())?
        };

        let mut results = Vec::new();

        for (filename, paths) in index {
            let matches = if self.config.case_sensitive {
                glob_pattern.matches(filename)
            } else {
                glob_pattern.matches(&filename.to_lowercase())
            };

            if matches {
                results.extend(paths.clone());
            }
        }

        results.sort();
        Ok(results)
    }

    /// Search using fuzzy matching with typo tolerance
    pub fn search_fuzzy(&self, index: &FileIndex, query: &str) -> Vec<(PathBuf, f64)> {
        let mut scored_results = Vec::new();

        for (filename, paths) in index {
            let score = self.calculate_fuzzy_score(filename, query);
            if score > 0.0 {
                for path in paths {
                    scored_results.push((path.clone(), score));
                }
            }
        }

        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored_results
    }

    fn calculate_fuzzy_score(&self, filename: &str, query: &str) -> f64 {
        let filename_lower = if self.config.case_sensitive {
            filename.to_string()
        } else {
            filename.to_lowercase()
        };

        let query_lower = if self.config.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        // Exact match
        if filename_lower == query_lower {
            return 1.0;
        }

        // Substring match
        if filename_lower.contains(&query_lower) {
            return 0.9
                - (filename_lower.len() as f64 - query_lower.len() as f64)
                    / filename_lower.len() as f64
                    * 0.1;
        }

        // Calculate multiple scoring methods and combine them
        let levenshtein_score = self.levenshtein_score(&filename_lower, &query_lower);
        let subsequence_score = self.subsequence_score(&filename_lower, &query_lower);
        let ngram_score = self.ngram_score(&filename_lower, &query_lower);

        // Combine scores with weights
        let combined_score =
            (levenshtein_score * 0.4) + (subsequence_score * 0.4) + (ngram_score * 0.2);

        // Only return meaningful scores
        if combined_score < 0.3 {
            0.0
        } else {
            combined_score
        }
    }

    fn levenshtein_score(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return if len2 == 0 { 1.0 } else { 0.0 };
        }
        if len2 == 0 {
            return 0.0;
        }

        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();

        let mut prev_row: Vec<usize> = (0..=len2).collect();
        let mut curr_row = vec![0; len2 + 1];

        for i in 1..=len1 {
            curr_row[0] = i;
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                curr_row[j] = std::cmp::min(
                    std::cmp::min(curr_row[j - 1] + 1, prev_row[j] + 1),
                    prev_row[j - 1] + cost,
                );
            }
            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        let distance = prev_row[len2];
        let max_len = std::cmp::max(len1, len2);

        if max_len == 0 {
            1.0
        } else {
            1.0 - (distance as f64 / max_len as f64)
        }
    }

    fn subsequence_score(&self, filename: &str, query: &str) -> f64 {
        let filename_chars: Vec<char> = filename.chars().collect();
        let query_chars: Vec<char> = query.chars().collect();

        if query_chars.is_empty() {
            return 1.0;
        }

        let mut query_idx = 0;
        let mut consecutive = 0;
        let mut max_consecutive = 0;
        let mut score = 0.0;

        for &ch in filename_chars.iter() {
            if query_idx < query_chars.len() && ch == query_chars[query_idx] {
                query_idx += 1;
                consecutive += 1;
                max_consecutive = std::cmp::max(max_consecutive, consecutive);
                score += 1.0 + (consecutive as f64 * 0.1); // Bonus for consecutive matches
            } else {
                consecutive = 0;
            }
        }

        if query_idx == query_chars.len() {
            let coverage = score / filename_chars.len() as f64;
            let completeness = query_idx as f64 / query_chars.len() as f64;
            let consecutiveness = max_consecutive as f64 / query_chars.len() as f64;

            (coverage * 0.4) + (completeness * 0.4) + (consecutiveness * 0.2)
        } else {
            0.0
        }
    }

    fn ngram_score(&self, s1: &str, s2: &str) -> f64 {
        const N: usize = 2; // bigrams

        let ngrams1 = self.get_ngrams(s1, N);
        let ngrams2 = self.get_ngrams(s2, N);

        if ngrams1.is_empty() && ngrams2.is_empty() {
            return 1.0;
        }
        if ngrams1.is_empty() || ngrams2.is_empty() {
            return 0.0;
        }

        let mut common = 0;
        for ngram in &ngrams1 {
            if ngrams2.contains(ngram) {
                common += 1;
            }
        }

        let total = std::cmp::max(ngrams1.len(), ngrams2.len());
        common as f64 / total as f64
    }

    fn get_ngrams(&self, s: &str, n: usize) -> Vec<String> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() < n {
            return vec![s.to_string()];
        }

        chars
            .windows(n)
            .map(|window| window.iter().collect())
            .collect()
    }
}

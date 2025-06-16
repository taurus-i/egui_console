use std::collections::HashMap;
use std::cmp::Ordering;

use crate::ConsoleWindow;

/// Result from a search operation
#[derive(Clone)]
pub struct SearchResult {
    /// The matching command
    pub command: String,
    /// The relevance score (higher is better)
    pub score: f32,
    /// Position where the command is found in history
    pub position: usize,
}

/// Enhanced search functionality for the terminal
pub struct SearchEngine {
    /// Previous search results cache
    cache: HashMap<String, Vec<SearchResult>>,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Search the command history with fuzzy matching
    pub fn search(&mut self, query: &str, history: &[String]) -> Vec<SearchResult> {
        // Check cache first
        if let Some(results) = self.cache.get(query) {
            return results.clone();
        }

        // If query is empty, return all history items
        if query.is_empty() {
            let results: Vec<SearchResult> = history
                .iter()
                .enumerate()
                .map(|(i, cmd)| SearchResult {
                    command: cmd.clone(),
                    score: 1.0,
                    position: i,
                })
                .collect();

            self.cache.insert(query.to_string(), results.clone());
            return results;
        }

        // Calculate scores for each history item
        let mut results: Vec<SearchResult> = history
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                let score = self.calculate_score(query, cmd);
                if score > 0.0 {
                    Some(SearchResult {
                        command: cmd.clone(),
                        score,
                        position: i,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal)
        });

        // Cache the results
        self.cache.insert(query.to_string(), results.clone());

        results
    }

    /// Calculate the relevance score for a query and a command
    fn calculate_score(&self, query: &str, command: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let command_lower = command.to_lowercase();

        // Exact match gets highest score
        if command_lower == query_lower {
            return 1.0;
        }

        // Starts with gets high score
        if command_lower.starts_with(&query_lower) {
            return 0.9;
        }

        // Contains gets medium score
        if command_lower.contains(&query_lower) {
            // Calculate how early in the string the match occurs
            let position = command_lower.find(&query_lower).unwrap_or(0) as f32;
            let position_factor = 1.0 - (position / command.len() as f32).min(0.9);
            return 0.5 + (0.4 * position_factor);
        }

        // Fuzzy matching
        let mut score = 0.0;

        // Check if all characters in query appear in order in command
        let mut last_found = 0;
        let mut all_found = true;

        for q_char in query_lower.chars() {
            let mut found = false;
            for (i, c_char) in command_lower[last_found..].char_indices() {
                if c_char == q_char {
                    last_found += i + 1;
                    found = true;
                    break;
                }
            }

            if !found {
                all_found = false;
                break;
            }
        }

        if all_found {
            // Calculate a score based on how spread out the matches are
            let spread = last_found as f32 / command.len() as f32;
            score = 0.1 + (0.3 * (1.0 - spread));
        }

        score
    }

    /// Clear the search cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl ConsoleWindow {
    /// Perform an enhanced search on command history
    pub fn enhanced_search(&mut self, query: &str) -> Vec<String> {
        let mut engine = SearchEngine::new();
        let history: Vec<String> = self.get_history().into_iter().collect();

        let results = engine.search(query, &history);
        results.into_iter().map(|r| r.command).collect()
    }
}

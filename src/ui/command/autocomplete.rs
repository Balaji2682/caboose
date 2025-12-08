/// Autocomplete engine with fuzzy matching for command suggestions
use super::registry::CommandMetadata;

/// Autocomplete suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub text: String,
    pub description: String,
    pub usage: String,
    pub score: usize,
}

impl Suggestion {
    pub fn new(text: String, description: String, usage: String, score: usize) -> Self {
        Self {
            text,
            description,
            usage,
            score,
        }
    }
}

/// Autocomplete engine for command suggestions
pub struct AutocompleteEngine {
    commands: Vec<CommandMetadata>,
}

impl AutocompleteEngine {
    /// Create a new autocomplete engine
    pub fn new(commands: Vec<CommandMetadata>) -> Self {
        Self { commands }
    }

    /// Get suggestions for a partial command
    ///
    /// Uses multiple matching strategies:
    /// 1. Exact prefix match (highest score)
    /// 2. Word boundary match
    /// 3. Fuzzy match (substring)
    ///
    /// Returns suggestions sorted by relevance (score descending)
    pub fn get_suggestions(&self, partial: &str, max_results: usize) -> Vec<Suggestion> {
        if partial.is_empty() {
            // Return all commands if input is empty
            return self
                .commands
                .iter()
                .take(max_results)
                .map(|cmd| {
                    Suggestion::new(
                        cmd.name.clone(),
                        cmd.description.clone(),
                        cmd.usage.clone(),
                        0,
                    )
                })
                .collect();
        }

        let partial_lower = partial.to_lowercase();
        let mut suggestions = Vec::new();

        // Score each command
        for cmd in &self.commands {
            if let Some(score) = self.calculate_score(&cmd.name, &partial_lower) {
                suggestions.push(Suggestion::new(
                    cmd.name.clone(),
                    cmd.description.clone(),
                    cmd.usage.clone(),
                    score,
                ));
            }

            // Also check aliases
            for alias in &cmd.aliases {
                if let Some(score) = self.calculate_score(alias, &partial_lower) {
                    suggestions.push(Suggestion::new(
                        alias.clone(),
                        format!("{} (alias for {})", cmd.description, cmd.name),
                        cmd.usage.clone(),
                        score,
                    ));
                }
            }
        }

        // Sort by score (descending) and take top results
        suggestions.sort_by(|a, b| b.score.cmp(&a.score));
        suggestions.truncate(max_results);
        suggestions
    }

    /// Calculate match score for a command name
    ///
    /// Returns None if no match, otherwise returns score (higher = better)
    fn calculate_score(&self, name: &str, partial: &str) -> Option<usize> {
        let name_lower = name.to_lowercase();

        // Exact match - highest score
        if name_lower == partial {
            return Some(1000);
        }

        // Exact prefix match - very high score
        if name_lower.starts_with(partial) {
            return Some(900 - partial.len());
        }

        // Fuzzy match - word boundary
        if self.matches_word_boundary(&name_lower, partial) {
            return Some(800);
        }

        // Fuzzy match - subsequence (each char appears in order)
        if self.matches_subsequence(&name_lower, partial) {
            return Some(700);
        }

        // Substring match - lower score
        if name_lower.contains(partial) {
            return Some(600);
        }

        None
    }

    /// Check if partial matches at word boundaries
    /// Example: "qua" matches "query_analysis" at 'q' and "ua" in "query"
    fn matches_word_boundary(&self, name: &str, partial: &str) -> bool {
        // Split on common word separators
        let words: Vec<&str> = name.split(&['_', '-', ' '][..]).collect();

        for word in words {
            if word.starts_with(partial) {
                return true;
            }
        }

        false
    }

    /// Check if partial is a subsequence of name
    /// Example: "sch" matches "search" (s, c, h appear in order)
    fn matches_subsequence(&self, name: &str, partial: &str) -> bool {
        let mut name_chars = name.chars();
        let mut partial_chars = partial.chars().peekable();

        while let Some(p_ch) = partial_chars.peek() {
            match name_chars.find(|&n_ch| n_ch == *p_ch) {
                Some(_) => {
                    partial_chars.next();
                }
                None => return false,
            }
        }

        true
    }

    /// Get argument suggestions for a command (if available)
    pub fn get_arg_suggestions(&self, command_name: &str) -> Vec<String> {
        self.commands
            .iter()
            .find(|cmd| cmd.name == command_name)
            .map(|cmd| cmd.arg_hints.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::command::registry::CommandMetadata;

    fn create_test_engine() -> AutocompleteEngine {
        let commands = vec![
            CommandMetadata {
                name: "search".to_string(),
                aliases: vec!["s".to_string(), "find".to_string()],
                description: "Search logs".to_string(),
                usage: "/search <query>".to_string(),
                arg_hints: vec!["error".to_string(), "warn".to_string()],
            },
            CommandMetadata {
                name: "quit".to_string(),
                aliases: vec!["q".to_string(), "exit".to_string()],
                description: "Quit application".to_string(),
                usage: "/quit".to_string(),
                arg_hints: vec![],
            },
            CommandMetadata {
                name: "query_analysis".to_string(),
                aliases: vec![],
                description: "Show query analysis".to_string(),
                usage: "/query_analysis".to_string(),
                arg_hints: vec![],
            },
        ];

        AutocompleteEngine::new(commands)
    }

    #[test]
    fn test_exact_prefix_match() {
        let engine = create_test_engine();
        let suggestions = engine.get_suggestions("sea", 5);

        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].text, "search");
        assert!(suggestions[0].score > 800);
    }

    #[test]
    fn test_alias_match() {
        let engine = create_test_engine();
        let suggestions = engine.get_suggestions("s", 5);

        // Should match both "search" and "s" alias
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_empty_input_returns_all() {
        let engine = create_test_engine();
        let suggestions = engine.get_suggestions("", 10);

        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn test_fuzzy_subsequence_match() {
        let engine = create_test_engine();
        let suggestions = engine.get_suggestions("qua", 5);

        // Should match "query_analysis" via word boundary
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.text == "query_analysis"));
    }

    #[test]
    fn test_no_match() {
        let engine = create_test_engine();
        let suggestions = engine.get_suggestions("xyz", 5);

        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_max_results() {
        let engine = create_test_engine();
        let suggestions = engine.get_suggestions("q", 1);

        assert_eq!(suggestions.len(), 1);
    }

    #[test]
    fn test_arg_suggestions() {
        let engine = create_test_engine();
        let hints = engine.get_arg_suggestions("search");

        assert_eq!(hints, vec!["error", "warn"]);
    }
}

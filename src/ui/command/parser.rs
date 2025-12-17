/// Command parser for parsing user input into commands and arguments
use std::str::FromStr;

/// Parsed command with name and arguments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub name: String,
    pub args: Vec<String>,
    pub raw: String,
}

/// Command parser
pub struct CommandParser;

impl CommandParser {
    /// Parse a command string into name and arguments
    ///
    /// # Examples
    ///
    /// ```
    /// use caboose::ui::command::CommandParser;
    ///
    /// let cmd = CommandParser::parse("/search error logs");
    /// assert_eq!(cmd.name, "search");
    /// assert_eq!(cmd.args, vec!["error", "logs"]);
    ///
    /// let cmd = CommandParser::parse("/export \"my file.txt\"");
    /// assert_eq!(cmd.name, "export");
    /// assert_eq!(cmd.args, vec!["my file.txt"]);
    /// ```
    pub fn parse(input: &str) -> ParsedCommand {
        let trimmed = input.trim();
        let raw = trimmed.to_string();

        // Remove leading slash if present
        let without_slash = trimmed.strip_prefix('/').unwrap_or(trimmed);

        if without_slash.is_empty() {
            return ParsedCommand {
                name: String::new(),
                args: vec![],
                raw,
            };
        }

        // Parse command with quoted string support
        let tokens = Self::tokenize(without_slash);

        if tokens.is_empty() {
            return ParsedCommand {
                name: String::new(),
                args: vec![],
                raw,
            };
        }

        ParsedCommand {
            name: tokens[0].clone(),
            args: tokens[1..].to_vec(),
            raw,
        }
    }

    /// Tokenize input, respecting quoted strings
    ///
    /// # Examples
    ///
    /// ```text
    /// "search error logs" -> ["search", "error", "logs"]
    /// "export \"my file.txt\"" -> ["export", "my file.txt"]
    /// "filter 'process name'" -> ["filter", "process name"]
    /// ```
    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                '"' | '\'' if in_quotes && ch == quote_char => {
                    in_quotes = false;
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                ' ' if !in_quotes => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        // Push remaining token
        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// Check if input is a valid command (starts with /)
    pub fn is_command(input: &str) -> bool {
        input.trim().starts_with('/')
    }

    /// Extract the command name from partial input (for autocomplete)
    pub fn extract_partial_command(input: &str) -> String {
        let trimmed = input.trim();
        let without_slash = trimmed.strip_prefix('/').unwrap_or(trimmed);

        // Get first word only
        without_slash
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }
}

impl FromStr for ParsedCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CommandParser::parse(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let cmd = CommandParser::parse("/quit");
        assert_eq!(cmd.name, "quit");
        assert_eq!(cmd.args, Vec::<String>::new());
    }

    #[test]
    fn test_parse_command_with_args() {
        let cmd = CommandParser::parse("/search error logs");
        assert_eq!(cmd.name, "search");
        assert_eq!(cmd.args, vec!["error", "logs"]);
    }

    #[test]
    fn test_parse_command_with_quoted_args() {
        let cmd = CommandParser::parse("/export \"my file.txt\"");
        assert_eq!(cmd.name, "export");
        assert_eq!(cmd.args, vec!["my file.txt"]);
    }

    #[test]
    fn test_parse_command_with_single_quotes() {
        let cmd = CommandParser::parse("/filter 'process name'");
        assert_eq!(cmd.name, "filter");
        assert_eq!(cmd.args, vec!["process name"]);
    }

    #[test]
    fn test_parse_empty_command() {
        let cmd = CommandParser::parse("/");
        assert_eq!(cmd.name, "");
        assert_eq!(cmd.args, Vec::<String>::new());
    }

    #[test]
    fn test_parse_without_slash() {
        let cmd = CommandParser::parse("quit");
        assert_eq!(cmd.name, "quit");
    }

    #[test]
    fn test_is_command() {
        assert!(CommandParser::is_command("/quit"));
        assert!(CommandParser::is_command("  /quit  "));
        assert!(!CommandParser::is_command("quit"));
        assert!(!CommandParser::is_command(""));
    }

    #[test]
    fn test_extract_partial_command() {
        assert_eq!(CommandParser::extract_partial_command("/sea"), "sea");
        assert_eq!(
            CommandParser::extract_partial_command("/search logs"),
            "search"
        );
        assert_eq!(CommandParser::extract_partial_command("/"), "");
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(
            CommandParser::tokenize("search error logs"),
            vec!["search", "error", "logs"]
        );
        assert_eq!(
            CommandParser::tokenize("export \"my file.txt\""),
            vec!["export", "my file.txt"]
        );
        assert_eq!(
            CommandParser::tokenize("filter 'process name' more"),
            vec!["filter", "process name", "more"]
        );
    }
}

/// Command history manager for navigating previous commands
use std::collections::VecDeque;

/// Command history with navigation support
pub struct CommandHistory {
    /// Stored commands (most recent last)
    history: VecDeque<String>,
    /// Maximum history size
    max_size: usize,
    /// Current navigation position (None = not navigating)
    position: Option<usize>,
    /// Temporary buffer for current input when starting navigation
    temp_buffer: String,
}

impl CommandHistory {
    /// Create a new command history with maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
            position: None,
            temp_buffer: String::new(),
        }
    }

    /// Add a command to history
    ///
    /// Ignores empty commands and duplicates of the most recent command
    pub fn add(&mut self, command: String) {
        // Ignore empty commands
        if command.trim().is_empty() {
            return;
        }

        // Ignore if same as last command
        if self.history.back() == Some(&command) {
            return;
        }

        // Add to history
        self.history.push_back(command);

        // Trim if exceeds max size
        if self.history.len() > self.max_size {
            self.history.pop_front();
        }

        // Reset navigation position
        self.reset_navigation();
    }

    /// Navigate to previous command (older)
    ///
    /// Returns the command at the new position, or None if at the beginning
    pub fn prev(&mut self, current_input: &str) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }

        // First time navigating - save current input
        if self.position.is_none() {
            self.temp_buffer = current_input.to_string();
            self.position = Some(self.history.len() - 1);
        } else if let Some(pos) = self.position {
            // Move to older command
            if pos > 0 {
                self.position = Some(pos - 1);
            } else {
                // Already at oldest
                return None;
            }
        }

        self.position.and_then(|pos| self.history.get(pos).cloned())
    }

    /// Navigate to next command (newer)
    ///
    /// Returns the command at the new position, or the temporary buffer if at the end
    pub fn next(&mut self) -> Option<String> {
        if self.position.is_none() {
            return None;
        }

        let new_pos = self.position.unwrap() + 1;

        if new_pos >= self.history.len() {
            // Reached the end - return temp buffer
            self.position = None;
            Some(self.temp_buffer.clone())
        } else {
            self.position = Some(new_pos);
            self.history.get(new_pos).cloned()
        }
    }

    /// Reset navigation position
    pub fn reset_navigation(&mut self) {
        self.position = None;
        self.temp_buffer.clear();
    }

    /// Get all history entries (most recent first)
    pub fn entries(&self) -> Vec<String> {
        self.history.iter().rev().cloned().collect()
    }

    /// Check if currently navigating history
    pub fn is_navigating(&self) -> bool {
        self.position.is_some()
    }

    /// Get current position in history
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    /// Get history size
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
        self.reset_navigation();
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new(5);
        history.add("/search error".to_string());
        history.add("/quit".to_string());

        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_ignore_empty() {
        let mut history = CommandHistory::new(5);
        history.add("".to_string());
        history.add("   ".to_string());

        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_ignore_duplicates() {
        let mut history = CommandHistory::new(5);
        history.add("/quit".to_string());
        history.add("/quit".to_string());

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_max_size() {
        let mut history = CommandHistory::new(3);
        history.add("/cmd1".to_string());
        history.add("/cmd2".to_string());
        history.add("/cmd3".to_string());
        history.add("/cmd4".to_string());

        assert_eq!(history.len(), 3);
        assert_eq!(history.entries(), vec!["/cmd4", "/cmd3", "/cmd2"]);
    }

    #[test]
    fn test_navigate_prev() {
        let mut history = CommandHistory::new(5);
        history.add("/cmd1".to_string());
        history.add("/cmd2".to_string());
        history.add("/cmd3".to_string());

        let prev = history.prev("");
        assert_eq!(prev, Some("/cmd3".to_string()));

        let prev = history.prev("");
        assert_eq!(prev, Some("/cmd2".to_string()));
    }

    #[test]
    fn test_navigate_next() {
        let mut history = CommandHistory::new(5);
        history.add("/cmd1".to_string());
        history.add("/cmd2".to_string());

        // Go back
        history.prev("");
        history.prev("");

        // Go forward
        let next = history.next();
        assert_eq!(next, Some("/cmd2".to_string()));
    }

    #[test]
    fn test_temp_buffer() {
        let mut history = CommandHistory::new(5);
        history.add("/cmd1".to_string());

        // Start typing new command
        let current = "/sea";
        history.prev(current);

        // Navigate back to temp buffer
        let next = history.next();
        assert_eq!(next, Some("/sea".to_string()));
    }

    #[test]
    fn test_reset_navigation() {
        let mut history = CommandHistory::new(5);
        history.add("/cmd1".to_string());

        history.prev("");
        assert!(history.is_navigating());

        history.reset_navigation();
        assert!(!history.is_navigating());
    }

    #[test]
    fn test_clear() {
        let mut history = CommandHistory::new(5);
        history.add("/cmd1".to_string());
        history.add("/cmd2".to_string());

        history.clear();
        assert_eq!(history.len(), 0);
    }
}

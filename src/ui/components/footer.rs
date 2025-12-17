/// Footer component builder for consistent keyboard shortcut display
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::ui::theme::{Icons, Theme};

/// Represents a keyboard shortcut
pub struct KeyBinding {
    pub key: String,
    pub description: String,
}

impl KeyBinding {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}

/// Builder for creating consistent footers with keyboard shortcuts
pub struct FooterBuilder {
    bindings: Vec<KeyBinding>,
}

impl FooterBuilder {
    /// Create a new footer builder
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Add a key binding
    pub fn add_binding(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.bindings.push(KeyBinding::new(key, description));
        self
    }

    /// Add standard quit binding
    pub fn with_quit(self) -> Self {
        self.add_binding("q", "Quit")
    }

    /// Add standard toggle binding
    pub fn with_toggle(self) -> Self {
        self.add_binding("t", "Cycle Views")
    }

    /// Add standard escape binding
    pub fn with_escape(self) -> Self {
        self.add_binding("Esc", "Back")
    }

    /// Add standard navigation bindings
    pub fn with_navigation(self) -> Self {
        self.add_binding("↑↓", "Navigate")
    }

    /// Add standard search binding
    pub fn with_search(self) -> Self {
        self.add_binding("/", "Search")
    }

    /// Build the footer line
    pub fn build(self) -> Line<'static> {
        let mut spans = vec![Span::raw("  ")];

        for (idx, binding) in self.bindings.iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw("   "));
            }

            spans.push(Span::styled(
                binding.key.clone(),
                Style::default()
                    .fg(Theme::text_primary())
                    .add_modifier(Modifier::BOLD),
            ));

            spans.push(Span::styled(
                format!(" {} {}", Icons::SEPARATOR, binding.description),
                Style::default().fg(Theme::text_secondary()),
            ));
        }

        Line::from(spans)
    }
}

impl Default for FooterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a standard footer for main views
pub fn standard_footer() -> Line<'static> {
    FooterBuilder::new()
        .with_quit()
        .with_toggle()
        .with_escape()
        .build()
}

/// Create a footer for list/navigation views
pub fn navigation_footer() -> Line<'static> {
    FooterBuilder::new()
        .with_quit()
        .with_toggle()
        .with_navigation()
        .add_binding("Enter", "View Details")
        .build()
}

/// Create a footer for search mode
pub fn search_footer() -> Line<'static> {
    FooterBuilder::new()
        .add_binding("Type to search", "")
        .add_binding("Esc", "Cancel")
        .add_binding("Enter", "Apply")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_builder() {
        let footer = FooterBuilder::new().add_binding("q", "Quit").build();

        assert!(!footer.spans.is_empty());
    }

    #[test]
    fn test_standard_footer() {
        let footer = standard_footer();
        assert!(!footer.spans.is_empty());
    }

    #[test]
    fn test_navigation_footer() {
        let footer = navigation_footer();
        assert!(!footer.spans.is_empty());
    }
}

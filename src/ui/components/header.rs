/// Header component builder for consistent header rendering
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::ui::theme::Theme;

/// Builder for creating consistent headers across views
pub struct HeaderBuilder<'a> {
    title: &'a str,
    icon: Option<&'a str>,
    lines: Vec<Line<'a>>,
}

impl<'a> HeaderBuilder<'a> {
    /// Create a new header builder with a title
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            icon: None,
            lines: Vec::new(),
        }
    }

    /// Set an icon for the header
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Add a line to the header
    pub fn add_line(mut self, line: Line<'a>) -> Self {
        self.lines.push(line);
        self
    }

    /// Add a stat line with label and value
    pub fn add_stat(
        mut self,
        label: &'a str,
        value: String,
        value_color: ratatui::style::Color,
    ) -> Self {
        let line = Line::from(vec![
            Span::raw("   "),
            Span::styled(label, Style::default().fg(Theme::text_secondary())),
            Span::styled(
                value,
                Style::default()
                    .fg(value_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        self.lines.push(line);
        self
    }

    /// Build the header lines
    pub fn build(self) -> Vec<Line<'a>> {
        let mut result = Vec::new();

        // Title line
        let mut title_spans = vec![];
        if let Some(icon) = self.icon {
            title_spans.push(Span::styled(
                format!("   {} ", icon),
                Style::default()
                    .fg(Theme::primary())
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            title_spans.push(Span::raw("   "));
        }

        title_spans.push(Span::styled(
            self.title,
            Style::default()
                .fg(Theme::primary())
                .add_modifier(Modifier::BOLD),
        ));

        result.push(Line::from(title_spans));

        // Empty line for spacing
        if !self.lines.is_empty() {
            result.push(Line::from(""));
        }

        // Add custom lines
        result.extend(self.lines);

        result
    }
}

/// Helper to create a simple metric line
pub fn metric_line<'a>(
    icon: &'a str,
    value: String,
    label: &'a str,
    value_color: ratatui::style::Color,
) -> Line<'a> {
    Line::from(vec![
        Span::raw("   "),
        Span::styled(format!("{} ", icon), Style::default().fg(value_color)),
        Span::styled(
            value,
            Style::default()
                .fg(Theme::text_primary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", label),
            Style::default().fg(Theme::text_secondary()),
        ),
    ])
}

/// Helper to create a separator line
pub fn separator_line() -> Line<'static> {
    Line::from("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Icons;

    #[test]
    fn test_header_builder() {
        let header = HeaderBuilder::new("Test Dashboard")
            .icon(Icons::database())
            .build();

        assert!(!header.is_empty());
        assert!(header.len() >= 1);
    }

    #[test]
    fn test_header_with_stats() {
        let header = HeaderBuilder::new("Test")
            .add_stat("Count: ", "100".to_string(), Theme::success())
            .build();

        assert!(header.len() >= 2); // Title + empty line + stat
    }
}

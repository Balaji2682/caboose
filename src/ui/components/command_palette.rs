/// Command palette UI component - Claude CLI inspired command interface
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::theme::{Icons, Theme};
use crate::ui::command::autocomplete::Suggestion;

/// Render the command palette at the bottom of the screen
///
/// # Layout
///
/// ```text
/// ┌─────────────────────────────────┐
/// │ > /search error logs            │ ← Input
/// ├─────────────────────────────────┤
/// │ search - Search logs for a query│ ← Suggestions
/// │ filter - Filter logs by process │
/// │ clear  - Clear search and filters│
/// └─────────────────────────────────┘
/// ```
pub fn render_command_palette(
    f: &mut Frame,
    area: Rect,
    input: &str,
    suggestions: &[Suggestion],
    selected_suggestion: usize,
    error: Option<&str>,
) {
    // Split area into input + suggestions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                           // Input
            Constraint::Length(suggestions.len().min(5) as u16 + 2), // Suggestions (max 5)
        ])
        .split(area);

    // Render input field
    render_input(f, chunks[0], input, error);

    // Render suggestions if available
    if !suggestions.is_empty() {
        render_suggestions(f, chunks[1], suggestions, selected_suggestion);
    }
}

/// Render the command input field
fn render_input(f: &mut Frame, area: Rect, input: &str, error: Option<&str>) {
    let (style, border_color) = if error.is_some() {
        (
            Style::default().fg(Theme::DANGER),
            Theme::DANGER,
        )
    } else {
        (
            Style::default().fg(Theme::TEXT_PRIMARY),
            Theme::PRIMARY,
        )
    };

    let text = if let Some(err_msg) = error {
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(input, style),
            Span::raw("  "),
            Span::styled(
                format!(" {} {}", Icons::error(), err_msg),
                Style::default().fg(Theme::DANGER),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(input, style),
            Span::styled("█", Style::default().fg(Theme::PRIMARY)), // Cursor
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(Theme::border_type())
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " Command ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

/// Render autocomplete suggestions
fn render_suggestions(
    f: &mut Frame,
    area: Rect,
    suggestions: &[Suggestion],
    selected: usize,
) {
    let items: Vec<ListItem> = suggestions
        .iter()
        .enumerate()
        .take(5) // Max 5 suggestions
        .map(|(idx, suggestion)| {
            let is_selected = idx == selected;

            let line = if is_selected {
                Line::from(vec![
                    Span::styled(
                        format!(" {} ", Icons::right_triangle()),
                        Style::default().fg(Theme::PRIMARY),
                    ),
                    Span::styled(
                        format!("{:<12}", suggestion.text),
                        Style::default()
                            .fg(Theme::PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" - {}", suggestion.description),
                        Style::default().fg(Theme::TEXT_SECONDARY),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("{:<12}", suggestion.text),
                        Style::default().fg(Theme::TEXT_PRIMARY),
                    ),
                    Span::styled(
                        format!(" - {}", suggestion.description),
                        Style::default().fg(Theme::TEXT_MUTED),
                    ),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(Theme::border_type())
            .border_style(Style::default().fg(Theme::TEXT_MUTED))
            .title(Span::styled(
                format!(" Suggestions ({}) ", suggestions.len()),
                Style::default().fg(Theme::TEXT_SECONDARY),
            )),
    );

    f.render_widget(list, area);
}

/// Render command result message (success or error)
pub fn render_command_result(f: &mut Frame, area: Rect, message: &str, is_error: bool) {
    let (icon, color) = if is_error {
        (Icons::error(), Theme::DANGER)
    } else {
        (Icons::success(), Theme::SUCCESS)
    };

    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", icon),
            Style::default().fg(color),
        ),
        Span::styled(
            message,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(Theme::border_type())
                .border_style(Style::default().fg(color)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Calculate the area for the command palette overlay
///
/// Returns a centered area at the bottom of the screen
pub fn calculate_palette_area(full_area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(10), // Height for palette
        ])
        .split(full_area);

    vertical[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_palette_area() {
        let area = Rect::new(0, 0, 100, 50);
        let palette_area = calculate_palette_area(area);

        assert_eq!(palette_area.height, 10);
        assert_eq!(palette_area.width, 100);
    }
}

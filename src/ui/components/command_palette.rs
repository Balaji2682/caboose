/// Command palette UI component - Claude CLI inspired command interface
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::ui::command::autocomplete::Suggestion;
use crate::ui::theme::{Icons, Theme};

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
    fade_progress: Option<f32>,
) {
    // Split area into input + suggestions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                                   // Input
            Constraint::Length(suggestions.len().min(5) as u16 + 2), // Suggestions (max 5)
        ])
        .split(area);

    // Render input field
    render_input(f, chunks[0], input, error, fade_progress);

    // Render suggestions if available
    if !suggestions.is_empty() {
        render_suggestions(
            f,
            chunks[1],
            suggestions,
            selected_suggestion,
            fade_progress,
        );
    }
}

/// Render the command input field
fn render_input(
    f: &mut Frame,
    area: Rect,
    input: &str,
    error: Option<&str>,
    fade_progress: Option<f32>,
) {
    let (style, border_color) = if error.is_some() {
        (
            Style::default().fg(Theme::apply_fade_to_color(
                Theme::danger(),
                fade_progress.unwrap_or(1.0),
            )),
            Theme::apply_fade_to_color(Theme::danger(), fade_progress.unwrap_or(1.0)),
        )
    } else {
        (
            Style::default().fg(Theme::apply_fade_to_color(
                Theme::text_primary(),
                fade_progress.unwrap_or(1.0),
            )),
            Theme::apply_fade_to_color(Theme::primary(), fade_progress.unwrap_or(1.0)),
        )
    };

    let text = if let Some(err_msg) = error {
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default()
                    .fg(Theme::apply_fade_to_color(
                        Theme::primary(),
                        fade_progress.unwrap_or(1.0),
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(input, style),
            Span::raw("  "),
            Span::styled(
                format!(" {} {}", Icons::error(), err_msg),
                Style::default().fg(Theme::apply_fade_to_color(
                    Theme::danger(),
                    fade_progress.unwrap_or(1.0),
                )),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default()
                    .fg(Theme::apply_fade_to_color(
                        Theme::primary(),
                        fade_progress.unwrap_or(1.0),
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(input, style),
            Span::styled(
                "█",
                Style::default().fg(Theme::apply_fade_to_color(
                    Theme::primary(),
                    fade_progress.unwrap_or(1.0),
                )),
            ), // Cursor
        ])
    };

    let block =
        Theme::block(" Command ", fade_progress).border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

/// Render autocomplete suggestions
fn render_suggestions(
    f: &mut Frame,
    area: Rect,
    suggestions: &[Suggestion],
    selected: usize,
    fade_progress: Option<f32>,
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
                        Style::default().fg(Theme::apply_fade_to_color(
                            Theme::primary(),
                            fade_progress.unwrap_or(1.0),
                        )),
                    ),
                    Span::styled(
                        format!("{:<12}", suggestion.text),
                        Style::default()
                            .fg(Theme::apply_fade_to_color(
                                Theme::text_primary(),
                                fade_progress.unwrap_or(1.0),
                            ))
                            .bg(Theme::apply_fade_to_color(
                                Theme::surface(),
                                fade_progress.unwrap_or(1.0),
                            )) // Subtle background for selected
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" - {}", suggestion.description),
                        Style::default()
                            .fg(Theme::apply_fade_to_color(
                                Theme::text_secondary(),
                                fade_progress.unwrap_or(1.0),
                            ))
                            .bg(Theme::apply_fade_to_color(
                                Theme::surface(),
                                fade_progress.unwrap_or(1.0),
                            )), // Also apply background to description
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("{:<12}", suggestion.text),
                        Style::default().fg(Theme::apply_fade_to_color(
                            Theme::text_primary(),
                            fade_progress.unwrap_or(1.0),
                        )),
                    ),
                    Span::styled(
                        format!(" - {}", suggestion.description),
                        Style::default().fg(Theme::apply_fade_to_color(
                            Theme::text_secondary(),
                            fade_progress.unwrap_or(1.0),
                        )),
                    ),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Theme::block_plain(fade_progress)
            .border_style(Style::default().fg(Theme::apply_fade_to_color(
                Theme::text_muted(),
                fade_progress.unwrap_or(1.0),
            )))
            .title(Span::styled(
                format!(" Suggestions ({}) ", suggestions.len()),
                Style::default().fg(Theme::apply_fade_to_color(
                    Theme::text_secondary(),
                    fade_progress.unwrap_or(1.0),
                )),
            )),
    );

    f.render_widget(list, area);
}

/// Render command result message (success or error)
pub fn render_command_result(
    f: &mut Frame,
    area: Rect,
    message: &str,
    is_error: bool,
    fade_progress: Option<f32>,
) {
    let (icon, color) = if is_error {
        (Icons::error(), Theme::danger())
    } else {
        (Icons::success(), Theme::success())
    };

    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", icon),
            Style::default().fg(Theme::apply_fade_to_color(
                color,
                fade_progress.unwrap_or(1.0),
            )),
        ),
        Span::styled(
            message,
            Style::default()
                .fg(Theme::apply_fade_to_color(
                    color,
                    fade_progress.unwrap_or(1.0),
                ))
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let paragraph = Paragraph::new(text)
        .block(
            Theme::block_plain(fade_progress).border_style(Style::default().fg(
                Theme::apply_fade_to_color(color, fade_progress.unwrap_or(1.0)),
            )),
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

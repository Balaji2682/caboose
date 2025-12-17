use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
};

use crate::exception::{ExceptionGroup, ExceptionSeverity, ExceptionTracker};
use crate::ui::formatting::format_relative_time;
use crate::ui::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    exception_tracker: &ExceptionTracker,
    exception_index: usize,
    fade_progress: Option<f32>,
) {
    let groups = exception_tracker.get_grouped_exceptions();

    if exception_index >= groups.len() {
        let paragraph = Paragraph::new("No exception selected")
            .block(Theme::block("Exception Detail", fade_progress))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let group = &groups[exception_index];
    let exception = &group.sample_exception;
    let severity = ExceptionSeverity::from_exception_type(&group.exception_type);

    // Split area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Header info
            Constraint::Min(10),   // Backtrace
        ])
        .split(area);

    // Header section with exception details
    render_header(f, chunks[0], group, severity, fade_progress);

    // Backtrace section
    render_backtrace(f, chunks[1], exception, fade_progress);
}

fn render_header(
    f: &mut Frame,
    area: Rect,
    group: &ExceptionGroup,
    severity: ExceptionSeverity,
    fade_progress: Option<f32>,
) {
    let severity_color = match severity {
        ExceptionSeverity::Critical => Color::Red,
        ExceptionSeverity::High => Color::LightRed,
        ExceptionSeverity::Medium => Color::Yellow,
        ExceptionSeverity::Low => Color::Blue,
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("Exception: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                &group.exception_type,
                Style::default()
                    .fg(Theme::apply_fade_to_color(
                        severity_color,
                        fade_progress.unwrap_or(1.0),
                    ))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("[{}]", severity.icon()),
                Style::default().fg(Theme::apply_fade_to_color(
                    severity_color,
                    fade_progress.unwrap_or(1.0),
                )),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Message: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&group.sample_exception.message),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Occurrences: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{} times", group.count)),
            Span::raw("  │  "),
            Span::styled("First: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_relative_time(group.first_seen.elapsed())),
            Span::raw("  │  "),
            Span::styled("Last: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_relative_time(group.last_seen.elapsed())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Location: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{}:{}",
                group
                    .sample_exception
                    .file_path
                    .as_ref()
                    .unwrap_or(&"unknown".to_string()),
                group.sample_exception.line_number.unwrap_or(0)
            )),
        ]),
    ];

    let paragraph = Paragraph::new(header_text)
        .block(
            Theme::block(" Exception Details ", fade_progress).border_style(Style::default().fg(
                Theme::apply_fade_to_color(Theme::text_secondary(), fade_progress.unwrap_or(1.0)),
            )),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_backtrace(
    f: &mut Frame,
    area: Rect,
    exception: &crate::exception::Exception,
    fade_progress: Option<f32>,
) {
    let backtrace_items: Vec<ListItem> = if exception.backtrace.is_empty() {
        vec![ListItem::new("No backtrace available")]
    } else {
        exception
            .backtrace
            .iter()
            .take(20) // Show first 20 lines
            .map(|line| {
                let style = if line.contains("app/") {
                    // Highlight application code
                    Style::default().fg(Color::Cyan)
                } else if line.contains("vendor/") || line.contains("gems/") {
                    // Dim vendor/gem code
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(line.as_str()).style(style)
            })
            .collect()
    };

    let backtrace_count = exception.backtrace.len();
    let title = if backtrace_count > 20 {
        format!(
            " Backtrace (showing 20 of {}) - Press Esc to go back ",
            backtrace_count
        )
    } else {
        " Backtrace - Press Esc to go back ".to_string()
    };

    let list = List::new(backtrace_items).block(Theme::block(title, fade_progress).border_style(
        Style::default().fg(Theme::apply_fade_to_color(
            Theme::text_secondary(),
            fade_progress.unwrap_or(1.0),
        )),
    ));

    f.render_widget(list, area);
}

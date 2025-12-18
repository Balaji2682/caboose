use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
};

use crate::process::{LogLine, ProcessInfo, ProcessStatus};
use crate::ui::components::ScrollIndicator;
use crate::ui::formatting::format_duration;
use crate::ui::theme::{Icons, Theme};

/// Render the logs view
pub fn render(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    processes: &[ProcessInfo],
    logs: &[LogLine],
    _search_mode: bool,
    search_query: &str,
    log_scroll: usize,
    horizontal_scroll: usize,
    auto_scroll: bool,
    filter_process: &Option<String>,
    spinner_frame: usize,
    fade_progress: Option<f32>,
) {
    // Clear full area to avoid artifacts bleeding between panels/spinner frames
    f.render_widget(Clear, area);

    // Split horizontally: processes panel (left) and logs panel (right)
    // Process panel is 30 chars wide (28 usable after borders)
    // Content must fit: Icon(1) + Space(1) + Name(10) + Space(1) + Uptime(7) = ~20 chars
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    render_processes(f, chunks[0], processes);
    render_logs(
        f,
        chunks[1],
        logs,
        log_scroll,
        horizontal_scroll,
        auto_scroll,
        search_query,
        filter_process,
        spinner_frame,
        fade_progress,
    );
}

fn render_processes(f: &mut Frame, area: ratatui::layout::Rect, processes: &[ProcessInfo]) {
    let process_items: Vec<ListItem> = processes
        .iter()
        .map(|p| {
            let (status_icon, status_color) = match p.status {
                ProcessStatus::Running => (Icons::running(), Theme::success()),
                ProcessStatus::Stopped => (Icons::stopped(), Theme::text_muted()),
                ProcessStatus::Crashed => (Icons::error(), Theme::danger()),
            };

            let uptime = p.start_time.map_or("--".to_string(), |start| {
                let elapsed = start.elapsed().as_secs();
                format_duration(elapsed)
            });

            // Truncate process name if needed to fit in panel (max 10 chars)
            let display_name = if p.name.len() > 10 {
                format!("{}…", &p.name[..9])
            } else {
                p.name.clone()
            };

            // Compact layout to fit 30-char panel width:
            // Icon(1) + Space(1) + Name(10) + Space(1) + Uptime(7) = ~20 chars
            let content = Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(
                    status_icon,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:<10}", display_name),
                    Style::default()
                        .fg(Theme::primary())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:>7}", uptime),
                    Style::default().fg(Theme::text_secondary()),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let processes_widget = List::new(process_items).block(
        Theme::block("  Processes  ", None) // No fade on process list for now
            .border_style(Style::default().fg(Theme::text_muted())),
    );

    // Clear in case a spinner or other overlay was previously occupying this area
    f.render_widget(Clear, area);
    f.render_widget(processes_widget, area);
}

fn render_logs(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    logs: &[LogLine],
    log_scroll: usize,
    horizontal_scroll: usize,
    auto_scroll: bool,
    search_query: &str,
    filter_process: &Option<String>,
    _spinner_frame: usize,
    fade_progress: Option<f32>,
) {
    // If there are no logs at all, show a loading spinner
    if logs.is_empty() {
        f.render_widget(Clear, area);
        let block = Theme::block("Logs", fade_progress).border_style(Style::default().fg(
            Theme::apply_fade_to_color(Theme::text_muted(), fade_progress.unwrap_or(1.0)),
        ));
        let empty = Paragraph::new("Waiting for logs...").block(block);
        f.render_widget(empty, area);
        return;
    }

    // Filter logs
    let mut filtered: Vec<&LogLine> = if let Some(filter) = filter_process {
        logs.iter()
            .filter(|log| &log.process_name == filter)
            .collect()
    } else {
        logs.iter().collect()
    };

    // Apply search filter
    if !search_query.is_empty() {
        let query = search_query.to_lowercase();
        filtered.retain(|log| log.content.to_lowercase().contains(&query));
    }

    let total_logs = filtered.len();
    let visible_height = area.height.saturating_sub(2) as usize;
    let start_idx = if auto_scroll {
        total_logs.saturating_sub(visible_height.max(1))
    } else {
        log_scroll.min(total_logs.saturating_sub(visible_height))
    };

    let h_scroll = horizontal_scroll; // Capture for use in closure
    let log_lines: Vec<Line> = filtered
        .iter()
        .skip(start_idx)
        .take(visible_height.max(1))
        .map(|log| {
            // Apply horizontal scrolling to the content
            let scrolled_content = if h_scroll > 0 && log.content.len() > h_scroll {
                &log.content[h_scroll..]
            } else if h_scroll > 0 {
                "" // Scrolled past the content
            } else {
                &log.content
            };
            // Check for Rails-specific errors first for prominent highlighting
            let is_rails_error = log.content.to_lowercase().contains("pending migration")
                || (log.content.to_lowercase().contains("database")
                    && log.content.to_lowercase().contains("does not exist"))
                || log
                    .content
                    .to_lowercase()
                    .contains("could not connect to server")
                || log
                    .content
                    .to_lowercase()
                    .contains("address already in use")
                || (log.content.to_lowercase().contains("port")
                    && log.content.to_lowercase().contains("already in use"))
                || log.content.to_lowercase().contains("could not find gem")
                || log.content.to_lowercase().contains("secret_key_base");

            let content_style = if is_rails_error {
                // Bright red + bold for critical Rails errors
                Style::default()
                    .fg(Theme::danger())
                    .add_modifier(Modifier::BOLD)
            } else if log.content.contains("SELECT")
                || log.content.contains("INSERT")
                || log.content.contains("UPDATE")
                || log.content.contains("DELETE")
            {
                Style::default().fg(Theme::info())
            } else if log.content.contains("ERROR") || log.content.contains("Exception") {
                Style::default().fg(Theme::danger())
            } else if log.content.contains("Completed") {
                Style::default().fg(Theme::success())
            } else {
                Style::default()
            };

            // Add process icon based on name
            let process_icon = match log.process_name.as_str() {
                "web" | "rails" => "🌐",
                "angular" | "frontend" | "ui" => "⚡",
                "worker" | "sidekiq" => "⚙️",
                _ => "▪",
            };

            Line::from(vec![
                Span::styled(
                    format!("[{}] ", log.process_name),
                    Style::default().fg(process_name_color(&log.process_name)),
                ),
                Span::raw(process_icon),
                Span::raw(" "),
                Span::styled(scrolled_content, content_style),
            ])
        })
        .collect();

    let _scroll_indicator = ScrollIndicator::new(start_idx, total_logs, visible_height);

    let log_title = if let Some(filter) = filter_process {
        format!(" Logs (Filtered by {})", filter)
    } else if !search_query.is_empty() {
        format!(" Logs (Search: {})", search_query)
    } else {
        " Logs ".to_string()
    };

    let logs_widget = Paragraph::new(log_lines).block(
        Theme::block(log_title, fade_progress).border_style(Style::default().fg(
            Theme::apply_fade_to_color(Theme::text_muted(), fade_progress.unwrap_or(1.0)),
        )),
    );

    // Render the scroll indicator separately as a title or suffix if needed
    // For now, it's removed from the main title to reduce density.

    // Clear before rendering to prevent artifacts when content shrinks (e.g., spinner to list)
    f.render_widget(Clear, area);
    f.render_widget(logs_widget, area);
}

fn process_name_color(name: &str) -> ratatui::style::Color {
    use ratatui::style::Color;
    let colors = [
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
    ];
    let hash: usize = name.bytes().map(|b| b as usize).sum();
    colors[hash % colors.len()]
}

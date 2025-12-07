use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::process::{LogLine, ProcessInfo, ProcessStatus};
use crate::ui::components::ScrollIndicator;
use crate::ui::formatting::{format_duration, format_number};
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
    auto_scroll: bool,
    filter_process: &Option<String>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_processes(f, chunks[0], processes);
    render_logs(
        f,
        chunks[1],
        logs,
        log_scroll,
        auto_scroll,
        search_query,
        filter_process,
    );
}

fn render_processes(f: &mut Frame, area: ratatui::layout::Rect, processes: &[ProcessInfo]) {
    let process_items: Vec<ListItem> = processes
        .iter()
        .map(|p| {
            let (status_icon, status_color) = match p.status {
                ProcessStatus::Running => (Icons::RUNNING, Theme::SUCCESS),
                ProcessStatus::Stopped => (Icons::STOPPED, Theme::TEXT_MUTED),
                ProcessStatus::Crashed => (Icons::ERROR, Theme::DANGER),
            };

            let uptime = p.start_time.map_or("--".to_string(), |start| {
                let elapsed = start.elapsed().as_secs();
                format_duration(elapsed)
            });

            let status_text = format!("{:?}", p.status);

            let content = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    status_icon,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:14}", p.name),
                    Style::default()
                        .fg(Theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:9}", status_text),
                    Style::default().fg(status_color),
                ),
                Span::styled("  ", Style::default().fg(Theme::TEXT_SECONDARY)),
                Span::raw(" "),
                Span::styled(
                    format!("{:>10}", uptime),
                    Style::default().fg(Theme::WARNING),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let processes_widget = List::new(process_items).block(
        Block::default()
            .title(Span::styled(
                "  Processes  ",
                Style::default().fg(Theme::TEXT_PRIMARY),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::TEXT_MUTED)),
    );

    f.render_widget(processes_widget, area);
}

fn render_logs(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    logs: &[LogLine],
    log_scroll: usize,
    auto_scroll: bool,
    search_query: &str,
    filter_process: &Option<String>,
) {
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

    let log_lines: Vec<Line> = filtered
        .iter()
        .skip(start_idx)
        .take(visible_height.max(1))
        .map(|log| {
            let content_style = if log.content.contains("SELECT")
                || log.content.contains("INSERT")
                || log.content.contains("UPDATE")
                || log.content.contains("DELETE")
            {
                Style::default().fg(Theme::INFO)
            } else if log.content.contains("ERROR") || log.content.contains("Exception") {
                Style::default().fg(Theme::DANGER)
            } else if log.content.contains("Completed") {
                Style::default().fg(Theme::SUCCESS)
            } else {
                Style::default()
            };

            Line::from(vec![
                Span::styled(
                    format!("[{:8}] ", log.process_name),
                    Style::default().fg(process_name_color(&log.process_name)),
                ),
                Span::styled(&log.content, content_style),
            ])
        })
        .collect();

    let scroll_indicator = ScrollIndicator::new(start_idx, total_logs, visible_height);

    let log_title = if let Some(filter) = filter_process {
        format!(
            "  Logs  {} {} / {}{}",
            filter,
            format_number(start_idx + 1),
            format_number(total_logs),
            scroll_indicator
        )
    } else if !search_query.is_empty() {
        format!(
            "  Logs  {} {} / {}{}",
            search_query,
            format_number(start_idx + 1),
            format_number(total_logs),
            scroll_indicator
        )
    } else {
        format!(
            "  Logs  {} / {}{}",
            format_number(start_idx + 1),
            format_number(total_logs.max(1)),
            scroll_indicator
        )
    };

    let logs_widget = Paragraph::new(log_lines).block(
        Block::default()
            .title(Span::styled(log_title, Style::default().fg(Theme::TEXT_PRIMARY)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::TEXT_MUTED)),
    );

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

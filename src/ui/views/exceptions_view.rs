use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};

use crate::exception::ExceptionTracker;
use crate::ui::formatting::format_relative_time;
use crate::ui::theme::Theme;
use crate::ui::widgets::Spinner;

pub fn render(
    f: &mut Frame,
    area: Rect,
    exception_tracker: &ExceptionTracker,
    selected_exception: usize,
    spinner_frame: usize,
    fade_progress: Option<f32>,
) {
    let stats = exception_tracker.get_stats();
    let groups = exception_tracker.get_grouped_exceptions();

    if groups.is_empty() {
        let loading_spinner = Spinner::new("Waiting for exceptions...", spinner_frame)
            .style(Style::default().fg(Theme::apply_fade_to_color(Theme::text_muted(), fade_progress.unwrap_or(1.0))));

        let block = Theme::block("Exceptions", fade_progress);
        f.render_widget(loading_spinner, block.inner(area));
        f.render_widget(block, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Exception"),
        Cell::from("Count"),
        Cell::from("Last Seen"),
    ])
    .style(Style::default().fg(Theme::warning()));

    let rows: Vec<Row> = groups
        .iter()
        .enumerate()
        .map(|(idx, group)| {
            let style = if idx == selected_exception {
                Style::default()
                    .fg(Theme::text_primary())
                    .bg(Theme::surface())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(group.exception_type.clone()),
                Cell::from(group.count.to_string()),
                Cell::from(format_relative_time(group.last_seen.elapsed())),
            ])
            .style(style)
        })
        .collect();

    let mut table_state = TableState::default();
    table_state.select(Some(selected_exception));

    let table = Table::new(
        rows,
        &[
            ratatui::layout::Constraint::Percentage(60),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Theme::block(format!("Exceptions ({}) - ↑/↓ Navigate, Enter View Details", stats.total_exceptions), fade_progress));

    f.render_stateful_widget(table, area, &mut table_state);
}
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::exception::ExceptionTracker;

pub fn render(f: &mut Frame, area: Rect, exception_tracker: &ExceptionTracker) {
    let stats = exception_tracker.get_stats();
    let groups = exception_tracker.get_grouped_exceptions();

    let header = Row::new(vec![
        Cell::from("Exception"),
        Cell::from("Count"),
        Cell::from("Last Seen"),
    ])
    .style(Style::default().fg(Color::Yellow));

    let rows: Vec<Row> = groups
        .iter()
        .map(|group| {
            Row::new(vec![
                Cell::from(group.exception_type.clone()),
                Cell::from(group.count.to_string()),
                Cell::from(format!("{:.2?}", group.last_seen.elapsed())),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            ratatui::layout::Constraint::Percentage(60),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!("Exceptions ({})", stats.total_exceptions))
            .borders(Borders::ALL),
    );

    f.render_widget(table, area);
}
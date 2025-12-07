use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::test::TestTracker;

pub fn render(f: &mut Frame, area: Rect, test_tracker: &TestTracker) {
    let stats = test_tracker.get_stats();
    let rows = vec![
        Row::new(vec![
            Cell::from("Total runs"),
            Cell::from(stats.total_runs.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Total tests"),
            Cell::from(stats.total_tests_run.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Failed"),
            Cell::from(stats.total_failed.to_string()),
        ])
        .style(if stats.total_failed > 0 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        }),
        Row::new(vec![
            Cell::from("Average duration"),
            Cell::from(format!("{:.2?}", stats.average_duration)),
        ]),
    ];

    let table = Table::new(
        rows,
        &[
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(50),
        ],
    )
    .block(Block::default().title("Test Results").borders(Borders::ALL));

    f.render_widget(table, area);
}
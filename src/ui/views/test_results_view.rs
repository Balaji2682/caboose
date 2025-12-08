use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::test::TestTracker;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, test_tracker: &TestTracker) {
    let stats = test_tracker.get_stats();
    let mut rows = vec![
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

    // Add debugger status (only show if active)
    if test_tracker.is_debugger_active() {
        if let Some(info) = test_tracker.get_debugger_info() {
            let debugger_text = format!(
                "{:?} @ {}:{}",
                info.debugger_type,
                info.file_path.as_deref().unwrap_or("unknown"),
                info.line_number.map(|n| n.to_string()).unwrap_or_else(|| "?".to_string())
            );
            rows.push(
                Row::new(vec![
                    Cell::from("⚡ Debugger"),
                    Cell::from(debugger_text),
                ])
                .style(Style::default().fg(Color::Yellow))
            );
        }
    }

    let table = Table::new(
        rows,
        &[
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(50),
        ],
    )
    .block(Theme::block("Test Results"));

    f.render_widget(table, area);
}
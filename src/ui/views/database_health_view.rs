use ratatui::{ 
    layout::Rect,
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::database::DatabaseHealth;

pub fn render(f: &mut Frame, area: Rect, db_health: &DatabaseHealth) {
    let score = db_health.calculate_health_score();
    let issues = db_health.get_issues();

    let gauge = Gauge::default()
        .block(Block::default().title("Database Health Score").borders(Borders::ALL))
        .gauge_style(db_health.get_health_style())
        .percent(score as u16);

    let issues_text: Vec<String> = issues.iter().map(|issue| format!("- {}", issue.title)).collect();
    let issues_list = Paragraph::new(issues_text.join("\n"))
        .block(Block::default().title("Issues").borders(Borders::ALL));

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(area);

    f.render_widget(gauge, chunks[0]);
    f.render_widget(issues_list, chunks[1]);
}
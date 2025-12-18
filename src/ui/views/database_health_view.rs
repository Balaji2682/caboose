use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::database::DatabaseHealth;
use crate::ui::theme::Theme;
use crate::ui::widgets::Gauge;

pub fn render(
    f: &mut Frame,
    area: Rect,
    db_health: &DatabaseHealth,
    _spinner_frame: usize,
    fade_progress: Option<f32>,
) {
    if db_health.get_stats().total_queries == 0 {
        let block = Theme::block("Database Health", fade_progress);
        let empty = Paragraph::new("Waiting for queries...")
            .style(Style::default().fg(Theme::text_muted()))
            .block(block);
        f.render_widget(empty, area);
        return;
    }

    let score = db_health.calculate_health_score();
    let issues = db_health.get_issues();

    let gauge = Gauge::default()
        .block(Theme::block("Database Health Score", fade_progress))
        .percent(score as u16)
        .label(format!("{}%", score))
        .gradient(vec![Theme::danger(), Theme::warning(), Theme::success()]);

    let issues_text: Vec<String> = issues
        .iter()
        .map(|issue| {
            if issue.description.is_empty() {
                format!("• {}", issue.title)
            } else {
                format!("• {}\n  Query: {}", issue.title, issue.description)
            }
        })
        .collect();
    let issues_list =
        Paragraph::new(issues_text.join("\n")).block(Theme::block("Issues", fade_progress));

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

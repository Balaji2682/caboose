use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::context::RequestContextTracker;
use crate::ui::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    context_tracker: &RequestContextTracker,
    _spinner_frame: usize,
    fade_progress: Option<f32>,
) {
    let requests = context_tracker.get_recent_requests();
    let n_plus_ones = context_tracker.get_all_n_plus_one_issues();

    if requests.is_empty() {
        let block = Theme::block("Query Analysis", fade_progress);
        let empty = Paragraph::new("Waiting for requests...")
            .style(Style::default().fg(Theme::text_muted()))
            .block(block);
        f.render_widget(empty, area);
        return;
    }

    let text = vec![
        format!("Recent requests: {}", requests.len()),
        format!("Detected N+1 issues: {}", n_plus_ones.len()),
    ];

    let block = Theme::block("Query Analysis", fade_progress);
    let para = Paragraph::new(text.join("\n")).block(block);
    f.render_widget(para, area);
}

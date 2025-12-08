use ratatui::{
    layout::Rect,
    widgets::Paragraph,
    Frame,
};

use crate::context::RequestContextTracker;
use crate::ui::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    context_tracker: &RequestContextTracker,
) {
    let requests = context_tracker.get_recent_requests();
    let n_plus_ones = context_tracker.get_all_n_plus_one_issues();

    let text = vec![
        format!("Recent requests: {}", requests.len()),
        format!("Detected N+1 issues: {}", n_plus_ones.len()),
    ];

    let block = Theme::block("Query Analysis");
    let para = Paragraph::new(text.join("\n")).block(block);
    f.render_widget(para, area);
}
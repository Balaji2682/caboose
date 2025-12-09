use ratatui::{
    layout::Rect,
    style::Style,
    widgets::Paragraph,
    Frame,
};

use crate::context::RequestContextTracker;
use crate::ui::theme::Theme;
use crate::ui::widgets::Spinner;

pub fn render(
    f: &mut Frame,
    area: Rect,
    context_tracker: &RequestContextTracker,
    spinner_frame: usize,
    fade_progress: Option<f32>,
) {
    let requests = context_tracker.get_recent_requests();
    let n_plus_ones = context_tracker.get_all_n_plus_one_issues();

    if requests.is_empty() {
        let loading_spinner = Spinner::new("Waiting for requests...", spinner_frame)
            .style(Style::default().fg(Theme::apply_fade_to_color(Theme::text_muted(), fade_progress.unwrap_or(1.0))));
        
        let block = Theme::block("Query Analysis", fade_progress);
        f.render_widget(loading_spinner, block.inner(area));
        f.render_widget(block, area);
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
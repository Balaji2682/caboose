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
    let current_requests = context_tracker.get_current_requests();
    let n_plus_ones = context_tracker.get_all_n_plus_one_issues();

    if requests.is_empty() {
        let block = Theme::block("Query Analysis", fade_progress);
        let debug_text = format!(
            "Waiting for completed requests...\n\n\
            Active requests: {}\n\
            Completed requests: {}\n\n\
            Note: Requests appear here after Rails logs show:\n\
            1. 'Started GET /path' (request start)\n\
            2. SQL queries during request\n\
            3. 'Completed 200' (request end)",
            current_requests.len(),
            requests.len()
        );
        let empty = Paragraph::new(debug_text)
            .style(Style::default().fg(Theme::text_muted()))
            .block(block);
        f.render_widget(empty, area);
        return;
    }

    // Show summary and list of recent requests
    let mut text = vec![
        format!("📊 Recent requests: {}", requests.len()),
        format!("⚠️  Detected N+1 issues: {}", n_plus_ones.len()),
        format!("🔄 Active requests: {}", current_requests.len()),
        String::new(),
        "Recent Requests:".to_string(),
    ];

    // Show last 10 requests
    for (i, req) in requests.iter().rev().take(10).enumerate() {
        let path = req.context.path.as_deref().unwrap_or("<unknown>");
        let status = req.status.unwrap_or(0);
        let queries = req.context.query_count();
        let duration = req.total_duration.unwrap_or(0.0);

        let status_icon = if status >= 500 { "❌" }
                         else if status >= 400 { "⚠️" }
                         else if status >= 300 { "↪️" }
                         else { "✅" };

        text.push(format!(
            "  {}. {} {} - {} queries ({:.1}ms)",
            i + 1, status_icon, path, queries, duration
        ));
    }

    let block = Theme::block("Query Analysis", fade_progress);
    let para = Paragraph::new(text.join("\n")).block(block);
    f.render_widget(para, area);
}

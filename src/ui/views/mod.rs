pub mod database_health_view;
pub mod exception_detail_view;
pub mod exceptions_view;
/// View modules - Each major view in its own file
pub mod logs_view;
pub mod query_analysis_view;
pub mod request_detail_view;
pub mod test_results_view;

use ratatui::Frame;

/// Trait for renderable views
pub trait View {
    /// Render the view to the terminal frame
    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);

    /// Get the view's title
    fn title(&self) -> &str;
}

use ratatui::{buffer::Buffer, layout::Rect, style::Style, text::Span, widgets::Widget};

/// Spinner placeholder. Rendering is deliberately no-op to avoid glyph bleed in tight layouts.
pub struct Spinner<'a> {
    #[allow(dead_code)]
    text: Span<'a>,
    #[allow(dead_code)]
    style: Style,
    #[allow(dead_code)]
    frame: usize,
}

impl<'a> Spinner<'a> {
    pub fn new(text: &'a str, frame: usize) -> Self {
        Self {
            text: Span::raw(text),
            style: Style::default(),
            frame,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for Spinner<'a> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {
        // Intentionally render nothing to prevent spinner artifacts.
    }
}

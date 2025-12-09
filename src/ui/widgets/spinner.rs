
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Span,
    widgets::Widget,
};

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const SPINNER_SPEED: usize = 10; // Change frame every 10 ticks

pub struct Spinner<'a> {
    text: Span<'a>,
    style: Style,
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
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let spinner_char = SPINNER_CHARS[(self.frame / SPINNER_SPEED) % SPINNER_CHARS.len()];
        let spinner_span = Span::styled(
            spinner_char.to_string(),
            self.style,
        );

        let combined_text = format!("{} {}", spinner_span, self.text);
        let text_span = Span::styled(combined_text, self.style);

        // Center the spinner horizontally and vertically
        let x = area.x + (area.width.saturating_sub(text_span.width() as u16)) / 2;
        let y = area.y + (area.height / 2);

        if y < area.bottom() && x < area.right() {
            buf.set_span(x, y, &text_span, text_span.width() as u16);
        }
    }
}

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Block, Widget},
};

/// A widget to display a progress gauge.
///
/// This gauge is highly customizable and supports gradients.
#[derive(Debug, Clone)]
pub struct Gauge<'a> {
    block: Option<Block<'a>>,
    percent: u16,
    label: Option<Span<'a>>,
    gradient: Vec<Color>,
    gauge_style: Style,
}

impl<'a> Default for Gauge<'a> {
    fn default() -> Self {
        Self {
            block: None,
            percent: 0,
            label: None,
            gradient: vec![Color::Green, Color::Yellow, Color::Red],
            gauge_style: Style::default(),
        }
    }
}

impl<'a> Gauge<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn percent(mut self, percent: u16) -> Self {
        self.percent = percent.min(100);
        self
    }

    pub fn label(mut self, label: impl Into<Span<'a>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn gradient(mut self, gradient: Vec<Color>) -> Self {
        self.gradient = gradient;
        self
    }

    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }

    fn get_gradient_color(&self, percent: u16) -> Color {
        if self.gradient.is_empty() {
            return self.gauge_style.fg.unwrap_or(Color::Reset);
        }
        let index = (percent as usize * (self.gradient.len() - 1)) / 100;
        self.gradient[index]
    }
}

impl<'a> Widget for Gauge<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let gauge_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if gauge_area.height < 1 {
            return;
        }

        let filled_width = (gauge_area.width as u16 * self.percent) / 100;

        // Render filled portion with gradient
        for i in 0..filled_width {
            let p = (i * 100) / gauge_area.width;
            let color = self.get_gradient_color(p);
            let cell = &mut buf[(gauge_area.x + i, gauge_area.y)];
            cell.set_symbol(symbols::block::FULL)
                .set_fg(color);
        }

        // Render empty portion
        for i in filled_width..gauge_area.width {
            let cell = &mut buf[(gauge_area.x + i, gauge_area.y)];
            cell.set_symbol(" ")
                .set_style(self.gauge_style);
        }

        // Render label
        if let Some(label) = self.label {
            let label_width = label.width() as u16;
            let label_col = gauge_area.x + (gauge_area.width - label_width) / 2;
            buf.set_span(label_col, gauge_area.y, &label, label_width);
        }
    }
}

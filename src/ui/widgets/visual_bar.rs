/// Visual bar widget for proportional display

/// VisualBar widget - displays a proportional bar
pub struct VisualBar {
    value: f64,
    max_value: f64,
    width: usize,
    char: char,
}

impl VisualBar {
    /// Create a new visual bar
    pub fn new(value: f64, max_value: f64, width: usize) -> Self {
        Self {
            value: value.max(0.0),
            max_value: max_value.max(1.0),
            width,
            char: '▅',
        }
    }

    /// Set custom bar character
    pub fn char(mut self, ch: char) -> Self {
        self.char = ch;
        self
    }

    /// Render the bar as a string with padding to full width
    pub fn render(&self) -> String {
        let bar_width = if self.max_value > 0.0 {
            ((self.value / self.max_value) * self.width as f64) as usize
        } else {
            0
        };
        let bar_width = bar_width
            .max(if self.value > 0.0 { 1 } else { 0 })
            .min(self.width);

        format!(
            "{:width$}",
            self.char.to_string().repeat(bar_width),
            width = self.width
        )
    }

    /// Render without padding
    pub fn render_compact(&self) -> String {
        let bar_width = if self.max_value > 0.0 {
            ((self.value / self.max_value) * self.width as f64) as usize
        } else {
            0
        };
        let bar_width = bar_width
            .max(if self.value > 0.0 { 1 } else { 0 })
            .min(self.width);

        self.char.to_string().repeat(bar_width)
    }
}

impl std::fmt::Display for VisualBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render_compact())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_bar() {
        let bar = VisualBar::new(50.0, 100.0, 10);
        assert_eq!(bar.render_compact(), "▅▅▅▅▅");
    }

    #[test]
    fn test_visual_bar_full() {
        let bar = VisualBar::new(100.0, 100.0, 10);
        assert_eq!(bar.render_compact(), "▅▅▅▅▅▅▅▅▅▅");
    }

    #[test]
    fn test_visual_bar_empty() {
        let bar = VisualBar::new(0.0, 100.0, 10);
        assert_eq!(bar.render_compact(), "");
    }

    #[test]
    fn test_visual_bar_custom_char() {
        let bar = VisualBar::new(50.0, 100.0, 10).char('█');
        assert_eq!(bar.render_compact(), "█████");
    }

    #[test]
    fn test_visual_bar_padded() {
        let bar = VisualBar::new(50.0, 100.0, 10);
        let rendered = bar.render();
        // Check character count, not byte length (Unicode chars are 3 bytes each)
        assert_eq!(rendered.chars().count(), 10);
        assert!(rendered.starts_with("▅▅▅▅▅"));
    }
}

/// Progress gauge widget for displaying percentages visually

/// Gauge widget - displays a visual progress bar
pub struct Gauge {
    percentage: f64,
    width: usize,
    filled_char: char,
    empty_char: char,
}

impl Gauge {
    /// Create a new gauge with default characters (█ and ░)
    pub fn new(percentage: f64, width: usize) -> Self {
        Self {
            percentage: percentage.max(0.0).min(100.0),
            width,
            filled_char: '█',
            empty_char: '░',
        }
    }

    /// Set custom filled character
    pub fn filled_char(mut self, ch: char) -> Self {
        self.filled_char = ch;
        self
    }

    /// Set custom empty character
    pub fn empty_char(mut self, ch: char) -> Self {
        self.empty_char = ch;
        self
    }

    /// Render the gauge as a string
    pub fn render(&self) -> String {
        let filled = ((self.percentage / 100.0) * self.width as f64) as usize;
        let empty = self.width.saturating_sub(filled);

        format!(
            "{}{}",
            self.filled_char.to_string().repeat(filled),
            self.empty_char.to_string().repeat(empty)
        )
    }

    /// Get the percentage value
    pub fn percentage(&self) -> f64 {
        self.percentage
    }
}

impl std::fmt::Display for Gauge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_rendering() {
        let gauge = Gauge::new(50.0, 10);
        assert_eq!(gauge.render(), "█████░░░░░");

        let gauge = Gauge::new(100.0, 10);
        assert_eq!(gauge.render(), "██████████");

        let gauge = Gauge::new(0.0, 10);
        assert_eq!(gauge.render(), "░░░░░░░░░░");
    }

    #[test]
    fn test_gauge_custom_chars() {
        let gauge = Gauge::new(50.0, 10)
            .filled_char('▓')
            .empty_char('░');
        assert_eq!(gauge.render(), "▓▓▓▓▓░░░░░");
    }

    #[test]
    fn test_gauge_bounds() {
        let gauge = Gauge::new(150.0, 10); // Over 100%
        assert_eq!(gauge.percentage(), 100.0);

        let gauge = Gauge::new(-10.0, 10); // Under 0%
        assert_eq!(gauge.percentage(), 0.0);
    }
}

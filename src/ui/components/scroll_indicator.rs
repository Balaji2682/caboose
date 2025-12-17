/// Scroll indicator component for showing position in scrollable content

/// Scroll indicator to show current position in a list
pub struct ScrollIndicator {
    current: usize,
    total: usize,
    visible: usize,
}

impl ScrollIndicator {
    /// Create a new scroll indicator
    pub fn new(current: usize, total: usize, visible: usize) -> Self {
        Self {
            current,
            total,
            visible,
        }
    }

    /// Check if scrolling is needed
    pub fn needs_scrolling(&self) -> bool {
        self.total > self.visible
    }

    /// Get scroll percentage
    pub fn percentage(&self) -> f64 {
        if !self.needs_scrolling() {
            return 100.0;
        }

        let max_scroll = self.total.saturating_sub(self.visible);
        if max_scroll == 0 {
            return 100.0;
        }

        (self.current as f64 / max_scroll as f64 * 100.0).min(100.0)
    }

    /// Render as a string (empty if no scrolling needed)
    pub fn render(&self) -> String {
        if !self.needs_scrolling() {
            return String::new();
        }

        format!(" {}%", self.percentage() as usize)
    }

    /// Render with custom format
    pub fn render_with_format(&self, format: &str) -> String {
        if !self.needs_scrolling() {
            return String::new();
        }

        format.replace("{}", &(self.percentage() as usize).to_string())
    }
}

impl std::fmt::Display for ScrollIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_indicator_no_scroll() {
        let indicator = ScrollIndicator::new(0, 10, 20);
        assert!(!indicator.needs_scrolling());
        assert_eq!(indicator.render(), "");
    }

    #[test]
    fn test_scroll_indicator_needs_scroll() {
        let indicator = ScrollIndicator::new(0, 100, 10);
        assert!(indicator.needs_scrolling());
        assert_eq!(indicator.percentage(), 0.0);
    }

    #[test]
    fn test_scroll_indicator_middle() {
        let indicator = ScrollIndicator::new(45, 100, 10);
        assert!(indicator.needs_scrolling());
        assert_eq!(indicator.percentage(), 50.0);
    }

    #[test]
    fn test_scroll_indicator_end() {
        let indicator = ScrollIndicator::new(90, 100, 10);
        assert!(indicator.needs_scrolling());
        assert_eq!(indicator.percentage(), 100.0);
    }

    #[test]
    fn test_scroll_indicator_format() {
        let indicator = ScrollIndicator::new(45, 100, 10);
        assert_eq!(indicator.render_with_format("[{}%]"), "[50%]");
    }
}

/// Sparkline widget for displaying trend data

/// Sparkline widget - displays a mini chart using Unicode characters
pub struct Sparkline<'a> {
    values: &'a [f64],
}

impl<'a> Sparkline<'a> {
    const CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    /// Create a new sparkline from values
    pub fn new(values: &'a [f64]) -> Self {
        Self { values }
    }

    /// Render the sparkline as a string
    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return String::new();
        }

        let max = self.values.iter().fold(0.0f64, |a, &b| a.max(b));

        if max == 0.0 {
            return Self::CHARS[0].to_string().repeat(self.values.len());
        }

        self.values
            .iter()
            .map(|&v| {
                let index = ((v / max) * (Self::CHARS.len() - 1) as f64) as usize;
                Self::CHARS[index.min(Self::CHARS.len() - 1)]
            })
            .collect()
    }
}

impl<'a> std::fmt::Display for Sparkline<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparkline_empty() {
        let sparkline = Sparkline::new(&[]);
        assert_eq!(sparkline.render(), "");
    }

    #[test]
    fn test_sparkline_all_zeros() {
        let sparkline = Sparkline::new(&[0.0, 0.0, 0.0]);
        assert_eq!(sparkline.render(), "▁▁▁");
    }

    #[test]
    fn test_sparkline_values() {
        let sparkline = Sparkline::new(&[1.0, 2.0, 3.0, 2.0, 1.0]);
        let result = sparkline.render();
        assert_eq!(result.chars().count(), 5);
        // Middle value should be highest character
        assert!(result.chars().nth(2).unwrap() > result.chars().nth(0).unwrap());
    }
}

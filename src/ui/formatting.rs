/// Formatting utilities for consistent display of numbers, durations, and text
use std::time::Duration;

/// Format elapsed time in human-readable relative format
///
/// # Examples
/// ```rust
/// use caboose::ui::formatting::format_relative_time;
/// use std::time::Duration;
///
/// assert_eq!(format_relative_time(Duration::from_secs(30)), "<1 min");
/// assert_eq!(format_relative_time(Duration::from_secs(90)), "1 min ago");
/// assert_eq!(format_relative_time(Duration::from_secs(150)), "2 mins ago");
/// assert_eq!(format_relative_time(Duration::from_secs(3700)), "1 hr ago");
/// ```
pub fn format_relative_time(duration: Duration) -> String {
    let seconds = duration.as_secs();

    if seconds < 60 {
        "<1 min".to_string()
    } else if seconds < 120 {
        "1 min ago".to_string()
    } else if seconds < 3600 {
        format!("{} mins ago", seconds / 60)
    } else if seconds < 7200 {
        "1 hr ago".to_string()
    } else if seconds < 86400 {
        format!("{} hrs ago", seconds / 3600)
    } else if seconds < 172800 {
        "1 day ago".to_string()
    } else {
        format!("{} days ago", seconds / 86400)
    }
}

/// Format numbers with thousands separators
///
/// # Examples
/// ```rust
/// use caboose::ui::formatting::format_number;
///
/// assert_eq!(format_number(1234), "1,234");
/// assert_eq!(format_number(1000000), "1,000,000");
/// ```
pub fn format_number(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

/// Format duration in seconds to human-readable format
///
/// # Examples
/// ```rust
/// use caboose::ui::formatting::format_duration;
///
/// assert_eq!(format_duration(45), "45s");
/// assert_eq!(format_duration(90), "1m 30s");
/// assert_eq!(format_duration(3700), "1h 1m");
/// ```
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// Format milliseconds with appropriate precision
///
/// # Examples
/// ```rust
/// use caboose::ui::formatting::format_ms;
///
/// assert_eq!(format_ms(0.5), "0.50ms");
/// assert_eq!(format_ms(45.2), "45.2ms");
/// assert_eq!(format_ms(1250.0), "1.25s");
/// ```
pub fn format_ms(ms: f64) -> String {
    if ms < 1.0 {
        format!("{:.2}ms", ms)
    } else if ms < 1000.0 {
        format!("{:.1}ms", ms)
    } else {
        format!("{:.2}s", ms / 1000.0)
    }
}

/// Format percentage with consistent precision
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format bytes to human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let size = bytes as f64;
    let base = 1024_f64;
    let exp = (size.ln() / base.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);

    let value = size / base.powi(exp as i32);
    format!("{:.2} {}", value, UNITS[exp])
}

/// Truncate text with ellipsis if it exceeds max length
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Pad or truncate text to exact width
pub fn pad_or_truncate(text: &str, width: usize) -> String {
    if text.len() >= width {
        truncate(text, width)
    } else {
        format!("{:width$}", text, width = width)
    }
}

/// Format Duration to human-readable string
pub fn format_rust_duration(duration: Duration) -> String {
    format_duration(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }

    #[test]
    fn test_format_ms() {
        assert_eq!(format_ms(0.5), "0.50ms");
        assert_eq!(format_ms(50.0), "50.0ms");
        assert_eq!(format_ms(1500.0), "1.50s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 1), "...");
    }

    #[test]
    fn test_pad_or_truncate() {
        assert_eq!(pad_or_truncate("hello", 10), "hello     ");
        assert_eq!(pad_or_truncate("hello world", 8), "hello...");
    }
}

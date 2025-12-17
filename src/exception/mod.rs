use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Exception {
    pub exception_type: String,
    pub message: String,
    pub backtrace: Vec<String>,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub timestamp: Instant,
    pub context: Option<String>, // HTTP request context if available
}

#[derive(Debug, Clone)]
pub struct ExceptionGroup {
    pub fingerprint: String,
    pub exception_type: String,
    pub message_pattern: String,
    pub count: usize,
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub sample_exception: Exception,
    pub occurrences: Vec<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExceptionSeverity {
    Low,      // Warnings, expected errors
    Medium,   // Handled exceptions
    High,     // Unhandled errors
    Critical, // Fatal errors, crashes
}

impl ExceptionSeverity {
    pub fn from_exception_type(exc_type: &str) -> Self {
        match exc_type {
            // Critical errors
            "NoMemoryError" | "SystemStackError" | "SignalException" => Self::Critical,

            // High severity
            "NameError" | "NoMethodError" | "ArgumentError" | "TypeError" | "ZeroDivisionError"
            | "SyntaxError" => Self::High,

            // Medium severity
            "ActiveRecord::RecordNotFound"
            | "ActiveRecord::RecordInvalid"
            | "ActionController::RoutingError"
            | "ActiveRecord::RecordNotUnique" => Self::Medium,

            // Low severity
            _ => Self::Low,
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Critical => "✗",
            Self::High => "⚠",
            Self::Medium => "!",
            Self::Low => "i",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExceptionStats {
    pub total_exceptions: usize,
    pub unique_exceptions: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

pub struct ExceptionTracker {
    exceptions: Arc<Mutex<Vec<Exception>>>,
    grouped_exceptions: Arc<Mutex<HashMap<String, ExceptionGroup>>>,
    stats: Arc<Mutex<ExceptionStats>>,
    current_exception: Arc<Mutex<Option<Exception>>>,
    parsing_backtrace: Arc<Mutex<bool>>,
}

impl ExceptionTracker {
    pub fn new() -> Self {
        Self {
            exceptions: Arc::new(Mutex::new(Vec::new())),
            grouped_exceptions: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ExceptionStats::default())),
            current_exception: Arc::new(Mutex::new(None)),
            parsing_backtrace: Arc::new(Mutex::new(false)),
        }
    }

    pub fn parse_line(&self, line: &str) {
        // Check if we're currently parsing a backtrace
        let mut parsing = self.parsing_backtrace.lock().unwrap();

        if *parsing {
            // Check if this is a backtrace line
            if Self::is_backtrace_line(line) {
                self.add_backtrace_line(line);
                return;
            } else {
                // End of backtrace, finalize exception
                *parsing = false;
                self.finalize_current_exception();
            }
        }

        // Check for new exception
        if let Some(exception) = Self::detect_exception(line) {
            let mut current = self.current_exception.lock().unwrap();
            *current = Some(exception);
            *parsing = true;
        }
    }

    fn detect_exception(line: &str) -> Option<Exception> {
        // Rails exception format: "ExceptionType (message):"
        // or "ExceptionType: message"

        // Pattern 1: "NameError (undefined local variable or method)"
        if let Some(pos) = line.find(" (") {
            let exc_type = line[..pos].trim();
            if Self::is_exception_type(exc_type) {
                if let Some(end_pos) = line[pos..].find("):") {
                    let message = &line[pos + 2..pos + end_pos];
                    return Some(Exception {
                        exception_type: exc_type.to_string(),
                        message: message.to_string(),
                        backtrace: Vec::new(),
                        file_path: None,
                        line_number: None,
                        timestamp: Instant::now(),
                        context: None,
                    });
                }
            }
        }

        // Pattern 2: "NameError: undefined local variable"
        if let Some(pos) = line.find(": ") {
            let exc_type = line[..pos].trim();
            if Self::is_exception_type(exc_type) {
                let message = &line[pos + 2..];
                return Some(Exception {
                    exception_type: exc_type.to_string(),
                    message: message.to_string(),
                    backtrace: Vec::new(),
                    file_path: None,
                    line_number: None,
                    timestamp: Instant::now(),
                    context: None,
                });
            }
        }

        None
    }

    fn is_exception_type(text: &str) -> bool {
        // Common Ruby/Rails exception patterns
        text.ends_with("Error")
            || text.ends_with("Exception")
            || text.contains("::") && (text.contains("Error") || text.contains("Exception"))
    }

    fn is_backtrace_line(line: &str) -> bool {
        // Backtrace lines typically start with file paths or indentation
        line.trim_start().starts_with("from ")
            || line.trim_start().starts_with("/")
            || line.trim_start().starts_with("app/")
            || line.trim_start().starts_with("lib/")
            || line.trim_start().starts_with("vendor/")
            || (line.starts_with("  ") && line.contains(".rb:"))
    }

    fn add_backtrace_line(&self, line: &str) {
        let mut current = self.current_exception.lock().unwrap();
        if let Some(ref mut exception) = *current {
            let cleaned_line = line.trim().to_string();
            exception.backtrace.push(cleaned_line.clone());

            // Extract file path and line number from first backtrace line if not set
            if exception.file_path.is_none() {
                if let Some((file, line_num)) = Self::parse_backtrace_location(&cleaned_line) {
                    exception.file_path = Some(file);
                    exception.line_number = Some(line_num);
                }
            }
        }
    }

    fn parse_backtrace_location(line: &str) -> Option<(String, usize)> {
        // Format: "from /path/to/file.rb:123:in `method_name'"
        // or "app/controllers/users_controller.rb:45:in `create'"

        let cleaned = line.trim_start_matches("from ").trim();

        if let Some(colon_pos) = cleaned.find(":") {
            let file_path = &cleaned[..colon_pos];
            let rest = &cleaned[colon_pos + 1..];

            if let Some(line_end) = rest.find(":") {
                let line_num_str = &rest[..line_end];
                if let Ok(line_num) = line_num_str.parse::<usize>() {
                    return Some((file_path.to_string(), line_num));
                }
            }
        }

        None
    }

    fn finalize_current_exception(&self) {
        let mut current = self.current_exception.lock().unwrap();
        if let Some(exception) = current.take() {
            // Generate fingerprint for grouping
            let fingerprint = Self::generate_fingerprint(&exception);

            // Update stats
            let mut stats = self.stats.lock().unwrap();
            stats.total_exceptions += 1;

            let severity = ExceptionSeverity::from_exception_type(&exception.exception_type);
            match severity {
                ExceptionSeverity::Critical => stats.critical_count += 1,
                ExceptionSeverity::High => stats.high_count += 1,
                ExceptionSeverity::Medium => stats.medium_count += 1,
                ExceptionSeverity::Low => stats.low_count += 1,
            }

            // Group exception
            let mut grouped = self.grouped_exceptions.lock().unwrap();
            if let Some(group) = grouped.get_mut(&fingerprint) {
                group.count += 1;
                group.last_seen = Instant::now();
                group.occurrences.push(Instant::now());
                // Keep only last 10 occurrences per group
                if group.occurrences.len() > 10 {
                    group.occurrences.remove(0);
                }
            } else {
                stats.unique_exceptions += 1;
                grouped.insert(
                    fingerprint.clone(),
                    ExceptionGroup {
                        fingerprint: fingerprint.clone(),
                        exception_type: exception.exception_type.clone(),
                        message_pattern: Self::normalize_message(&exception.message),
                        count: 1,
                        first_seen: Instant::now(),
                        last_seen: Instant::now(),
                        sample_exception: exception.clone(),
                        occurrences: vec![Instant::now()],
                    },
                );
            }

            // Store in recent exceptions (keep last 100)
            let mut exceptions = self.exceptions.lock().unwrap();
            exceptions.push(exception);
            if exceptions.len() > 100 {
                exceptions.remove(0);
            }
        }
    }

    fn generate_fingerprint(exception: &Exception) -> String {
        // Generate a fingerprint based on exception type and normalized message
        let normalized_msg = Self::normalize_message(&exception.message);
        format!("{}:{}", exception.exception_type, normalized_msg)
    }

    fn normalize_message(message: &str) -> String {
        // Remove dynamic parts like IDs, numbers, specific values
        let mut normalized = message.to_string();

        // Replace numbers with placeholder
        normalized = regex::Regex::new(r"\d+")
            .unwrap()
            .replace_all(&normalized, "N")
            .to_string();

        // Replace quoted strings with placeholder
        normalized = regex::Regex::new(r#""[^"]*""#)
            .unwrap()
            .replace_all(&normalized, "\"STR\"")
            .to_string();

        // Replace single-quoted strings
        normalized = regex::Regex::new(r"'[^']*'")
            .unwrap()
            .replace_all(&normalized, "'STR'")
            .to_string();

        // Truncate if too long
        if normalized.len() > 100 {
            normalized.truncate(100);
        }

        normalized
    }

    pub fn get_recent_exceptions(&self, limit: usize) -> Vec<Exception> {
        let exceptions = self.exceptions.lock().unwrap();
        exceptions.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_grouped_exceptions(&self) -> Vec<ExceptionGroup> {
        let grouped = self.grouped_exceptions.lock().unwrap();
        let mut groups: Vec<ExceptionGroup> = grouped.values().cloned().collect();

        // Sort by count (most frequent first)
        groups.sort_by(|a, b| b.count.cmp(&a.count));

        groups
    }

    pub fn get_stats(&self) -> ExceptionStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn get_top_exceptions(&self, limit: usize) -> Vec<ExceptionGroup> {
        let groups = self.get_grouped_exceptions();
        groups.into_iter().take(limit).collect()
    }

    pub fn get_critical_exceptions(&self) -> Vec<ExceptionGroup> {
        self.get_grouped_exceptions()
            .into_iter()
            .filter(|g| {
                ExceptionSeverity::from_exception_type(&g.exception_type)
                    == ExceptionSeverity::Critical
            })
            .collect()
    }

    pub fn get_exception_rate(&self) -> f64 {
        // Calculate exceptions per minute based on recent occurrences
        let groups = self.get_grouped_exceptions();
        let now = Instant::now();
        let mut recent_count = 0;

        for group in groups {
            for occurrence in &group.occurrences {
                let age = now.duration_since(*occurrence).as_secs();
                if age < 60 {
                    recent_count += 1;
                }
            }
        }

        recent_count as f64
    }

    pub fn clear_stats(&self) {
        let mut exceptions = self.exceptions.lock().unwrap();
        exceptions.clear();

        let mut grouped = self.grouped_exceptions.lock().unwrap();
        grouped.clear();

        let mut stats = self.stats.lock().unwrap();
        *stats = ExceptionStats::default();
    }
}

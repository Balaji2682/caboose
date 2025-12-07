use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub status: Option<u16>,
    pub duration: Option<f64>,
    pub controller: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SqlQuery {
    pub query: String,
    pub duration: Option<f64>,
    pub rows: Option<usize>,
    pub name: Option<String>, // e.g., "User Load"
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    HttpRequest(HttpRequest),
    SqlQuery(SqlQuery),
    Error(String),
    Info(String),
}

pub struct RailsLogParser;

impl RailsLogParser {
    // Regex patterns (compiled once)
    fn http_start_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r#"Started (GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS) "([^"]+)""#).unwrap()
        })
    }

    fn processing_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"Processing by ([^#]+)#(\w+)").unwrap()
        })
    }

    fn completed_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"Completed (\d+) .* in (\d+(?:\.\d+)?)ms").unwrap()
        })
    }

    fn sql_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Matches Rails SQL logs like: User Load (0.5ms)  SELECT "users".* FROM "users"
            Regex::new(r"([\w\s]+)\s*\((\d+(?:\.\d+)?)ms\)\s+(SELECT|INSERT|UPDATE|DELETE).+").unwrap()
        })
    }

    fn sql_simple_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"(SELECT|INSERT|UPDATE|DELETE|BEGIN|COMMIT|ROLLBACK)[^(]*").unwrap()
        })
    }

    pub fn parse_line(line: &str) -> Option<LogEvent> {
        // Check for HTTP request start
        if let Some(caps) = Self::http_start_pattern().captures(line) {
            return Some(LogEvent::HttpRequest(HttpRequest {
                method: caps[1].to_string(),
                path: caps[2].to_string(),
                status: None,
                duration: None,
                controller: None,
                action: None,
            }));
        }

        // Check for processing (controller#action)
        if let Some(caps) = Self::processing_pattern().captures(line) {
            return Some(LogEvent::Info(format!(
                "Processing: {}#{}",
                &caps[1], &caps[2]
            )));
        }

        // Check for completed request
        if let Some(caps) = Self::completed_pattern().captures(line) {
            let status: u16 = caps[1].parse().unwrap_or(0);
            let duration: f64 = caps[2].parse().unwrap_or(0.0);
            return Some(LogEvent::HttpRequest(HttpRequest {
                method: String::new(),
                path: String::new(),
                status: Some(status),
                duration: Some(duration),
                controller: None,
                action: None,
            }));
        }

        // Check for SQL query (Rails format with timing)
        if let Some(caps) = Self::sql_pattern().captures(line) {
            let name = caps[1].trim().to_string();
            let duration: f64 = caps[2].parse().unwrap_or(0.0);
            let query = caps[0].to_string();

            return Some(LogEvent::SqlQuery(SqlQuery {
                query,
                duration: Some(duration),
                rows: None,
                name: Some(name),
            }));
        }

        // Fallback to simple SQL pattern
        if let Some(_caps) = Self::sql_simple_pattern().captures(line) {
            return Some(LogEvent::SqlQuery(SqlQuery {
                query: line.to_string(),
                duration: None,
                rows: None,
                name: None,
            }));
        }

        // Check for errors
        if line.contains("ERROR") || line.contains("FATAL") || line.contains("Exception") {
            return Some(LogEvent::Error(line.to_string()));
        }

        None
    }

    pub fn highlight_sql(query: &str) -> String {
        let keywords = [
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE",
            "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON",
            "GROUP BY", "ORDER BY", "LIMIT", "OFFSET",
            "AND", "OR", "NOT", "IN", "LIKE", "BETWEEN",
            "CREATE", "ALTER", "DROP", "TABLE", "INDEX",
            "BEGIN", "COMMIT", "ROLLBACK",
        ];

        let mut highlighted = query.to_string();
        for keyword in keywords {
            highlighted = highlighted.replace(keyword, &format!("[KW]{}[/KW]", keyword));
        }
        highlighted
    }
}

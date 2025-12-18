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
    RailsStartupError(RailsError),
    Info(String),
}

#[derive(Debug, Clone)]
pub enum RailsError {
    PendingMigrations,
    DatabaseNotFound(String),
    DatabaseConnectionFailed(String),
    MissingGem(String),
    BundlerError(String),
    ConfigurationError(String),
    PortInUse(u16),
    GenericStartupError(String),
}

pub struct RailsLogParser;

impl RailsLogParser {
    // Regex patterns (compiled once)

    /// Strip timestamp prefixes like [INFO 2018-07-01 11:55:04 65048] : or Rails tagged format
    /// Examples:
    /// - D, [2024-01-15T10:30:45.043111 #6322] DEBUG -- : Started GET...
    /// - I, [2024-01-15T10:30:45.043111 #6322]  INFO -- : Started GET...
    /// - [INFO 2018-07-01 11:55:04 65048] : Started GET...
    fn strip_timestamp_prefix(line: &str) -> &str {
        static TIMESTAMP_PREFIX: OnceLock<Regex> = OnceLock::new();
        let re = TIMESTAMP_PREFIX.get_or_init(|| {
            // Match various timestamp formats:
            // 1. Rails tagged: D, [2024-01-15T10:30:45.043111 #6322] DEBUG -- :
            // 2. Bracketed: [INFO 2018-07-01 11:55:04 65048] :
            // 3. Plain timestamp: 2024-01-15 10:30:45
            Regex::new(r"^(?:[DIWEF],\s*[^\]]+]\s+(?:DEBUG|INFO|WARN|ERROR|FATAL)\s+--\s*:\s*|[^\]]+]\s*:\s*|\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}[^\s]*\s+)").unwrap()
        });

        if let Some(m) = re.find(line) {
            &line[m.end()..]
        } else {
            line
        }
    }

    fn http_start_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Match Rails 6/7 format: Started METHOD "PATH" for IP at TIMESTAMP
            // Supports both quoted and unquoted paths
            Regex::new(r#"Started (GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+"([^"]+)"|Started (GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+([^\s]+)"#).unwrap()
        })
    }

    fn http_start_keyvalue_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Match key-value format: method=POST path=/users format=html
            // Captures method and path from anywhere in the line
            Regex::new(r"method=([A-Z]+).*?path=([^\s]+)").unwrap()
        })
    }

    fn lograge_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Match Lograge single-line format:
            // method=GET path=/users ... status=200 duration=123.45
            // Must have method, path, status, and duration to be a complete request
            Regex::new(r"method=([A-Z]+).*?path=([^\s]+).*?status=(\d+).*?duration=(\d+(?:\.\d+)?)").unwrap()
        })
    }

    fn processing_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"Processing by ([^#]+)#(\w+)").unwrap())
    }

    fn completed_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Match various completion formats:
            // - Completed 200 OK in 45ms (Views: 32.1ms | ActiveRecord: 8.9ms)
            // - Completed 302 Found in 25ms (ActiveRecord: 6.6ms | Allocations: 2809)
            // - Completed 200 OK in 104ms (Views: 90.8ms | ActiveRecord: 0.4ms)
            Regex::new(r"Completed (\d+)\s+\w+\s+in\s+(\d+(?:\.\d+)?)ms").unwrap()
        })
    }

    fn sql_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Matches Rails 6/7 SQL logs, including Rails 7 query comments:
            // User Load (0.5ms)  SELECT "users".* FROM "users" /*application='Blog'*/
            // Allow for optional query comments at the end
            Regex::new(r"([\w\s]+)\s*\((\d+(?:\.\d+)?)ms\)\s+(SELECT|INSERT|UPDATE|DELETE).+?(?:/\*.*?\*/)?$")
                .unwrap()
        })
    }

    fn sql_simple_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Match SQL keywords anywhere in the line, strip query comments
            Regex::new(r"(SELECT|INSERT|UPDATE|DELETE|BEGIN|COMMIT|ROLLBACK)").unwrap()
        })
    }

    pub fn parse_line(line: &str) -> Option<LogEvent> {
        // Strip timestamp prefixes for Rails 6/7 compatibility
        let clean_line = Self::strip_timestamp_prefix(line);

        // Check for Rails-specific startup errors first
        if let Some(rails_error) = Self::detect_rails_error(clean_line) {
            return Some(LogEvent::RailsStartupError(rails_error));
        }

        // Check for Lograge single-line format FIRST (has status + duration)
        // This takes priority because it's a complete request in one line
        if let Some(caps) = Self::lograge_pattern().captures(clean_line) {
            let method = caps[1].to_string();
            let path = caps[2].to_string();
            let status: u16 = caps[3].parse().unwrap_or(0);
            let duration: f64 = caps[4].parse().unwrap_or(0.0);

            // For Lograge, we create a complete request immediately
            // First emit a "start" event
            return Some(LogEvent::HttpRequest(HttpRequest {
                method: method.clone(),
                path: path.clone(),
                status: Some(status),
                duration: Some(duration),
                controller: None,
                action: None,
            }));
        }

        // Check for HTTP request start (traditional format)
        if let Some(caps) = Self::http_start_pattern().captures(clean_line) {
            // Handle both quoted and unquoted path formats
            let method = caps.get(1).or_else(|| caps.get(3))?.as_str().to_string();
            let path = caps.get(2).or_else(|| caps.get(4))?.as_str().to_string();

            return Some(LogEvent::HttpRequest(HttpRequest {
                method,
                path,
                status: None,
                duration: None,
                controller: None,
                action: None,
            }));
        }

        // Check for HTTP request start (key-value format: method=POST path=/users)
        // Only if it doesn't have status/duration (otherwise Lograge would catch it)
        if let Some(caps) = Self::http_start_keyvalue_pattern().captures(clean_line) {
            let method = caps[1].to_string();
            let path = caps[2].to_string();

            return Some(LogEvent::HttpRequest(HttpRequest {
                method,
                path,
                status: None,
                duration: None,
                controller: None,
                action: None,
            }));
        }

        // Check for processing (controller#action)
        if let Some(caps) = Self::processing_pattern().captures(clean_line) {
            return Some(LogEvent::Info(format!(
                "Processing: {}#{}",
                &caps[1], &caps[2]
            )));
        }

        // Check for completed request
        if let Some(caps) = Self::completed_pattern().captures(clean_line) {
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
        if let Some(caps) = Self::sql_pattern().captures(clean_line) {
            let name = caps[1].trim().to_string();
            let duration: f64 = caps[2].parse().unwrap_or(0.0);
            // Strip Rails 7 query comments from the query text
            let query = Self::strip_query_comments(caps[0].to_string());

            return Some(LogEvent::SqlQuery(SqlQuery {
                query,
                duration: Some(duration),
                rows: None,
                name: Some(name),
            }));
        }

        // Fallback to simple SQL pattern
        if let Some(_caps) = Self::sql_simple_pattern().captures(clean_line) {
            let query = Self::strip_query_comments(clean_line.to_string());
            return Some(LogEvent::SqlQuery(SqlQuery {
                query,
                duration: None,
                rows: None,
                name: None,
            }));
        }

        // Check for generic errors
        if clean_line.contains("ERROR") || clean_line.contains("FATAL") || clean_line.contains("Exception") {
            return Some(LogEvent::Error(clean_line.to_string()));
        }

        None
    }

    /// Strip Rails 7 query comments like /*application='Blog',controller='articles'*/
    fn strip_query_comments(query: String) -> String {
        static QUERY_COMMENT: OnceLock<Regex> = OnceLock::new();
        let re = QUERY_COMMENT.get_or_init(|| {
            Regex::new(r"/\*.*?\*/").unwrap()
        });
        re.replace_all(&query, "").trim().to_string()
    }

    /// Detect specific Rails startup and runtime errors
    fn detect_rails_error(line: &str) -> Option<RailsError> {
        let line_lower = line.to_lowercase();

        // Pending migrations
        if line_lower.contains("pending migration")
            || (line_lower.contains("migrations") && line_lower.contains("pending"))
            || line_lower.contains("run `bin/rails db:migrate`")
        {
            return Some(RailsError::PendingMigrations);
        }

        // Database not found
        if (line_lower.contains("database") && line_lower.contains("does not exist"))
            || line_lower.contains("unknown database")
            || line_lower.contains("database \"") && line_lower.contains("\" does not exist")
        {
            let db_name = Self::extract_database_name(line);
            return Some(RailsError::DatabaseNotFound(db_name));
        }

        // Database connection failed
        if line_lower.contains("could not connect to server")
            || line_lower.contains("connection refused")
            || line_lower.contains("no connection")
            || (line_lower.contains("connection") && line_lower.contains("fail"))
            || line_lower.contains("activerecord::connectionnotestablished")
        {
            return Some(RailsError::DatabaseConnectionFailed(line.to_string()));
        }

        // Missing gem / bundler error
        if line_lower.contains("could not find gem")
            || line_lower.contains("gem::loaderror")
            || line_lower.contains("cannot load such file") && line.contains("gem")
        {
            let gem_name = Self::extract_gem_name(line);
            return Some(RailsError::MissingGem(gem_name));
        }

        // Bundler errors
        if line_lower.contains("bundler::gemnotfound")
            || line_lower.contains("your bundle is locked")
            || line_lower.contains("bundle install") && line_lower.contains("error")
        {
            return Some(RailsError::BundlerError(line.to_string()));
        }

        // Port already in use
        if line_lower.contains("address already in use")
            || line_lower.contains("port") && line_lower.contains("already in use")
        {
            let port = Self::extract_port_from_error(line);
            return Some(RailsError::PortInUse(port));
        }

        // Configuration errors
        if line_lower.contains("secret_key_base")
            || line_lower.contains("config")
                && (line_lower.contains("missing") || line_lower.contains("invalid"))
            || line_lower.contains("credentials") && line_lower.contains("error")
        {
            return Some(RailsError::ConfigurationError(line.to_string()));
        }

        // Generic Rails startup errors
        if (line_lower.contains("rails") || line_lower.contains("rack"))
            && (line_lower.contains("error") || line_lower.contains("failed"))
            && !line_lower.contains("test")
        {
            // Don't match test failures
            return Some(RailsError::GenericStartupError(line.to_string()));
        }

        None
    }

    fn extract_database_name(line: &str) -> String {
        // Try to extract database name from error message
        if let Some(start) = line.find("database \"") {
            let rest = &line[start + 10..];
            if let Some(end) = rest.find('"') {
                return rest[..end].to_string();
            }
        }
        if let Some(start) = line.find("database '") {
            let rest = &line[start + 10..];
            if let Some(end) = rest.find('\'') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }

    fn extract_gem_name(line: &str) -> String {
        // Try to extract gem name from error message
        if let Some(start) = line.find("gem '") {
            let rest = &line[start + 5..];
            if let Some(end) = rest.find('\'') {
                return rest[..end].to_string();
            }
        }
        if let Some(start) = line.find("gem \"") {
            let rest = &line[start + 5..];
            if let Some(end) = rest.find('"') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }

    fn extract_port_from_error(line: &str) -> u16 {
        // Try to extract port number from error message
        let words: Vec<&str> = line.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase().contains("port") && i + 1 < words.len() {
                if let Ok(port) = words[i + 1].trim_matches(|c: char| !c.is_numeric()).parse() {
                    return port;
                }
            }
        }
        3000 // Default Rails port
    }

    pub fn highlight_sql(query: &str) -> String {
        let keywords = [
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "JOIN", "LEFT", "RIGHT",
            "INNER", "OUTER", "ON", "GROUP BY", "ORDER BY", "LIMIT", "OFFSET", "AND", "OR", "NOT",
            "IN", "LIKE", "BETWEEN", "CREATE", "ALTER", "DROP", "TABLE", "INDEX", "BEGIN",
            "COMMIT", "ROLLBACK",
        ];

        let mut highlighted = query.to_string();
        for keyword in keywords {
            highlighted = highlighted.replace(keyword, &format!("[KW]{}[/KW]", keyword));
        }
        highlighted
    }
}

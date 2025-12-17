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
    fn http_start_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r#"Started (GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS) "([^"]+)""#).unwrap()
        })
    }

    fn processing_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"Processing by ([^#]+)#(\w+)").unwrap())
    }

    fn completed_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"Completed (\d+) .* in (\d+(?:\.\d+)?)ms").unwrap())
    }

    fn sql_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            // Matches Rails SQL logs like: User Load (0.5ms)  SELECT "users".* FROM "users"
            Regex::new(r"([\w\s]+)\s*\((\d+(?:\.\d+)?)ms\)\s+(SELECT|INSERT|UPDATE|DELETE).+")
                .unwrap()
        })
    }

    fn sql_simple_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"(SELECT|INSERT|UPDATE|DELETE|BEGIN|COMMIT|ROLLBACK)[^(]*").unwrap()
        })
    }

    pub fn parse_line(line: &str) -> Option<LogEvent> {
        // Check for Rails-specific startup errors first
        if let Some(rails_error) = Self::detect_rails_error(line) {
            return Some(LogEvent::RailsStartupError(rails_error));
        }

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

        // Check for generic errors
        if line.contains("ERROR") || line.contains("FATAL") || line.contains("Exception") {
            return Some(LogEvent::Error(line.to_string()));
        }

        None
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

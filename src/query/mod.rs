use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryFingerprint {
    pub normalized: String,
}

#[derive(Debug, Clone)]
pub struct QueryInfo {
    pub raw_query: String,
    pub fingerprint: QueryFingerprint,
    pub duration: f64,
    pub rows: Option<usize>,
    pub query_type: QueryType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Begin,
    Commit,
    Rollback,
    Other,
}

impl QueryType {
    pub fn from_sql(sql: &str) -> Self {
        let sql_upper = sql.trim().to_uppercase();
        if sql_upper.starts_with("SELECT") {
            QueryType::Select
        } else if sql_upper.starts_with("INSERT") {
            QueryType::Insert
        } else if sql_upper.starts_with("UPDATE") {
            QueryType::Update
        } else if sql_upper.starts_with("DELETE") {
            QueryType::Delete
        } else if sql_upper.starts_with("BEGIN") {
            QueryType::Begin
        } else if sql_upper.starts_with("COMMIT") {
            QueryType::Commit
        } else if sql_upper.starts_with("ROLLBACK") {
            QueryType::Rollback
        } else {
            QueryType::Other
        }
    }
}

impl QueryFingerprint {
    pub fn new(query: &str) -> Self {
        Self {
            normalized: Self::normalize_query(query),
        }
    }

    /// Normalize query by replacing values with placeholders
    fn normalize_query(query: &str) -> String {
        static NUMBER_PATTERN: OnceLock<Regex> = OnceLock::new();
        static STRING_PATTERN: OnceLock<Regex> = OnceLock::new();
        static PLACEHOLDER_PATTERN: OnceLock<Regex> = OnceLock::new();

        let number_re = NUMBER_PATTERN.get_or_init(|| Regex::new(r"\b\d+\b").unwrap());
        let string_re = STRING_PATTERN.get_or_init(|| Regex::new(r"'[^']*'").unwrap());
        let placeholder_re = PLACEHOLDER_PATTERN.get_or_init(|| Regex::new(r"\$\d+").unwrap());

        let mut normalized = query.to_string();

        // Replace placeholders like $1, $2
        normalized = placeholder_re.replace_all(&normalized, "?").to_string();

        // Replace string literals
        normalized = string_re.replace_all(&normalized, "?").to_string();

        // Replace numbers
        normalized = number_re.replace_all(&normalized, "?").to_string();

        // Normalize whitespace
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

        normalized
    }
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub queries: Vec<QueryInfo>,
    pub start_time: std::time::Instant,
    pub path: Option<String>,
}

impl RequestContext {
    pub fn new(path: Option<String>) -> Self {
        Self {
            queries: Vec::new(),
            start_time: std::time::Instant::now(),
            path,
        }
    }

    pub fn add_query(&mut self, query: QueryInfo) {
        self.queries.push(query);
    }

    pub fn total_query_time(&self) -> f64 {
        self.queries.iter().map(|q| q.duration).sum()
    }

    pub fn query_count(&self) -> usize {
        self.queries.len()
    }
}

#[derive(Debug, Clone)]
pub struct NPlusOneIssue {
    pub fingerprint: QueryFingerprint,
    pub count: usize,
    pub total_duration: f64,
    pub sample_query: String,
    pub suggestion: String,
}

pub struct NPlusOneDetector;

impl NPlusOneDetector {
    /// Detect N+1 queries in a request context
    pub fn detect(context: &RequestContext) -> Vec<NPlusOneIssue> {
        let mut issues = Vec::new();
        let mut fingerprint_counts: HashMap<QueryFingerprint, Vec<&QueryInfo>> = HashMap::new();

        // Group queries by fingerprint
        for query in &context.queries {
            // Only check SELECT queries
            if query.query_type == QueryType::Select {
                fingerprint_counts
                    .entry(query.fingerprint.clone())
                    .or_insert_with(Vec::new)
                    .push(query);
            }
        }

        // Find queries executed multiple times
        for (fingerprint, queries) in fingerprint_counts {
            if queries.len() > 2 {
                // N+1 pattern: same query executed multiple times
                let total_duration: f64 = queries.iter().map(|q| q.duration).sum();
                let sample_query = queries[0].raw_query.clone();

                let suggestion = Self::generate_suggestion(&sample_query, queries.len());

                issues.push(NPlusOneIssue {
                    fingerprint,
                    count: queries.len(),
                    total_duration,
                    sample_query,
                    suggestion,
                });
            }
        }

        // Sort by count (most repeated first)
        issues.sort_by(|a, b| b.count.cmp(&a.count));
        issues
    }

    fn generate_suggestion(query: &str, count: usize) -> String {
        // Try to extract table name
        static TABLE_PATTERN: OnceLock<Regex> = OnceLock::new();
        let table_re = TABLE_PATTERN.get_or_init(|| Regex::new(r#"FROM\s+"?(\w+)"?"#).unwrap());

        if let Some(caps) = table_re.captures(query) {
            let table = &caps[1];
            format!(
                "Possible N+1 query detected ({} times). Consider using eager loading:\n  \
                Model.includes(:{}) instead of lazy loading",
                count,
                table.trim_end_matches('s') // Simple singularization
            )
        } else {
            format!(
                "Possible N+1 query detected ({} times). Consider using eager loading with .includes() or .preload()",
                count
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceIssue {
    SelectStar,
    NoIndex,
    LargeResultSet,
    SlowQuery,
}

#[derive(Debug, Clone)]
pub struct QueryRecommendation {
    pub issue_type: PerformanceIssue,
    pub severity: Severity,
    pub message: String,
    pub suggestion: String,
    pub migration_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct QueryAnalyzer;

impl QueryAnalyzer {
    pub fn analyze(query: &QueryInfo) -> Vec<QueryRecommendation> {
        let mut recommendations = Vec::new();

        // Check for SELECT *
        if query.raw_query.contains("SELECT *") {
            recommendations.push(QueryRecommendation {
                issue_type: PerformanceIssue::SelectStar,
                severity: Severity::Medium,
                message: "Using SELECT * is inefficient".to_string(),
                suggestion: "Specify only the columns you need".to_string(),
                migration_code: None,
            });
        }

        // Check for slow queries
        if query.duration > 100.0 {
            recommendations.push(QueryRecommendation {
                issue_type: PerformanceIssue::SlowQuery,
                severity: if query.duration > 1000.0 {
                    Severity::Critical
                } else if query.duration > 500.0 {
                    Severity::High
                } else {
                    Severity::Medium
                },
                message: format!("Slow query: {:.1}ms", query.duration),
                suggestion: "Consider adding indexes or optimizing the query".to_string(),
                migration_code: Self::suggest_index(&query.raw_query),
            });
        }

        // Check for large result sets (if we have row count)
        if let Some(rows) = query.rows {
            if rows > 100 {
                recommendations.push(QueryRecommendation {
                    issue_type: PerformanceIssue::LargeResultSet,
                    severity: if rows > 1000 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    message: format!("Large result set: {} rows", rows),
                    suggestion: "Consider using pagination (limit/offset) or find_each".to_string(),
                    migration_code: None,
                });
            }
        }

        recommendations
    }

    fn suggest_index(query: &str) -> Option<String> {
        // Simple index suggestion based on WHERE clause
        static WHERE_PATTERN: OnceLock<Regex> = OnceLock::new();
        let where_re = WHERE_PATTERN
            .get_or_init(|| Regex::new(r#"WHERE\s+"?(\w+)"?\."?(\w+)"?\s*="#).unwrap());

        if let Some(caps) = where_re.captures(query) {
            let table = &caps[1];
            let column = &caps[2];

            Some(format!(
                "# Add to migration:\nadd_index :{}, :{}\n\n# Or generate:\nrails g migration AddIndexTo{} {}:index",
                table,
                column,
                table.chars().next()?.to_uppercase().collect::<String>() + &table[1..],
                column
            ))
        } else {
            None
        }
    }
}

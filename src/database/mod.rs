use ratatui::style::{Color, Style};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Memory management constants
const MAX_TABLES_TRACKED: usize = 100;
const TABLES_WARNING_THRESHOLD: usize = 90; // 90% of max

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub estimated_rows: usize,
    pub has_primary_key: bool,
    pub indexes: Vec<IndexInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub usage_count: usize,
}

#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub column: String,
    pub references_table: String,
    pub has_index: bool,
}

#[derive(Debug, Clone)]
pub struct SlowQuery {
    pub query: String,
    pub duration: f64,
    pub table: Option<String>,
    pub execution_count: usize,
    pub last_seen: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    MissingIndex,
    UnusedIndex,
    DuplicateIndex,
    MissingForeignKeyIndex,
    SlowQuery,
    LargeTable,
    SelectStar,
}

#[derive(Debug, Clone)]
pub struct DatabaseIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub migration_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl IssueSeverity {
    pub fn score(&self) -> u32 {
        match self {
            IssueSeverity::Low => 1,
            IssueSeverity::Medium => 5,
            IssueSeverity::High => 10,
            IssueSeverity::Critical => 20,
        }
    }
}

pub struct DatabaseHealth {
    _tables: Arc<Mutex<HashMap<String, TableInfo>>>,
    slow_queries: Arc<Mutex<Vec<SlowQuery>>>,
    query_stats: Arc<Mutex<QueryStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    pub total_queries: usize,
    pub slow_queries_count: usize,
    pub select_star_count: usize,
    pub missing_index_hints: usize,
    pub tables_accessed: HashMap<String, usize>,
}

impl DatabaseHealth {
    pub fn new() -> Self {
        Self {
            _tables: Arc::new(Mutex::new(HashMap::new())),
            slow_queries: Arc::new(Mutex::new(Vec::new())),
            query_stats: Arc::new(Mutex::new(QueryStats::default())),
        }
    }

    pub fn analyze_query(&self, query: &str, duration: f64) {
        let mut stats = self.query_stats.lock().unwrap();
        stats.total_queries += 1;

        // Track slow queries (>100ms)
        if duration > 100.0 {
            stats.slow_queries_count += 1;

            // Extract table name
            let table = Self::extract_table_name(query);

            let mut slow_queries = self.slow_queries.lock().unwrap();

            // Check if we already have this query
            if let Some(existing) = slow_queries.iter_mut().find(|sq| sq.query == query) {
                existing.execution_count += 1;
                existing.last_seen = std::time::Instant::now();
                if duration > existing.duration {
                    existing.duration = duration;
                }
            } else {
                slow_queries.push(SlowQuery {
                    query: query.to_string(),
                    duration,
                    table: table.clone(),
                    execution_count: 1,
                    last_seen: std::time::Instant::now(),
                });

                // Keep only last 50 slow queries
                if slow_queries.len() > 50 {
                    slow_queries.remove(0);
                }
            }

            // Track table access
            if let Some(table_name) = table {
                // Check if we're at capacity before adding new table
                if stats.tables_accessed.len() >= MAX_TABLES_TRACKED
                    && !stats.tables_accessed.contains_key(&table_name)
                {
                    // Log warning when at capacity
                    eprintln!(
                        "[WARN] Tables tracking at capacity ({}), evicting least accessed table",
                        MAX_TABLES_TRACKED
                    );

                    // Evict least accessed table
                    if let Some(least_accessed_table) = stats
                        .tables_accessed
                        .iter()
                        .min_by_key(|(_, count)| *count)
                        .map(|(table, _)| table.clone())
                    {
                        stats.tables_accessed.remove(&least_accessed_table);
                    }
                } else if stats.tables_accessed.len() >= TABLES_WARNING_THRESHOLD
                    && !stats.tables_accessed.contains_key(&table_name)
                {
                    // Log warning when approaching capacity
                    eprintln!(
                        "[WARN] Tables tracking approaching capacity: {}/{} ({}%)",
                        stats.tables_accessed.len(),
                        MAX_TABLES_TRACKED,
                        (stats.tables_accessed.len() * 100) / MAX_TABLES_TRACKED
                    );
                }

                *stats.tables_accessed.entry(table_name).or_insert(0) += 1;
            }
        }

        // Check for SELECT *
        if query.to_uppercase().contains("SELECT *") {
            stats.select_star_count += 1;
        }

        // Simple heuristic for missing indexes (sequential scans in WHERE clauses)
        if query.to_uppercase().contains("WHERE") && duration > 50.0 {
            stats.missing_index_hints += 1;
        }
    }

    fn extract_table_name(query: &str) -> Option<String> {
        let query_upper = query.to_uppercase();

        // Try to find table name after FROM
        if let Some(from_pos) = query_upper.find(" FROM ") {
            let after_from = &query[from_pos + 6..];
            let table = after_from
                .split_whitespace()
                .next()?
                .trim_matches('"')
                .trim_matches('`')
                .to_string();
            return Some(table);
        }

        // Try UPDATE
        if let Some(update_pos) = query_upper.find("UPDATE ") {
            let after_update = &query[update_pos + 7..];
            let table = after_update
                .split_whitespace()
                .next()?
                .trim_matches('"')
                .trim_matches('`')
                .to_string();
            return Some(table);
        }

        // Try INSERT INTO
        if let Some(insert_pos) = query_upper.find("INSERT INTO ") {
            let after_insert = &query[insert_pos + 12..];
            let table = after_insert
                .split_whitespace()
                .next()?
                .trim_matches('"')
                .trim_matches('`')
                .to_string();
            return Some(table);
        }

        None
    }

    pub fn get_issues(&self) -> Vec<DatabaseIssue> {
        let mut issues = Vec::new();
        let stats = self.query_stats.lock().unwrap();
        let slow_queries = self.slow_queries.lock().unwrap();

        // Issue: High slow query count
        if stats.slow_queries_count > 10 {
            let severity = if stats.slow_queries_count > 50 {
                IssueSeverity::Critical
            } else if stats.slow_queries_count > 25 {
                IssueSeverity::High
            } else {
                IssueSeverity::Medium
            };

            issues.push(DatabaseIssue {
                issue_type: IssueType::SlowQuery,
                severity,
                title: format!("{} slow queries detected", stats.slow_queries_count),
                description: format!(
                    "Queries taking >100ms have been detected {} times. This indicates potential performance issues.",
                    stats.slow_queries_count
                ),
                recommendation: "Review slow queries and consider adding indexes or optimizing query logic.".to_string(),
                migration_code: None,
            });
        }

        // Issue: SELECT * usage
        if stats.select_star_count > 5 {
            issues.push(DatabaseIssue {
                issue_type: IssueType::SelectStar,
                severity: IssueSeverity::Medium,
                title: format!("{} queries using SELECT *", stats.select_star_count),
                description: "SELECT * queries fetch all columns, which can be inefficient."
                    .to_string(),
                recommendation: "Specify only the columns you need in SELECT queries.".to_string(),
                migration_code: None,
            });
        }

        // Issue: Potential missing indexes
        if stats.missing_index_hints > 5 {
            issues.push(DatabaseIssue {
                issue_type: IssueType::MissingIndex,
                severity: IssueSeverity::High,
                title: format!("{} queries may benefit from indexes", stats.missing_index_hints),
                description: "Slow queries with WHERE clauses detected. Adding indexes may improve performance.".to_string(),
                recommendation: "Analyze slow queries and add indexes on frequently filtered columns.".to_string(),
                migration_code: Some("# Review slow queries to determine appropriate indexes\n# rails g migration AddIndexToTable column:index".to_string()),
            });
        }

        // Analyze individual slow queries
        for sq in slow_queries.iter().take(5) {
            if sq.duration > 500.0 {
                let table_hint = sq
                    .table
                    .as_ref()
                    .map_or(String::new(), |t| format!(" on table '{}'", t));

                issues.push(DatabaseIssue {
                    issue_type: IssueType::SlowQuery,
                    severity: if sq.duration > 1000.0 {
                        IssueSeverity::Critical
                    } else {
                        IssueSeverity::High
                    },
                    title: format!("Very slow query{}: {:.1}ms", table_hint, sq.duration),
                    description: sq.query[..sq.query.len().min(100)].to_string(),
                    recommendation: format!(
                        "This query has been executed {} times. Consider optimization or caching.",
                        sq.execution_count
                    ),
                    migration_code: None,
                });
            }
        }

        // Sort by severity
        issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        issues
    }

    pub fn calculate_health_score(&self) -> u32 {
        let issues = self.get_issues();
        let stats = self.query_stats.lock().unwrap();

        // Start with perfect score
        let mut score = 100u32;

        // Deduct points for issues
        for issue in &issues {
            score = score.saturating_sub(issue.severity.score());
        }

        // Bonus points for good practices
        if stats.total_queries > 0 {
            let select_star_ratio =
                (stats.select_star_count as f64 / stats.total_queries as f64) * 100.0;
            if select_star_ratio < 5.0 {
                score = (score + 5).min(100);
            }

            let slow_query_ratio =
                (stats.slow_queries_count as f64 / stats.total_queries as f64) * 100.0;
            if slow_query_ratio < 2.0 {
                score = (score + 10).min(100);
            }
        }

        score
    }

    pub fn get_stats(&self) -> QueryStats {
        self.query_stats.lock().unwrap().clone()
    }

    pub fn get_slow_queries(&self) -> Vec<SlowQuery> {
        let mut queries = self.slow_queries.lock().unwrap().clone();
        queries.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
        queries
    }

    pub fn get_top_tables(&self) -> Vec<(String, usize)> {
        let stats = self.query_stats.lock().unwrap();
        let mut tables: Vec<_> = stats
            .tables_accessed
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        tables.sort_by(|a, b| b.1.cmp(&a.1));
        tables.into_iter().take(10).collect()
    }

    pub fn get_health_style(&self) -> Style {
        let score = self.calculate_health_score();
        let color = match score {
            90..=100 => Color::Green,
            70..=89 => Color::Yellow,
            50..=69 => Color::Rgb(255, 165, 0), // Orange
            _ => Color::Red,
        };
        Style::default().fg(color)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlan {
    pub raw_output: String,
    pub formatted: String,
    pub warnings: Vec<ExplainWarning>,
    pub cost: Option<f64>,
    pub rows: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainWarning {
    pub severity: WarningSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Critical,
}

pub struct ExplainExecutor {
    _database_url: Option<String>,
}

impl ExplainExecutor {
    pub fn new(database_url: Option<String>) -> Self {
        Self {
            _database_url: database_url,
        }
    }

    pub fn explain_query(&self, query: &str) -> Result<ExplainPlan, String> {
        // For now, this is a placeholder that would connect to the database
        // In a real implementation, we'd use a database connection pool

        // Simulate EXPLAIN output for demonstration
        Ok(self.simulate_explain(query))
    }

    fn simulate_explain(&self, _query: &str) -> ExplainPlan {
        let raw_output = format!(
            "Seq Scan on users  (cost=0.00..15.00 rows=500 width=32)\n  \
            Filter: (active = true)"
        );

        let formatted = self.format_explain(&raw_output);
        let warnings = self.analyze_plan(&raw_output);

        ExplainPlan {
            raw_output: raw_output.clone(),
            formatted,
            warnings,
            cost: Some(15.0),
            rows: Some(500),
        }
    }

    fn format_explain(&self, raw: &str) -> String {
        // Add indentation and formatting
        raw.lines()
            .map(|line| {
                let indent_count = line.chars().take_while(|c| c.is_whitespace()).count();
                format!("{}{}", "  ".repeat(indent_count / 2), line.trim())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn analyze_plan(&self, plan: &str) -> Vec<ExplainWarning> {
        let mut warnings = Vec::new();

        // Check for sequential scans
        if plan.contains("Seq Scan") {
            warnings.push(ExplainWarning {
                severity: WarningSeverity::Warning,
                message: "Sequential scan detected - consider adding an index".to_string(),
            });
        }

        // Check for high cost
        if let Some(cost) = self.extract_cost(plan) {
            if cost > 1000.0 {
                warnings.push(ExplainWarning {
                    severity: WarningSeverity::Critical,
                    message: format!("Very high query cost: {:.2}", cost),
                });
            } else if cost > 100.0 {
                warnings.push(ExplainWarning {
                    severity: WarningSeverity::Warning,
                    message: format!("High query cost: {:.2}", cost),
                });
            }
        }

        // Check for large row estimates
        if let Some(rows) = self.extract_rows(plan) {
            if rows > 10000 {
                warnings.push(ExplainWarning {
                    severity: WarningSeverity::Warning,
                    message: format!("Large result set estimated: {} rows", rows),
                });
            }
        }

        warnings
    }

    fn extract_cost(&self, plan: &str) -> Option<f64> {
        // Parse cost from PostgreSQL EXPLAIN output
        // Format: cost=0.00..15.00
        plan.find("cost=").and_then(|start| {
            let after_cost = &plan[start + 5..];
            after_cost.find("..").and_then(|end_pos| {
                after_cost[end_pos + 2..]
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
            })
        })
    }

    fn extract_rows(&self, plan: &str) -> Option<usize> {
        // Parse rows from EXPLAIN output
        // Format: rows=500
        plan.find("rows=").and_then(|start| {
            let after_rows = &plan[start + 5..];
            after_rows
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<usize>().ok())
        })
    }
}

impl ExplainPlan {
    pub fn has_index_scan(&self) -> bool {
        self.raw_output.contains("Index Scan")
            || self.raw_output.contains("Index Only Scan")
            || self.raw_output.contains("Bitmap Index Scan")
    }

    pub fn has_seq_scan(&self) -> bool {
        self.raw_output.contains("Seq Scan")
    }

    pub fn suggest_indexes(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.has_seq_scan() {
            suggestions.push("Consider adding an index to avoid sequential scans".to_string());
        }

        if let Some(cost) = self.cost {
            if cost > 100.0 && !self.has_index_scan() {
                suggestions.push(
                    "High cost without index usage - investigate index opportunities".to_string(),
                );
            }
        }

        suggestions
    }
}

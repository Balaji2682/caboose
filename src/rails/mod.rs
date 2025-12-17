use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RailsApp {
    pub detected: bool,
    pub database: Option<String>,
    pub background_job: Option<String>,
    pub asset_pipeline: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RailsHealthIssue {
    PendingMigrations(Vec<String>),
    DatabaseNotCreated,
    DatabaseConnectionError(String),
    BundleOutdated(String),
}

impl RailsApp {
    pub fn detect() -> Self {
        Self::detect_in_path(".")
    }

    pub fn detect_in_path<P: AsRef<Path>>(root: P) -> Self {
        let mut app = RailsApp {
            detected: false,
            database: None,
            background_job: None,
            asset_pipeline: None,
        };

        let root = root.as_ref();

        // Check if it's a Rails app
        if !root.join("Gemfile").exists() || !root.join("config/application.rb").exists() {
            return app;
        }

        app.detected = true;

        // Detect database
        if let Ok(database_yml) = fs::read_to_string(root.join("config/database.yml")) {
            if database_yml.contains("postgresql") || database_yml.contains("adapter: postgresql") {
                app.database = Some("postgresql".to_string());
            } else if database_yml.contains("mysql") {
                app.database = Some("mysql".to_string());
            } else if database_yml.contains("sqlite") {
                app.database = Some("sqlite".to_string());
            }
        }

        // Detect background job framework
        if let Ok(gemfile) = fs::read_to_string(root.join("Gemfile")) {
            if gemfile.contains("gem \"sidekiq\"") || gemfile.contains("gem 'sidekiq'") {
                app.background_job = Some("sidekiq".to_string());
            } else if gemfile.contains("gem \"good_job\"") || gemfile.contains("gem 'good_job'") {
                app.background_job = Some("good_job".to_string());
            } else if gemfile.contains("gem \"solid_queue\"")
                || gemfile.contains("gem 'solid_queue'")
            {
                app.background_job = Some("solid_queue".to_string());
            }
        }

        // Detect asset pipeline
        if let Ok(gemfile) = fs::read_to_string(root.join("Gemfile")) {
            if gemfile.contains("gem \"vite_rails\"") || gemfile.contains("gem 'vite_rails'") {
                app.asset_pipeline = Some("vite".to_string());
            } else if gemfile.contains("gem \"propshaft\"") || gemfile.contains("gem 'propshaft'") {
                app.asset_pipeline = Some("propshaft".to_string());
            } else if gemfile.contains("gem \"sprockets\"") || gemfile.contains("gem 'sprockets'") {
                app.asset_pipeline = Some("sprockets".to_string());
            }
        }

        app
    }

    pub fn generate_procfile(&self, port_override: Option<u16>) -> String {
        let mut procfile = String::new();

        // Web server with configurable port
        let port = port_override.unwrap_or(3000);
        procfile.push_str(&format!("web: bundle exec rails server -p {}\n", port));

        // Background job worker
        if let Some(ref job_framework) = self.background_job {
            match job_framework.as_str() {
                "sidekiq" => procfile.push_str("worker: bundle exec sidekiq\n"),
                "good_job" => procfile.push_str("worker: bundle exec good_job start\n"),
                "solid_queue" => procfile.push_str("worker: bundle exec rake solid_queue:start\n"),
                _ => {}
            }
        }

        // Asset pipeline
        if let Some(ref asset_pipeline) = self.asset_pipeline {
            if asset_pipeline == "vite" {
                procfile.push_str("vite: bin/vite dev\n");
            }
        }

        procfile
    }

    /// Check for Rails health issues (pending migrations, database connectivity)
    pub fn check_health(&self) -> Vec<RailsHealthIssue> {
        if !self.detected {
            return vec![];
        }

        let mut issues = vec![];

        // Check if bundle install is needed
        if let Ok(output) = Command::new("bundle").args(["check"]).output() {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let message = if !stderr.is_empty() {
                    stderr.to_string()
                } else if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    "Gemfile dependencies are not satisfied".to_string()
                };
                issues.push(RailsHealthIssue::BundleOutdated(message));
                // If bundle check fails, skip other checks as they'll likely fail too
                return issues;
            }
        }

        // Check for pending migrations
        if let Ok(output) = Command::new("bundle")
            .args(["exec", "rails", "db:migrate:status"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let pending: Vec<String> = stdout
                    .lines()
                    .filter(|line| line.contains("down"))
                    .map(|line| line.trim().to_string())
                    .collect();

                if !pending.is_empty() {
                    issues.push(RailsHealthIssue::PendingMigrations(pending));
                }
            } else {
                // Check if database doesn't exist
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("database") && stderr.contains("does not exist") {
                    issues.push(RailsHealthIssue::DatabaseNotCreated);
                } else if stderr.contains("could not connect") || stderr.contains("connection") {
                    issues.push(RailsHealthIssue::DatabaseConnectionError(
                        stderr.lines().next().unwrap_or("Unknown error").to_string(),
                    ));
                }
            }
        }

        issues
    }
}

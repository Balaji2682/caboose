use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RailsApp {
    pub detected: bool,
    pub database: Option<String>,
    pub background_job: Option<String>,
    pub asset_pipeline: Option<String>,
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
            } else if gemfile.contains("gem \"solid_queue\"") || gemfile.contains("gem 'solid_queue'") {
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
}

//! # Caboose: Rails + Frontend Development Companion
//!
//! Caboose is a terminal-first toolkit for running Rails backends and modern
//! frontends together with rich observability. It autogenerates Procfiles when
//! possible, streams colored logs, highlights Rails requests, flags N+1 queries,
//! scores database health, tracks tests/debuggers, and groups exceptions – all
//! inside a single Ratatui interface.
//!
//! ## Installation
//! - Prerequisites: Rust toolchain (Cargo) and your Rails/frontend dependencies.
//! - Build release binary: `cargo build --release`
//! - Run in debug: `cargo run`
//!
//! ## Project Setup
//! - Run Caboose from the Rails project root. It auto-detects Rails, database,
//!   background jobs, asset pipeline, Git status, and a colocated or sibling
//!   frontend (Vite, Next.js, Nuxt, SvelteKit, Remix, Astro, CRA, Vue CLI,
//!   Angular).
//! - Works for monorepos (`frontend/`, `client/`, `web/`, `app/`, `ui/`, `www/`)
//!   and sibling layouts (`../frontend`, `../client`, `../web`).
//! - If no Procfile is present, Caboose generates one with Rails, worker, and
//!   frontend entries using the detected package manager.
//!
//! ## Usage
//! ```bash
//! # From Rails project root
//! cargo run
//! ./target/release/caboose
//! ```
//! - Coming soon CLI shims: `caboose dev [process]`, `caboose stop`, `caboose restart`, `caboose logs`, `caboose ps`.
//! - Keyboard inside the TUI: `q`/`Esc` quit, `t` cycles views, `/` search,
//!   `↑/↓` scroll, `PageUp/PageDown` page scroll, `c` clear filters, `e` export logs.
//!
//! ## Configuration Priority
//! 1) **Procfile** – full manual control.
//! 2) **`.caboose.toml`** – persistent, ergonomic config (recommended).
//! 3) **Auto-detection** – zero-config defaults.
//!
//! ## `.caboose.toml` Reference (key fields)
//! ```toml
//! [frontend]
//! path = "frontend"              # Explicit path; supports sibling paths ("../web")
//! disable_auto_detect = false    # Turn off scanning when multiple package.json files exist
//! dev_command = "npm run dev"    # Override framework default and add flags (--host, --port, etc.)
//! port = 5173                    # Override default dev port
//! process_name = "ui"            # Name shown in logs/headers
//!
//! [rails]
//! port = 3000                    # Rails server port override
//! disable_auto_detect = false
//!
//! [processes.web]                # Per-process overrides
//! command = "bundle exec puma -p 4000"
//! env = { RAILS_ENV = "development", RAILS_LOG_LEVEL = "debug" }
//!
//! [processes.frontend]
//! command = "cd client && pnpm dev -- --port 3001"
//! env = { NODE_ENV = "development", VITE_API_URL = "http://localhost:4000" }
//! ```
//! - Use `.caboose.toml` to set non-standard frontend locations, custom package
//!   managers (pnpm/bun/yarn), custom ports, and custom process names.
//! - Disable auto-detect when multiple frontends exist, then drive everything via
//!   Procfile entries.
//! - Keep `.env` files out of VCS; commit `.caboose.toml` for team consistency.
//!
//! ## Procfile Examples
//! ```procfile
//! web: bundle exec rails server -p 3000
//! worker: bundle exec sidekiq
//! frontend: cd frontend && npm run dev
//! vite: bin/vite dev
//! ```
//! - For multi-frontend setups, disable auto-detect and enumerate each entry in
//!   Procfile (e.g., `admin`, `customer`).
//!
//! ## Environment Variables
//! - Rails: set in `.env` (DATABASE_URL, REDIS_URL, SECRET_KEY_BASE, RAILS_ENV,
//!   RAILS_LOG_LEVEL, service keys).
//! - Frontend: set in `frontend/.env` or framework-specific files
//!   (`VITE_API_URL`, `NEXT_PUBLIC_API_URL`, etc.).
//!
//! ## Features at a Glance
//! - **Process management:** PTY-backed Procfile runner, start/stop lifecycle,
//!   health and status display, colorized logs.
//! - **Rails integration:** Auto-detect Rails, DB, background jobs, asset
//!   pipeline; auto-generate Procfiles when absent.
//! - **Query analysis:** Fingerprints SQL, detects N+1 patterns, request
//!   scoping, EXPLAIN framework, query recommendations.
//! - **Advanced UI:** Live search, filtering, paging scroll, request selection,
//!   detailed request view with query timelines, log export, dynamic footer.
//! - **Database health:** Health score (0-100), slow query tracking, SELECT *
//!   warnings, missing-index hints, table stats, prioritized issues.
//! - **Testing & debugger:** Auto-detect RSpec/Minitest/Test::Unit, live test
//!   parsing, success metrics, slow test list, debugger (Pry/Byebug/Debug)
//!   detection.
//! - **Exception tracking:** Auto-detects Ruby/Rails exceptions, groups by
//!   fingerprint, classifies severity, shows file:line and recency stats.
//! - **Multi-project:** Detects modern frontend frameworks, package managers,
//!   and monorepo/sibling layouts with zero manual wiring.
//!
//! ## View Cycle (press `t` to advance)
//! Logs → Query Analysis → Database Health → Test Results → Exception Tracking
//! → back to Logs. Press `Enter` in Query Analysis to open Request Details.
//!
//! ## Common Configuration Scenarios
//! - **Non-standard frontend dir:** `path = "apps/web"` in `[frontend]`.
//! - **Custom ports:** `[rails] port = 4000` and `[frontend] port = 3001`.
//! - **Custom package manager:** `dev_command = "pnpm dev"` or `bun dev`.
//! - **Sibling frontend:** `path = "../web"` so Procfile auto-generation uses it.
//! - **Multiple frontends:** `disable_auto_detect = true` and define Procfile
//!   entries (`admin`, `customer`) with custom ports/commands.
//! - **Custom flags:** `dev_command = "npm run dev -- --host --open --port 5173"`.
//!
//! ## Module Guide
//! - `cli` – Clap-based argument parsing and future `caboose dev/stop/logs/ps`
//!   command definitions.
//! - `config` – `.caboose.toml` loading, Procfile parsing/generation helpers, and
//!   `.env` ingestion.
//! - `process` – PTY-backed process spawning, environment merging, lifecycle
//!   management, and log channel fan-out (`LogLine`).
//! - `parser` – Rails log parsing (HTTP requests, SQL statements), color coding,
//!   and extraction into structured events.
//! - `query` – SQL fingerprinting, N+1 detection, query recommendations, and
//!   EXPLAIN scaffolding.
//! - `context` – Request-scoped aggregation of queries and metadata for
//!   downstream analysis.
//! - `database` – Health scoring engine, slow query tracking, issue generation,
//!   table-level stats.
//! - `stats` – Cross-cutting performance counters for header metrics.
//! - `test` – Test framework detection, live result tracking, slow test ledger,
//!   debugger detection and status.
//! - `exception` – Exception detection, fingerprinting, severity classification,
//!   grouping, and recent exception store.
//! - `frontend` – Frontend framework and package manager detection plus Procfile
//!   entry generation.
//! - `rails` – Rails project detection and Procfile scaffolding for web/worker
//!   processes with DB/background-job hints.
//! - `git` – Branch name, dirty indicator, ahead/behind counts for the UI header.
//! - `ui` – Ratatui views, components, theming, keyboard handling, and navigation
//!   among Logs/Query Analysis/Database Health/Test Results/Exceptions.
//! - `explain` – EXPLAIN plan data structures and warnings (ready for DB
//!   integration).
//! - `main.rs` (this file) – binary entrypoint wiring all modules together.
//!
//! ## Observability Dashboards
//! - **Logs view:** real-time logs, process filter, search, paging, export.
//! - **Query Analysis:** request list with query counts, N+1 warnings, toggle to
//!   Request Details for per-query timelines with duration coloring.
//! - **Database Health:** health score, slowest queries, issue list with
//!   severity, SELECT */index hints, table stats.
//! - **Test Results:** live run progress, recent history, slow tests, debugger
//!   status.
//! - **Exception Tracking:** grouped exceptions with counts, severity icons,
//!   recent occurrences, and file:line when present.
//!
//! ## Troubleshooting
//! - Frontend not detected → set `[frontend].path` explicitly or add
//!   `package.json`/framework config; disable auto-detect if multiple frontends.
//! - Wrong package manager used → ensure the correct lockfile exists (`yarn.lock`
//!   vs `package-lock.json` vs `pnpm-lock.yaml` vs `bun.lockb`).
//! - Port conflict → set `[rails].port` and `[frontend].port` or override in
//!   Procfile commands.
//! - Need custom commands → override in `[processes.<name>]` or in Procfile.
//!
//! ## Development & Testing
//! - Build: `cargo build`
//! - Run against simulated Rails logs: `./test-rails-logs.sh` then `cargo run`
//!   and exercise the UI.
//! - The UI refactor is modularized (see `src/ui/*`) with reusable widgets,
//!   theming, and formatting utilities to ease further contributions.
use caboose::cli::{Cli, Commands};
use caboose::config::{CabooseConfig, Procfile, load_env};
use caboose::context::RequestContextTracker;
use caboose::database::DatabaseHealth;
use caboose::exception::ExceptionTracker;
use caboose::frontend::{FrontendApp, PackageManager};
use caboose::git::GitInfo;
use caboose::process::{LogLine, ProcessManager};
use caboose::rails::RailsApp;
use caboose::stats::StatsCollector;
use caboose::test::TestTracker;
use caboose::ui::{self, App};
use clap::Parser;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Dev { process: _ }) | None => {
            run_dev_mode().await?;
        }
        Some(Commands::Stop) => {
            println!("Stop command not yet implemented");
        }
        Some(Commands::Restart { process }) => {
            println!("Restart '{}' not yet implemented", process);
        }
        Some(Commands::Logs { process }) => {
            println!("Logs for '{}' not yet implemented", process);
        }
        Some(Commands::Ps) => {
            println!("Ps command not yet implemented");
        }
    }

    Ok(())
}

async fn run_dev_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let caboose_config = CabooseConfig::load();

    // Detect Rails application
    let rails_app = if caboose_config.rails.disable_auto_detect {
        RailsApp { detected: false, database: None, background_job: None, asset_pipeline: None }
    } else {
        RailsApp::detect()
    };

    if rails_app.detected {
        println!("✓ Rails application detected");
        if let Some(ref db) = rails_app.database {
            println!("  Database: {}", db);
        }
        if let Some(ref job) = rails_app.background_job {
            println!("  Background jobs: {}", job);
        }
        if let Some(ref assets) = rails_app.asset_pipeline {
            println!("  Assets: {}", assets);
        }
    }

    // Detect Frontend application
    let frontend_app = if caboose_config.frontend.disable_auto_detect {
        FrontendApp { detected: false, framework: None, path: String::new(), package_manager: PackageManager::Npm }
    } else if let Some(ref path) = caboose_config.frontend.path {
        println!("Using configured frontend path: {}", path);
        FrontendApp::detect_with_config(Some(path))
    } else {
        FrontendApp::detect()
    };

    if frontend_app.detected {
        println!("✓ Frontend application detected");
        if let Some(ref framework) = frontend_app.framework {
            println!("  Framework: {}", framework.name());
            println!("  Path: {}", frontend_app.path);
            println!("  Package manager: {:?}", frontend_app.package_manager);
        }
    }

    // Load or generate Procfile
    let procfile = if std::path::Path::new("Procfile").exists() {
        println!("Loading Procfile...");
        Procfile::parse("Procfile")
            .map_err(|e| format!("Failed to load Procfile: {}", e))?
    } else if rails_app.detected || frontend_app.detected {
        println!("No Procfile found, auto-generating...");
        let procfile_content = generate_multi_project_procfile(&rails_app, &frontend_app);
        println!("{}", procfile_content);
        Procfile::parse_content(&procfile_content)?
    } else {
        return Err("No Procfile found and no Rails/Frontend app detected. Create a Procfile to continue.".into());
    };

    println!("Starting {} processes", procfile.processes.len());

    // Load .env
    let env_vars = load_env(".env").unwrap_or_default();
    if !env_vars.is_empty() {
        println!("Loaded {} environment variables", env_vars.len());
    }

    // Get Git info
    let git_info = GitInfo::get();

    // Create stats collector
    let stats_collector = StatsCollector::new();

    // Create request context tracker
    let context_tracker = Arc::new(RequestContextTracker::new());

    // Create database health tracker
    let db_health = Arc::new(DatabaseHealth::new());

    // Create test tracker
    let test_tracker = Arc::new(TestTracker::new());

    // Create exception tracker
    let exception_tracker = Arc::new(ExceptionTracker::new());

    // Create log channel
    let (log_tx, log_rx) = mpsc::unbounded_channel::<LogLine>();

    // Create process manager
    let process_manager = Arc::new(ProcessManager::new(log_tx));
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    // Handle Ctrl+C to trigger graceful shutdown
    {
        let process_manager = process_manager.clone();
        let shutdown_flag = shutdown_flag.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            shutdown_flag.store(true, Ordering::SeqCst);
            process_manager.stop_all();
        });
    }

    // Spawn processes
    for proc_config in procfile.processes {
        println!("  → Starting: {}", proc_config.name);
        process_manager.spawn_process(
            proc_config.name.clone(),
            proc_config.command.clone(),
            env_vars.clone(),
        )?;
    }

    // Wait a bit for processes to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Run TUI
    let app = App::new(
        git_info,
        stats_collector.clone(),
        context_tracker.clone(),
        db_health.clone(),
        test_tracker.clone(),
        exception_tracker.clone(),
    );
    let process_manager_for_ui = process_manager.clone();
    let ui_result = ui::run_ui(
        app,
        log_rx,
        process_manager_for_ui,
        stats_collector,
        context_tracker,
        db_health,
        test_tracker,
        exception_tracker,
        shutdown_flag.clone(),
    )
    .await;

    // Ensure all child processes are torn down when leaving the UI
    process_manager.stop_all();

    // Propagate any UI errors after cleanup
    ui_result?;

    Ok(())
}

fn generate_multi_project_procfile(rails_app: &RailsApp, frontend_app: &FrontendApp) -> String {
    let mut procfile_content = String::new();

    // Add Rails processes if detected
    if rails_app.detected {
        procfile_content.push_str(&rails_app.generate_procfile());
    }

    // Add frontend process if detected
    if frontend_app.detected {
        if let Some(frontend_entry) = frontend_app.generate_procfile_entry() {
            if !procfile_content.is_empty() {
                procfile_content.push('\n');
            }
            procfile_content.push_str(&format!("frontend: {}", frontend_entry));
        }
    }

    procfile_content
}

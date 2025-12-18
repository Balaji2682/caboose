pub mod command;
pub mod components;
pub mod formatting;
pub mod icon_manager;
/// UI Module - Terminal User Interface
///
/// This module provides a modular, professional-grade terminal UI framework
/// following DRY principles and clean architecture patterns.
// Public modules
pub mod theme;
pub mod themes;
pub mod views;
pub mod widgets;

// Re-exports for convenience
pub use formatting::*;
pub use theme::Theme;

use crate::context::RequestContextTracker;
use crate::database::DatabaseHealth;
use crate::exception::ExceptionTracker;
use crate::git::GitInfo;
use crate::parser::{LogEvent, RailsLogParser};
use crate::process::{LogLine, ProcessInfo};
use crate::stats::StatsCollector;
use crate::test::TestTracker;
use crate::ui::components::FooterBuilder;
use crate::ui::theme::Icons;
use crate::ui::widgets::Sparkline; // Import Sparkline

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant}; // Import Instant
use tokio::sync::mpsc;

// ============================================================================
// VIEW MODE
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Logs,
    QueryAnalysis,
    RequestDetail(usize),
    DatabaseHealth,
    TestResults,
    Exceptions,
    ExceptionDetail(usize),
}

impl ViewMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::Logs => "Logs",
            ViewMode::QueryAnalysis => "Query Analysis",
            ViewMode::RequestDetail(_) => "Request Detail",
            ViewMode::DatabaseHealth => "Database Health",
            ViewMode::TestResults => "Test Results",
            ViewMode::Exceptions => "Exceptions",
            ViewMode::ExceptionDetail(_) => "Exception Detail",
        }
    }

    pub fn all_variants() -> Vec<ViewMode> {
        vec![
            ViewMode::Logs,
            ViewMode::QueryAnalysis,
            ViewMode::DatabaseHealth,
            ViewMode::TestResults,
            ViewMode::Exceptions,
        ]
    }

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(ViewMode::Logs),
            1 => Some(ViewMode::QueryAnalysis),
            2 => Some(ViewMode::DatabaseHealth),
            3 => Some(ViewMode::TestResults),
            4 => Some(ViewMode::Exceptions),
            _ => None,
        }
    }
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

/// Main application state
pub struct App {
    // Process and log data
    processes: Vec<ProcessInfo>,
    logs: Vec<LogLine>,
    max_logs: usize,

    // Application state
    should_quit: bool,
    view_mode: ViewMode,
    active_tab_index: usize,

    // Data trackers
    _git_info: GitInfo,
    environment_info: crate::environment::EnvironmentInfo,
    stats_collector: StatsCollector,
    context_tracker: std::sync::Arc<RequestContextTracker>,
    db_health: std::sync::Arc<DatabaseHealth>,
    test_tracker: std::sync::Arc<TestTracker>,
    exception_tracker: std::sync::Arc<ExceptionTracker>,

    // UI state
    search_mode: bool,
    search_query: String,
    log_scroll: usize,
    horizontal_scroll: usize,
    auto_scroll: bool,
    _request_scroll: usize,
    selected_request: usize,
    selected_exception: usize,
    filter_process: Option<String>,

    // Command system
    command_mode: bool,
    command_input: String,
    command_registry: command::CommandRegistry,
    command_autocomplete: command::AutocompleteEngine,
    command_history: command::CommandHistory,
    command_suggestions: Vec<command::autocomplete::Suggestion>,
    selected_suggestion: usize,
    last_command_result: Option<command::ExecutionResult>,

    // Animation state
    spinner_frame: usize,

    // View transition state
    previous_view_mode: Option<ViewMode>,
    last_view_change_time: Option<Instant>,
}

impl App {
    /// Create a new application instance
    pub fn new(
        git_info: GitInfo,
        stats_collector: StatsCollector,
        context_tracker: std::sync::Arc<RequestContextTracker>,
        db_health: std::sync::Arc<DatabaseHealth>,
        test_tracker: std::sync::Arc<TestTracker>,
        exception_tracker: std::sync::Arc<ExceptionTracker>,
    ) -> Self {
        // Build command registry
        let command_registry = command::commands::build_command_registry();
        let command_metadata = command_registry.get_metadata().to_vec();
        let command_autocomplete = command::AutocompleteEngine::new(command_metadata);

        Self {
            processes: Vec::new(),
            logs: Vec::new(),
            max_logs: 1000,
            should_quit: false,
            _git_info: git_info,
            environment_info: crate::environment::EnvironmentInfo::detect(),
            stats_collector,
            context_tracker,
            db_health,
            test_tracker,
            exception_tracker,
            view_mode: ViewMode::Logs,
            active_tab_index: 0,
            search_mode: false,
            search_query: String::new(),
            log_scroll: 0,
            horizontal_scroll: 0,
            auto_scroll: true,
            _request_scroll: 0,
            selected_request: 0,
            selected_exception: 0,
            filter_process: None,
            command_mode: false,
            command_input: String::new(),
            command_registry,
            command_autocomplete,
            command_history: command::CommandHistory::new(100),
            command_suggestions: Vec::new(),
            selected_suggestion: 0,
            last_command_result: None,
            spinner_frame: 0,
            previous_view_mode: None,
            last_view_change_time: None,
        }
    }

    // ========================================================================
    // LOG MANAGEMENT
    // ========================================================================

    /// Add a log line and update trackers
    pub fn add_log(&mut self, log: LogLine) {
        // Parse log for stats and context tracking
        if let Some(event) = RailsLogParser::parse_line(&log.content) {
            match &event {
                LogEvent::HttpRequest(req) => {
                    if let (Some(status), Some(duration)) = (req.status, req.duration) {
                        self.stats_collector.record_request(status, duration);
                    }
                }
                LogEvent::SqlQuery(query) => {
                    if let Some(duration) = query.duration {
                        self.stats_collector.record_sql_query(duration);
                        self.db_health.analyze_query(&query.query, duration);
                    }
                }
                LogEvent::RailsStartupError(rails_error) => {
                    // Handle Rails errors - they're already logged, no additional action needed here
                    // The error will appear in the logs view with appropriate highlighting
                    use crate::parser::RailsError;
                    match rails_error {
                        RailsError::PendingMigrations => {
                            // Could potentially auto-trigger migration dialog in future
                        }
                        RailsError::DatabaseNotFound(_) => {
                            // Could show "Run db:create" suggestion
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            self.context_tracker.process_log_event(&event);
        }

        // Feed to test tracker
        self.test_tracker.parse_line(&log.content);

        // Feed to exception tracker
        self.exception_tracker.parse_line(&log.content);

        self.logs.push(log);
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
            // If we removed a log and scroll is out of bounds, adjust it
            if !self.auto_scroll && self.log_scroll > 0 {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
        }
    }

    // ========================================================================
    // VIEW MANAGEMENT
    // ========================================================================

    /// Toggle to next view
    pub fn toggle_view(&mut self) {
        let variants = ViewMode::all_variants();
        let current_index = self.active_tab_index;
        let next_index = (current_index + 1) % variants.len();

        // Record previous view and time for transition
        self.previous_view_mode = Some(self.view_mode.clone());
        self.last_view_change_time = Some(Instant::now());

        self.view_mode = ViewMode::from_index(next_index).unwrap_or(ViewMode::Logs);
        self.active_tab_index = next_index;
    }

    /// Toggle to previous view (backward cycling)
    pub fn toggle_view_backward(&mut self) {
        let variants = ViewMode::all_variants();
        let current_index = self.active_tab_index;
        let prev_index = if current_index == 0 {
            variants.len() - 1
        } else {
            current_index - 1
        };

        // Record previous view and time for transition
        self.previous_view_mode = Some(self.view_mode.clone());
        self.last_view_change_time = Some(Instant::now());

        self.view_mode = ViewMode::from_index(prev_index).unwrap_or(ViewMode::Logs);
        self.active_tab_index = prev_index;
    }

    // ========================================================================
    // SEARCH MODE
    // ========================================================================

    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn remove_search_char(&mut self) {
        self.search_query.pop();
    }

    // ========================================================================
    // COMMAND MODE
    // ========================================================================

    pub fn enter_command_mode(&mut self) {
        self.command_mode = true;
        self.command_input = "/".to_string(); // Start with / prefix
        self.command_suggestions.clear();
        self.selected_suggestion = 0;
        self.last_command_result = None;
        self.update_command_suggestions(); // Show all commands initially
    }

    pub fn exit_command_mode(&mut self) {
        self.command_mode = false;
        self.command_input.clear();
        self.command_suggestions.clear();
        self.selected_suggestion = 0;
    }

    pub fn add_command_char(&mut self, c: char) {
        self.command_input.push(c);
        self.update_command_suggestions();
        self.selected_suggestion = 0;
    }

    pub fn remove_command_char(&mut self) {
        // Don't allow deleting the "/" prefix
        if self.command_input.len() > 1 {
            self.command_input.pop();
            self.update_command_suggestions();
            self.selected_suggestion = 0;
        }
    }

    pub fn update_command_suggestions(&mut self) {
        let partial = command::CommandParser::extract_partial_command(&self.command_input);
        self.command_suggestions = self.command_autocomplete.get_suggestions(&partial, 5);
    }

    pub fn select_next_suggestion(&mut self) {
        if !self.command_suggestions.is_empty() {
            self.selected_suggestion =
                (self.selected_suggestion + 1) % self.command_suggestions.len();
        }
    }

    pub fn select_prev_suggestion(&mut self) {
        if !self.command_suggestions.is_empty() {
            self.selected_suggestion = if self.selected_suggestion == 0 {
                self.command_suggestions.len() - 1
            } else {
                self.selected_suggestion - 1
            };
        }
    }

    pub fn autocomplete_selected(&mut self) {
        if let Some(suggestion) = self.command_suggestions.get(self.selected_suggestion) {
            self.command_input = format!("/{}", suggestion.text);
            self.update_command_suggestions();
        }
    }

    pub fn navigate_command_history_prev(&mut self) {
        if let Some(cmd) = self.command_history.prev(&self.command_input) {
            self.command_input = cmd;
            self.update_command_suggestions();
        }
    }

    pub fn navigate_command_history_next(&mut self) {
        if let Some(cmd) = self.command_history.next() {
            self.command_input = cmd;
            self.update_command_suggestions();
        }
    }

    pub fn execute_command(&mut self) {
        if self.command_input.trim() == "/" || self.command_input.trim().is_empty() {
            self.exit_command_mode();
            return;
        }

        // Parse command
        let parsed = command::CommandParser::parse(&self.command_input);

        // Add to history
        self.command_history.add(self.command_input.clone());

        // Create context
        let mut ctx = command::commands::AppContext {
            view_mode: &mut self.view_mode,
            search_query: &mut self.search_query,
            filter_process: &mut self.filter_process,
            auto_scroll: &mut self.auto_scroll,
            should_quit: &mut self.should_quit,
            logs: &self.logs,
        };

        // Execute command
        let result = self
            .command_registry
            .execute(&parsed.name, parsed.args, &mut ctx);

        // Store result and handle based on success/failure
        match result {
            Ok(msg) => {
                self.last_command_result = Some(command::ExecutionResult::Success(msg));
                // Exit command mode on success
                self.exit_command_mode();
            }
            Err(err) => {
                self.last_command_result = Some(command::ExecutionResult::Error(err));
                // Stay in command mode on error, clear input to try again
                self.command_input = "/".to_string();
                self.update_command_suggestions();
            }
        }
    }

    // ========================================================================
    // NAVIGATION
    // ========================================================================

    pub fn scroll_up(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
        self.auto_scroll = false;
    }

    pub fn scroll_down(&mut self) {
        self.log_scroll += 1;
        self.auto_scroll = false;

        // Re-enable auto-scroll if we scroll to near the bottom
        let total_logs = self.filtered_logs().len();
        if total_logs > 0 && self.log_scroll + 10 >= total_logs {
            self.auto_scroll = true;
            // Don't reset scroll position - let auto-scroll handle it
        }
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(10);
    }

    pub fn scroll_right(&mut self) {
        self.horizontal_scroll += 10;
    }

    pub fn scroll_home(&mut self) {
        self.horizontal_scroll = 0;
    }

    pub fn scroll_page_up(&mut self, page_size: usize) {
        self.log_scroll = self.log_scroll.saturating_sub(page_size);
        self.auto_scroll = false;
    }

    pub fn scroll_page_down(&mut self, page_size: usize) {
        self.log_scroll += page_size;
        self.auto_scroll = false;

        // Re-enable auto-scroll if we scroll to near the bottom
        let total_logs = self.filtered_logs().len();
        if total_logs > 0 && self.log_scroll + 10 >= total_logs {
            self.auto_scroll = true;
            // Don't reset scroll position - let auto-scroll handle it
        }
    }

    pub fn select_next_request(&mut self) {
        let total = self.context_tracker.get_recent_requests().len();
        if total > 0 {
            self.selected_request = (self.selected_request + 1).min(total - 1);
        }
    }

    pub fn select_previous_request(&mut self) {
        if self.selected_request > 0 {
            self.selected_request -= 1;
        }
    }

    pub fn select_next_exception(&mut self) {
        let total = self.exception_tracker.get_grouped_exceptions().len();
        if total > 0 {
            self.selected_exception = (self.selected_exception + 1).min(total - 1);
        }
    }

    pub fn select_previous_exception(&mut self) {
        if self.selected_exception > 0 {
            self.selected_exception -= 1;
        }
    }

    pub fn view_selected_request(&mut self) {
        self.view_mode = ViewMode::RequestDetail(self.selected_request);
    }

    pub fn view_selected_exception(&mut self) {
        self.view_mode = ViewMode::ExceptionDetail(self.selected_exception);
    }

    // ========================================================================
    // FILTERING
    // ========================================================================

    pub fn clear_filter(&mut self) {
        self.filter_process = None;
        self.auto_scroll = true;
        self.log_scroll = 0;
    }

    pub fn enable_auto_scroll(&mut self) {
        self.auto_scroll = true;
        self.log_scroll = 0;
    }

    pub fn filtered_logs(&self) -> Vec<&LogLine> {
        let mut logs: Vec<&LogLine> = if let Some(ref filter) = self.filter_process {
            self.logs
                .iter()
                .filter(|log| &log.process_name == filter)
                .collect()
        } else {
            self.logs.iter().collect()
        };

        // Apply search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            logs.retain(|log| log.content.to_lowercase().contains(&query));
        }

        logs
    }

    // ========================================================================
    // EXPORT
    // ========================================================================

    pub fn export_logs(&self, path: &str) -> Result<(), std::io::Error> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        for log in &self.logs {
            writeln!(file, "[{}] {}", log.process_name, log.content)?;
        }
        Ok(())
    }

    // ========================================================================
    // PROCESS MANAGEMENT
    // ========================================================================

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.processes = processes;
    }

    // ========================================================================
    // APPLICATION CONTROL
    // ========================================================================

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

// ============================================================================
// UI EVENT LOOP
// ============================================================================

/// Run the UI event loop
pub async fn run_ui(
    mut app: App,
    mut log_rx: mpsc::UnboundedReceiver<LogLine>,
    process_manager: std::sync::Arc<crate::process::ProcessManager>,
    _stats_collector: StatsCollector,
    _context_tracker: std::sync::Arc<RequestContextTracker>,
    _db_health: std::sync::Arc<DatabaseHealth>,
    _test_tracker: std::sync::Arc<TestTracker>,
    _exception_tracker: std::sync::Arc<ExceptionTracker>,
    shutdown_flag: std::sync::Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        // Receive new logs (non-blocking)
        while let Ok(log) = log_rx.try_recv() {
            app.add_log(log);
        }

        // Check for external shutdown request (e.g., Ctrl+C)
        if shutdown_flag.load(Ordering::Relaxed) {
            app.quit();
        }

        // Update process list
        let processes = process_manager.get_processes();
        app.update_processes(processes);

        // Update animation frame
        app.spinner_frame = app.spinner_frame.wrapping_add(1);

        // Draw UI using modular render function
        terminal.draw(|f| render_ui(f, &app))?;

        // Handle input (with timeout)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, key);
            }
        }

        if app.should_quit() {
            // Stop all managed processes immediately on quit
            process_manager.stop_all();
            shutdown_flag.store(true, Ordering::Relaxed);
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ============================================================================
// RENDERING
// ============================================================================

// ============================================================================
// RENDERING
// ============================================================================

/// Main rendering dispatcher

fn render_ui(f: &mut ratatui::Frame, app: &App) {
    // Clear the full frame to avoid artifacts bleeding between views/spinner frames
    f.render_widget(Clear, f.area());

    let fade_progress = if let Some(last_change_time) = app.last_view_change_time {
        let elapsed = last_change_time.elapsed();
        let fade_duration = Duration::from_millis(200);
        (elapsed.as_secs_f32() / fade_duration.as_secs_f32()).min(1.0)
    } else {
        1.0
    };

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(4), // For header (with environment info)
            Constraint::Length(3), // For tabs
            Constraint::Min(0),    // For content
            Constraint::Length(1), // For footer
        ])
        .split(f.area());

    render_header(
        f,
        chunks[0],
        &app._git_info,
        &app.environment_info,
        &app.stats_collector,
        &app.test_tracker,
        Some(fade_progress),
    );

    let tab_titles: Vec<_> = ViewMode::all_variants()
        .iter()
        .map(|v| v.as_str())
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Theme::block("Caboose", None) // Using Theme::block with no fade
                .style(
                    Style::default()
                        .fg(Theme::text_primary())
                        .bg(Theme::surface()),
                ),
        )
        .select(app.active_tab_index)
        .style(Style::default().fg(Theme::text_secondary()))
        .highlight_style(
            Style::default()
                .fg(Theme::primary())
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[1]);

    match &app.view_mode {
        ViewMode::Logs => {
            views::logs_view::render(
                f,
                chunks[2],
                &app.processes,
                &app.logs,
                app.search_mode,
                &app.search_query,
                app.log_scroll,
                app.horizontal_scroll,
                app.auto_scroll,
                &app.filter_process,
                app.spinner_frame,
                Some(fade_progress),
            );
        }

        ViewMode::QueryAnalysis => {
            views::query_analysis_view::render(
                f,
                chunks[2],
                &app.context_tracker,
                app.spinner_frame,
                Some(fade_progress),
            );
        }

        ViewMode::RequestDetail(idx) => {
            render_request_detail_view_fallback(f, chunks[2], app, *idx);
        }

        ViewMode::DatabaseHealth => {
            views::database_health_view::render(
                f,
                chunks[2],
                &app.db_health,
                app.spinner_frame,
                Some(fade_progress),
            );
        }

        ViewMode::TestResults => {
            views::test_results_view::render(
                f,
                chunks[2],
                &app.test_tracker,
                app.spinner_frame,
                Some(fade_progress),
            );
        }

        ViewMode::Exceptions => {
            views::exceptions_view::render(
                f,
                chunks[2],
                &app.exception_tracker,
                app.selected_exception,
                app.spinner_frame,
                Some(fade_progress),
            );
        }

        ViewMode::ExceptionDetail(exception_index) => {
            views::exception_detail_view::render(
                f,
                chunks[2],
                &app.exception_tracker,
                *exception_index,
                Some(fade_progress),
            );
        }
    }

    render_footer(f, chunks[3], app, Some(fade_progress));

    // Render command palette overlay if in command mode
    if app.command_mode {
        let palette_area = components::command_palette::calculate_palette_area(f.area());

        // Get error message if in command mode with error
        let error_msg = if let Some(ref result) = app.last_command_result {
            if !result.is_success() {
                result.message()
            } else {
                None
            }
        } else {
            None
        };

        components::command_palette::render_command_palette(
            f,
            palette_area,
            &app.command_input,
            &app.command_suggestions,
            app.selected_suggestion,
            error_msg,
            Some(fade_progress),
        );
    } else if let Some(ref result) = app.last_command_result {
        // Only show success messages after command mode exits
        if result.is_success() {
            if let Some(message) = result.message() {
                let result_area = Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(f.area())[1];

                components::command_palette::render_command_result(
                    f,
                    result_area,
                    message,
                    false,
                    Some(fade_progress),
                );
            }
        }
    }
}

fn render_header(
    f: &mut ratatui::Frame,

    area: ratatui::layout::Rect,

    git_info: &GitInfo,

    environment_info: &crate::environment::EnvironmentInfo,

    stats_collector: &StatsCollector,

    test_tracker: &std::sync::Arc<crate::test::TestTracker>,

    fade_progress: Option<f32>,
) {
    let stats = stats_collector.get_stats();

    let error_rate = stats.error_rate();

    let avg_time = stats.avg_response_time();

    let response_time_history = stats_collector.get_response_time_history();
    // Convert u64 to f64 for Sparkline
    let response_time_history_f64: Vec<f64> =
        response_time_history.iter().map(|&x| x as f64).collect();

    // Define overall header layout
    let _header_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Environment line
            Constraint::Length(1), // Git info line
            Constraint::Length(1), // Stats line + Sparkline
        ])
        .split(area);

    // Render Block around header content
    // Get username from environment or use "caboose" as fallback
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "caboose".to_string());

    let header_block = Block::default()
        .title(Span::styled(
            format!(" {} ", username),
            Style::default()
                .fg(Theme::apply_fade_to_color(
                    Theme::primary(),
                    fade_progress.unwrap_or(1.0),
                ))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::apply_fade_to_color(
            Theme::text_muted(),
            fade_progress.unwrap_or(1.0),
        )));

    // Compute inner area before rendering to avoid move
    let inner_area = header_block.inner(area);
    let inner_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Environment line
            Constraint::Length(1), // Git info line
            Constraint::Length(1), // Stats line + Sparkline
        ])
        .split(inner_area);

    // Environment segments (Powerlevel10k style)
    let env_segments = environment_info.format_segment();
    let env_line = Line::from(
        env_segments
            .iter()
            .enumerate()
            .flat_map(|(i, segment)| {
                let mut spans = Vec::new();

                if i > 0 {
                    spans.push(Span::styled(
                        " │ ",
                        Style::default().fg(Theme::apply_fade_to_color(
                            Theme::text_muted(),
                            fade_progress.unwrap_or(1.0),
                        )),
                    ));
                }

                spans.push(Span::styled(
                    segment,
                    Style::default().fg(Theme::apply_fade_to_color(
                        Theme::text_secondary(),
                        fade_progress.unwrap_or(1.0),
                    )),
                ));

                spans
            })
            .collect::<Vec<_>>(),
    );
    f.render_widget(Paragraph::new(env_line), inner_chunks[0]);

    // Build git line with optional debugger indicator
    let mut git_spans = vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            Icons::git(),
            Style::default().fg(Theme::apply_fade_to_color(
                Theme::info(),
                fade_progress.unwrap_or(1.0),
            )),
        ),
        Span::raw(" "),
        Span::styled(
            git_info.format_short(),
            Style::default()
                .fg(Theme::apply_fade_to_color(
                    Theme::primary(),
                    fade_progress.unwrap_or(1.0),
                ))
                .add_modifier(Modifier::BOLD),
        ),
    ];

    // Add debugger indicator if active
    if test_tracker.is_debugger_active() {
        git_spans.push(Span::raw("   │   "));

        if let Some(info) = test_tracker.get_debugger_info() {
            let debugger_text = format!(
                "⚡ {:?} @ {}:{}",
                info.debugger_type,
                info.file_path.as_deref().unwrap_or("unknown"),
                info.line_number
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".to_string())
            );
            git_spans.push(Span::styled(
                debugger_text,
                Style::default()
                    .fg(Theme::apply_fade_to_color(
                        Theme::warning(),
                        fade_progress.unwrap_or(1.0),
                    ))
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            git_spans.push(Span::styled(
                "⚡ Debugger Active",
                Style::default()
                    .fg(Theme::apply_fade_to_color(
                        Theme::warning(),
                        fade_progress.unwrap_or(1.0),
                    ))
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    let git_line = Line::from(git_spans);
    f.render_widget(Paragraph::new(git_line), inner_chunks[1]);

    // Stats line and Sparkline
    let stats_layout = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // total requests
            Constraint::Length(15), // avg time
            Constraint::Length(10), // sparkline
            Constraint::Length(15), // error rate
            Constraint::Min(0),     // sql queries (flexible)
        ])
        .split(inner_chunks[2]);

    // Render total requests
    let total_requests_span = Span::styled(
        format!(
            "   {} {} requests",
            Icons::success(),
            format_number(stats.total_requests)
        ),
        Style::default().fg(Theme::apply_fade_to_color(
            Theme::success(),
            fade_progress.unwrap_or(1.0),
        )),
    );
    f.render_widget(Paragraph::new(total_requests_span), stats_layout[0]);

    // Render avg time
    let avg_time_span = Span::styled(
        format!("{} {} avg", Icons::info(), format_ms(avg_time)),
        Style::default().fg(Theme::apply_fade_to_color(
            Theme::warning(),
            fade_progress.unwrap_or(1.0),
        )),
    );
    f.render_widget(Paragraph::new(avg_time_span), stats_layout[1]);

    // Render Sparkline as text
    let sparkline = Sparkline::new(&response_time_history_f64);
    let sparkline_span = Span::styled(
        sparkline.render(),
        Style::default().fg(Theme::apply_fade_to_color(
            Theme::warning(),
            fade_progress.unwrap_or(1.0),
        )),
    );
    f.render_widget(Paragraph::new(sparkline_span), stats_layout[2]);

    // Render error rate
    let error_rate_text = format_percentage(error_rate);
    let error_rate_color = if error_rate > 5.0 {
        Theme::danger()
    } else {
        Theme::success()
    };
    let error_rate_span = Span::styled(
        format!(
            " {} {} errors",
            if error_rate > 5.0 {
                Icons::error()
            } else {
                Icons::success()
            },
            error_rate_text
        ),
        Style::default().fg(Theme::apply_fade_to_color(
            error_rate_color,
            fade_progress.unwrap_or(1.0),
        )),
    );
    f.render_widget(Paragraph::new(error_rate_span), stats_layout[3]);

    // Render sql queries
    let sql_queries_span = Span::styled(
        format!(
            " {} {} queries",
            Icons::database(),
            format_number(stats.sql_queries)
        ),
        Style::default().fg(Theme::apply_fade_to_color(
            Theme::info(),
            fade_progress.unwrap_or(1.0),
        )),
    );
    f.render_widget(Paragraph::new(sql_queries_span), stats_layout[4]);

    f.render_widget(header_block, area); // This line was missing
}

fn render_footer(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &App,
    fade_progress: Option<f32>,
) {
    let footer = if app.search_mode {
        FooterBuilder::new()
            .add_binding("Type to search", "")
            .add_binding("Esc", "Cancel")
            .add_binding("Enter", "Apply")
            .build()
    } else {
        let mut footer = FooterBuilder::new()
            .add_binding("q", "Quit")
            .add_binding(":", "Command")
            .add_binding("t/T", "Tab ←→");

        // Add view-specific bindings
        if matches!(app.view_mode, ViewMode::Logs) {
            footer = footer
                .add_binding("/", "Search")
                .add_binding("↑↓", "V-Scroll")
                .add_binding("←→", "H-Scroll");

            // Show auto-scroll or Home hint
            if !app.auto_scroll {
                footer = footer.add_binding("End", "⚠️ Auto-scroll OFF");
            } else if app.horizontal_scroll > 0 {
                footer = footer.add_binding("Home", "Reset H-Scroll");
            } else {
                footer = footer.add_binding("c", "Clear");
            }
        } else {
            footer = footer
                .add_binding("/", "Search")
                .add_binding("↑↓", "Scroll")
                .add_binding("c", "Clear");
        }

        footer.build()
    };

    let footer_widget = Paragraph::new(footer).style(
        Style::default()
            .bg(Theme::apply_fade_to_color(
                Theme::surface(),
                fade_progress.unwrap_or(1.0),
            ))
            .fg(Theme::apply_fade_to_color(
                Theme::text_secondary(),
                fade_progress.unwrap_or(1.0),
            )),
    );

    f.render_widget(footer_widget, area);
}

// ============================================================================

// KEY HANDLING

// ============================================================================

fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Clear success messages on any key press
    if let Some(ref result) = app.last_command_result {
        if result.is_success() && !app.command_mode {
            app.last_command_result = None;
        }
    }

    // Handle command mode first
    if app.command_mode {
        // Clear error messages on typing in command mode
        if app.last_command_result.is_some() && matches!(key.code, KeyCode::Char(_)) {
            app.last_command_result = None;
        }

        match key.code {
            KeyCode::Char(c) => app.add_command_char(c),
            KeyCode::Backspace => app.remove_command_char(),
            KeyCode::Esc => app.exit_command_mode(),
            KeyCode::Enter => app.execute_command(),
            KeyCode::Tab => app.autocomplete_selected(),
            KeyCode::Down => {
                if app.command_suggestions.is_empty() {
                    // No suggestions - navigate history forward
                    app.navigate_command_history_next();
                } else {
                    // Has suggestions - navigate suggestions
                    app.select_next_suggestion();
                }
            }
            KeyCode::Up => {
                if app.command_suggestions.is_empty() {
                    // No suggestions - navigate history backward
                    app.navigate_command_history_prev();
                } else {
                    // Has suggestions - navigate suggestions
                    app.select_prev_suggestion();
                }
            }
            _ => {}
        }
        return;
    }

    // Handle search mode separately
    if app.search_mode {
        match key.code {
            KeyCode::Char(c) => app.add_search_char(c),
            KeyCode::Backspace => app.remove_search_char(),
            KeyCode::Esc => {
                app.exit_search_mode();
                app.enable_auto_scroll();
            }
            KeyCode::Enter => {
                app.exit_search_mode();
                app.enable_auto_scroll();
            }
            _ => {}
        }
        return;
    }

    // Normal mode key handling
    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => {
            // Esc only navigates back, doesn't quit
            match app.view_mode {
                ViewMode::RequestDetail(_) => app.view_mode = ViewMode::QueryAnalysis,
                ViewMode::ExceptionDetail(_) => app.view_mode = ViewMode::Exceptions,
                _ => {} // Do nothing in other views
            }
        }
        KeyCode::Char('t') => app.toggle_view(),
        KeyCode::Char('T') => app.toggle_view_backward(), // Shift+T for backward cycling
        KeyCode::Char(':') => app.enter_command_mode(),
        KeyCode::Char('/') => {
            if matches!(app.view_mode, ViewMode::Logs) {
                app.enter_search_mode();
            }
        }
        KeyCode::Char('c') => app.clear_filter(),
        KeyCode::End => app.enable_auto_scroll(),
        KeyCode::Up => match app.view_mode {
            ViewMode::Logs => app.scroll_up(),
            ViewMode::QueryAnalysis => app.select_previous_request(),
            ViewMode::Exceptions => app.select_previous_exception(),
            _ => {}
        },
        KeyCode::Down => match app.view_mode {
            ViewMode::Logs => app.scroll_down(),
            ViewMode::QueryAnalysis => app.select_next_request(),
            ViewMode::Exceptions => app.select_next_exception(),
            _ => {}
        },
        KeyCode::Left => {
            if matches!(app.view_mode, ViewMode::Logs) {
                app.scroll_left();
            }
        }
        KeyCode::Right => {
            if matches!(app.view_mode, ViewMode::Logs) {
                app.scroll_right();
            }
        }
        KeyCode::Home => {
            if matches!(app.view_mode, ViewMode::Logs) {
                app.scroll_home();
            }
        }
        KeyCode::PageUp => {
            if matches!(app.view_mode, ViewMode::Logs) {
                app.scroll_page_up(10);
            }
        }
        KeyCode::PageDown => {
            if matches!(app.view_mode, ViewMode::Logs) {
                app.scroll_page_down(10);
            }
        }
        KeyCode::Enter => match app.view_mode {
            ViewMode::QueryAnalysis => app.view_selected_request(),
            ViewMode::Exceptions => app.view_selected_exception(),
            _ => {}
        },
        KeyCode::Char('e') => {
            if matches!(app.view_mode, ViewMode::Logs) {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let filename = format!("caboose_logs_{}.txt", timestamp);
                let _ = app.export_logs(&filename);
            }
        }
        _ => {}
    }
}

// ============================================================================
// FALLBACK IMPLEMENTATIONS (to be migrated to views module)
// ============================================================================

// These are temporary fallback implementations using the original code
// They will be gradually migrated to the views module

fn render_request_detail_view_fallback(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &App,
    idx: usize,
) {
    let requests = app.context_tracker.get_recent_requests();
    let lines = if let Some(req) = requests.get(idx) {
        let path = req
            .context
            .path
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        let qcount = req.context.query_count();
        let duration = req.total_duration.unwrap_or(0.0);
        vec![
            Line::raw("Request Detail (fallback)"),
            Line::raw(format!("Path: {}", path)),
            Line::raw(format!("Status: {:?}", req.status.unwrap_or(0))),
            Line::raw(format!("Queries: {}", qcount)),
            Line::raw(format!("Duration: {:.1}ms", duration)),
        ]
    } else {
        vec![Line::raw("No request selected")]
    };

    let block = Block::default()
        .title("Request Details")
        .borders(Borders::ALL);
    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

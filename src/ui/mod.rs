/// UI Module - Terminal User Interface
///
/// This module provides a modular, professional-grade terminal UI framework
/// following DRY principles and clean architecture patterns.

// Public modules
pub mod theme;
pub mod formatting;
pub mod widgets;
pub mod components;
pub mod views;

// Re-exports for convenience
pub use theme::Theme;
pub use formatting::*;

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

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};

use std::io;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
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
    stats_collector: StatsCollector,
    context_tracker: std::sync::Arc<RequestContextTracker>,
    db_health: std::sync::Arc<DatabaseHealth>,
    test_tracker: std::sync::Arc<TestTracker>,
    exception_tracker: std::sync::Arc<ExceptionTracker>,

    // UI state
    search_mode: bool,
    search_query: String,
    log_scroll: usize,
    auto_scroll: bool,
    _request_scroll: usize,
    selected_request: usize,
    selected_exception: usize,
    filter_process: Option<String>,
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
        Self {
            processes: Vec::new(),
            logs: Vec::new(),
            max_logs: 1000,
            should_quit: false,
            _git_info: git_info,
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
            auto_scroll: true,
            _request_scroll: 0,
            selected_request: 0,
            selected_exception: 0,
            filter_process: None,
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
        self.view_mode = ViewMode::from_index(next_index).unwrap_or(ViewMode::Logs);
        self.active_tab_index = next_index;
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
    }

    pub fn scroll_page_up(&mut self, page_size: usize) {
        self.log_scroll = self.log_scroll.saturating_sub(page_size);
        self.auto_scroll = false;
    }

    pub fn scroll_page_down(&mut self, page_size: usize) {
        self.log_scroll += page_size;
        self.auto_scroll = false;
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

    let chunks = Layout::default()

        .direction(ratatui::layout::Direction::Vertical)

        .constraints([

            Constraint::Length(3), // For header

            Constraint::Length(3), // For tabs

            Constraint::Min(0),    // For content

            Constraint::Length(1), // For footer

        ])

        .split(f.area());



    render_header(f, chunks[0], &app._git_info, &app.stats_collector);



    let tab_titles: Vec<_> = ViewMode::all_variants()

        .iter()

        .map(|v| v.as_str())

        .collect();



    let tabs = Tabs::new(tab_titles)

        .block(

            Block::default()

                .borders(Borders::ALL)

                .title("Caboose")

                .style(Style::default().fg(Theme::TEXT_PRIMARY).bg(Theme::SURFACE)),

        )

        .select(app.active_tab_index)

        .style(Style::default().fg(Theme::TEXT_SECONDARY))

        .highlight_style(

            Style::default()

                .fg(Theme::PRIMARY)

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

                app.auto_scroll,

                &app.filter_process,

            );

        }

        ViewMode::QueryAnalysis => {

            views::query_analysis_view::render(f, chunks[2], &app.context_tracker);

        }

        ViewMode::RequestDetail(idx) => {

            render_request_detail_view_fallback(f, chunks[2], app, *idx);

        }

        ViewMode::DatabaseHealth => {

            views::database_health_view::render(f, chunks[2], &app.db_health);

        }

        ViewMode::TestResults => {

            views::test_results_view::render(f, chunks[2], &app.test_tracker);

        }

        ViewMode::Exceptions => {

            views::exceptions_view::render(f, chunks[2], &app.exception_tracker);

        }

    }



    render_footer(f, chunks[3], app.search_mode);

}



fn render_header(

    f: &mut ratatui::Frame,

    area: ratatui::layout::Rect,

    git_info: &GitInfo,

    stats_collector: &StatsCollector,

) {

    let stats = stats_collector.get_stats();

    let error_rate = stats.error_rate();

    let avg_time = stats.avg_response_time();



    let git_line = Line::from(vec![

        Span::styled(" ", Style::default()),

        Span::styled(Icons::GIT, Style::default().fg(Theme::INFO)),

        Span::raw(" "),

        Span::styled(

            git_info.format_short(),

            Style::default()

                .fg(Theme::PRIMARY)

                .add_modifier(Modifier::BOLD),

        ),

    ]);



    let stats_line = Line::from(vec![

        Span::styled(

            format!("   {} ", Icons::SUCCESS),

            Style::default().fg(Theme::SUCCESS),

        ),

        Span::styled(

            format_number(stats.total_requests),

            Style::default()

                .fg(Theme::TEXT_PRIMARY)

                .add_modifier(Modifier::BOLD),

        ),

        Span::styled(" requests", Style::default().fg(Theme::TEXT_SECONDARY)),

        Span::raw("   │   "),

        Span::styled(Icons::INFO, Style::default().fg(Theme::WARNING)),

        Span::raw(" "),

        Span::styled(

            format_ms(avg_time),

            Style::default()

                .fg(Theme::WARNING)

                .add_modifier(Modifier::BOLD),

        ),

        Span::styled(" avg", Style::default().fg(Theme::TEXT_SECONDARY)),

        Span::raw("   │   "),

        Span::styled(

            if error_rate > 5.0 {

                Icons::ERROR

            } else {

                Icons::SUCCESS

            },

            Style::default().fg(if error_rate > 5.0 {

                Theme::DANGER

            } else {

                Theme::SUCCESS

            }),

        ),

        Span::raw(" "),

        Span::styled(

            format_percentage(error_rate),

            Style::default()

                .fg(if error_rate > 5.0 {

                    Theme::DANGER

                } else {

                    Theme::SUCCESS

                })

                .add_modifier(Modifier::BOLD),

        ),

        Span::styled(" errors", Style::default().fg(Theme::TEXT_SECONDARY)),

        Span::raw("   │   "),

        Span::styled(

            format!("{} ", Icons::DATABASE),

            Style::default().fg(Theme::INFO),

        ),

        Span::styled(

            format_number(stats.sql_queries),

            Style::default()

                .fg(Theme::INFO)

                .add_modifier(Modifier::BOLD),

        ),

        Span::styled(" queries", Style::default().fg(Theme::TEXT_SECONDARY)),

    ]);



    let header = Paragraph::new(vec![git_line, stats_line]).block(

        Block::default()

            .title(Span::styled(

                " Caboose ",

                Style::default()

                    .fg(Theme::PRIMARY)

                    .add_modifier(Modifier::BOLD),

            ))

            .borders(Borders::ALL)

            .border_style(Style::default().fg(Theme::TEXT_MUTED)),

    );



    f.render_widget(header, area);

}



fn render_footer(f: &mut ratatui::Frame, area: ratatui::layout::Rect, search_mode: bool) {

    let footer = if search_mode {

        FooterBuilder::new()

            .add_binding("Type to search", "")

            .add_binding("Esc", "Cancel")

            .add_binding("Enter", "Apply")

            .build()

    } else {

        FooterBuilder::new()

            .add_binding(format!("{} Quit", Icons::QUIT), "")

            .add_binding(format!("{} Toggle", Icons::TOGGLE), "")

            .add_binding(format!("{} Search", Icons::SEARCH), "")

            .add_binding(format!("{} Scroll", Icons::SCROLL), "")

            .add_binding(format!("{} Clear", Icons::CLEAR), "")

            .build()

    };



    let footer_widget = Paragraph::new(footer).style(

        Style::default()

            .bg(Theme::SURFACE)

            .fg(Theme::TEXT_SECONDARY),

    );



    f.render_widget(footer_widget, area);

}



// ============================================================================

// KEY HANDLING

// ============================================================================

fn handle_key_event(app: &mut App, key: KeyEvent) {
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
            match app.view_mode {
                ViewMode::RequestDetail(_) => app.view_mode = ViewMode::QueryAnalysis,
                _ => app.quit(),
            }
        }
        KeyCode::Char('t') => app.toggle_view(),
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
        KeyCode::Enter => {
            if matches!(app.view_mode, ViewMode::QueryAnalysis) {
                app.view_selected_request();
            }
        }
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



fn render_request_detail_view_fallback(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App, idx: usize) {
    let requests = app.context_tracker.get_recent_requests();
    let lines = if let Some(req) = requests.get(idx) {
        let path = req.context.path.clone().unwrap_or_else(|| "<unknown>".to_string());
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







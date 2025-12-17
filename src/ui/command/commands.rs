/// Built-in commands for the application
use super::registry::{Command, CommandContext, CommandResult};
use crate::ui::ViewMode;

/// Command context implementation for the application
pub struct AppContext<'a> {
    pub view_mode: &'a mut ViewMode,
    pub search_query: &'a mut String,
    pub filter_process: &'a mut Option<String>,
    pub auto_scroll: &'a mut bool,
    pub should_quit: &'a mut bool,
    pub logs: &'a Vec<crate::process::LogLine>,
}

impl<'a> CommandContext for AppContext<'a> {}

// ============================================================================
// QUIT COMMAND
// ============================================================================

pub struct QuitCommand;

impl Command for QuitCommand {
    fn name(&self) -> &str {
        "quit"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["q", "exit"]
    }

    fn description(&self) -> &str {
        "Exit the application"
    }

    fn usage(&self) -> &str {
        "/quit"
    }

    fn execute(&self, _args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
        // Safety: We know this is always AppContext in our application
        let ctx = unsafe { &mut *(ctx as *mut dyn CommandContext as *mut AppContext) };

        *ctx.should_quit = true;
        Ok("Quitting application...".to_string())
    }
}

// ============================================================================
// SEARCH COMMAND
// ============================================================================

pub struct SearchCommand;

impl Command for SearchCommand {
    fn name(&self) -> &str {
        "search"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["s", "find", "/"]
    }

    fn description(&self) -> &str {
        "Search logs for a query"
    }

    fn usage(&self) -> &str {
        "/search <query>"
    }

    fn arg_hints(&self) -> Vec<&str> {
        vec![
            "error", "warn", "info", "debug", "SELECT", "INSERT", "UPDATE",
        ]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn execute(&self, args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
        // Safety: We know this is always AppContext in our application
        let ctx = unsafe { &mut *(ctx as *mut dyn CommandContext as *mut AppContext) };

        let query = args.join(" ");
        *ctx.search_query = query.clone();
        *ctx.auto_scroll = false;

        Ok(format!("Searching for: '{}'", query))
    }
}

// ============================================================================
// CLEAR COMMAND
// ============================================================================

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["c", "reset"]
    }

    fn description(&self) -> &str {
        "Clear search and filters"
    }

    fn usage(&self) -> &str {
        "/clear"
    }

    fn execute(&self, _args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
        // Safety: We know this is always AppContext in our application
        let ctx = unsafe { &mut *(ctx as *mut dyn CommandContext as *mut AppContext) };

        ctx.search_query.clear();
        *ctx.filter_process = None;
        *ctx.auto_scroll = true;

        Ok("Cleared all filters".to_string())
    }
}

// ============================================================================
// VIEW COMMAND
// ============================================================================

pub struct ViewCommand;

impl Command for ViewCommand {
    fn name(&self) -> &str {
        "view"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["v", "switch"]
    }

    fn description(&self) -> &str {
        "Switch to a different view"
    }

    fn usage(&self) -> &str {
        "/view <logs|query|db|tests|exceptions>"
    }

    fn arg_hints(&self) -> Vec<&str> {
        vec!["logs", "query", "db", "tests", "exceptions"]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }

    fn execute(&self, args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
        // Safety: We know this is always AppContext in our application
        let ctx = unsafe { &mut *(ctx as *mut dyn CommandContext as *mut AppContext) };

        let view_name = args[0].to_lowercase();

        *ctx.view_mode = match view_name.as_str() {
            "logs" | "log" => ViewMode::Logs,
            "query" | "queries" | "sql" => ViewMode::QueryAnalysis,
            "db" | "database" | "health" => ViewMode::DatabaseHealth,
            "tests" | "test" => ViewMode::TestResults,
            "exceptions" | "errors" | "err" => ViewMode::Exceptions,
            _ => {
                return Err(format!(
                    "Unknown view: '{}'. Available views: logs, query, db, tests, exceptions",
                    view_name
                ));
            }
        };

        Ok(format!("Switched to {} view", ctx.view_mode.as_str()))
    }
}

// ============================================================================
// FILTER COMMAND
// ============================================================================

pub struct FilterCommand;

impl Command for FilterCommand {
    fn name(&self) -> &str {
        "filter"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["f"]
    }

    fn description(&self) -> &str {
        "Filter logs by process name"
    }

    fn usage(&self) -> &str {
        "/filter <process>"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn execute(&self, args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
        // Safety: We know this is always AppContext in our application
        let ctx = unsafe { &mut *(ctx as *mut dyn CommandContext as *mut AppContext) };

        let process = args[0].clone();
        *ctx.filter_process = Some(process.clone());
        *ctx.auto_scroll = false;

        Ok(format!("Filtering by process: '{}'", process))
    }
}

// ============================================================================
// EXPORT COMMAND
// ============================================================================

pub struct ExportCommand;

impl Command for ExportCommand {
    fn name(&self) -> &str {
        "export"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["e", "save"]
    }

    fn description(&self) -> &str {
        "Export logs to a file"
    }

    fn usage(&self) -> &str {
        "/export <filename>"
    }

    fn arg_hints(&self) -> Vec<&str> {
        vec!["logs.txt", "output.log"]
    }

    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }

    fn execute(&self, args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
        // Safety: We know this is always AppContext in our application
        let ctx = unsafe { &mut *(ctx as *mut dyn CommandContext as *mut AppContext) };

        let filename = if args.is_empty() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("Failed to get timestamp: {}", e))?
                .as_secs();
            format!("caboose_logs_{}.txt", timestamp)
        } else {
            args[0].clone()
        };

        // Write logs to file
        use std::fs::File;
        use std::io::Write;

        let mut file =
            File::create(&filename).map_err(|e| format!("Failed to create file: {}", e))?;

        for log in ctx.logs {
            writeln!(file, "[{}] {}", log.process_name, log.content)
                .map_err(|e| format!("Failed to write to file: {}", e))?;
        }

        Ok(format!(
            "Exported {} logs to '{}'",
            ctx.logs.len(),
            filename
        ))
    }
}

// ============================================================================
// HELP COMMAND
// ============================================================================

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["h", "?"]
    }

    fn description(&self) -> &str {
        "Show available commands"
    }

    fn usage(&self) -> &str {
        "/help"
    }

    fn execute(&self, _args: Vec<String>, _ctx: &mut dyn CommandContext) -> CommandResult {
        Ok("Available commands:\n\
            /quit (q, exit) - Exit the application\n\
            /search <query> (s, find) - Search logs\n\
            /clear (c, reset) - Clear filters\n\
            /view <name> (v) - Switch views\n\
            /filter <process> (f) - Filter by process\n\
            /export [file] (e) - Export logs\n\
            /theme <name> (color) - Change color theme\n\
            /icons [on|off|toggle] - Toggle icon mode\n\
            /help (h, ?) - Show this help"
            .to_string())
    }
}

// ============================================================================
// THEME COMMAND
// ============================================================================

pub struct ThemeCommand;

impl Command for ThemeCommand {
    fn name(&self) -> &str {
        "theme"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["color", "colors"]
    }

    fn description(&self) -> &str {
        "Switch color theme"
    }

    fn usage(&self) -> &str {
        "/theme <name>"
    }

    fn arg_hints(&self) -> Vec<&str> {
        vec!["material", "solarized", "dracula", "nord", "tokyo-night"]
    }

    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }

    fn execute(&self, args: Vec<String>, _ctx: &mut dyn CommandContext) -> CommandResult {
        use crate::ui::themes::{ThemeManager, ThemeName};

        if args.is_empty() {
            // List available themes
            let current = ThemeManager::current();
            let themes = ThemeName::all()
                .iter()
                .map(|t| {
                    if *t == current {
                        format!("• {} (active)", t.display_name())
                    } else {
                        format!("  {}", t.display_name())
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            Ok(format!(
                "Available themes:\n{}\n\nUsage: /theme <name>",
                themes
            ))
        } else {
            // Set theme
            match ThemeManager::set_by_name(&args[0]) {
                Ok(theme) => Ok(format!("Theme changed to: {}", theme.display_name())),
                Err(err) => Err(err),
            }
        }
    }
}

// ============================================================================
// ICON COMMAND
// ============================================================================

pub struct IconCommand;

impl Command for IconCommand {
    fn name(&self) -> &str {
        "icons"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["icon"]
    }

    fn description(&self) -> &str {
        "Toggle between Nerd Fonts and ASCII icons"
    }

    fn usage(&self) -> &str {
        "/icons [on|off|toggle]"
    }

    fn arg_hints(&self) -> Vec<&str> {
        vec!["on", "off", "toggle"]
    }

    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }

    fn execute(&self, args: Vec<String>, _ctx: &mut dyn CommandContext) -> CommandResult {
        use crate::ui::icon_manager::IconManager;

        if args.is_empty() {
            // Show current status
            let current = if IconManager::using_nerd_fonts() {
                "Nerd Fonts (Unicode symbols)"
            } else {
                "ASCII (maximum compatibility)"
            };
            Ok(format!(
                "Current icon mode: {}\n\nUsage: /icons [on|off|toggle]\n  on     - Enable Nerd Fonts\n  off    - Use ASCII icons\n  toggle - Switch between modes",
                current
            ))
        } else {
            match args[0].to_lowercase().as_str() {
                "on" | "nerd" | "unicode" => {
                    IconManager::set_nerd_fonts(true);
                    Ok("Switched to Nerd Fonts icons ✔".to_string())
                }
                "off" | "ascii" => {
                    IconManager::set_nerd_fonts(false);
                    Ok("Switched to ASCII icons [✓]".to_string())
                }
                "toggle" | "switch" => {
                    let new_val = IconManager::toggle();
                    let mode = if new_val { "Nerd Fonts" } else { "ASCII" };
                    Ok(format!("Toggled to {} icons", mode))
                }
                _ => Err("Invalid argument. Use: on, off, or toggle".to_string()),
            }
        }
    }
}

// ============================================================================
// COMMAND BUILDER
// ============================================================================

/// Builder for creating and registering all built-in commands
pub fn build_command_registry() -> super::registry::CommandRegistry {
    let mut registry = super::registry::CommandRegistry::new();

    registry.register(Box::new(QuitCommand));
    registry.register(Box::new(SearchCommand));
    registry.register(Box::new(ClearCommand));
    registry.register(Box::new(ViewCommand));
    registry.register(Box::new(FilterCommand));
    registry.register(Box::new(ExportCommand));
    registry.register(Box::new(ThemeCommand));
    registry.register(Box::new(IconCommand));
    registry.register(Box::new(HelpCommand));

    registry
}

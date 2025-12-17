pub mod autocomplete;
pub mod commands;
pub mod history;
pub mod parser;
/// Command system module - Claude CLI inspired command palette
///
/// This module provides a modern, extensible command system with:
/// - Trait-based command registration
/// - Autocomplete with fuzzy matching
/// - Command history
/// - Type-safe argument parsing
///
/// # Architecture
///
/// ```text
/// ┌─────────────────┐
/// │ CommandPalette  │ (UI Component)
/// └────────┬────────┘
///          │
/// ┌────────▼────────┐
/// │ CommandRegistry │ (Command Storage & Dispatch)
/// └────────┬────────┘
///          │
/// ┌────────▼────────┐
/// │ Command Trait   │ (Individual Commands)
/// └─────────────────┘
/// ```
pub mod registry;

pub use autocomplete::AutocompleteEngine;
pub use history::CommandHistory;
pub use parser::CommandParser;
pub use registry::{Command, CommandContext, CommandRegistry, CommandResult};

use crate::ui::ViewMode;

/// Command execution context containing app state references
pub struct AppCommandContext<'a> {
    pub view_mode: &'a mut ViewMode,
    pub search_query: &'a mut String,
    pub filter_process: &'a mut Option<String>,
    pub auto_scroll: &'a mut bool,
    pub should_quit: &'a mut bool,
}

/// Result of command execution
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Success(String),
    Error(String),
    NoOp,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success(_))
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            ExecutionResult::Success(msg) | ExecutionResult::Error(msg) => Some(msg),
            ExecutionResult::NoOp => None,
        }
    }
}

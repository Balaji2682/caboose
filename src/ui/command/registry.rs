/// Command registry for managing and dispatching commands
use std::collections::HashMap;

/// Command execution context (generic to allow flexibility)
pub trait CommandContext {}

/// Result of command execution
pub type CommandResult = Result<String, String>;

/// Trait for implementing commands
///
/// # Example
///
/// ```rust
/// use caboose::ui::command::{Command, CommandContext, CommandResult};
///
/// struct QuitCommand;
///
/// impl Command for QuitCommand {
///     fn name(&self) -> &str { "quit" }
///     fn aliases(&self) -> Vec<&str> { vec!["q", "exit"] }
///     fn description(&self) -> &str { "Exit the application" }
///     fn usage(&self) -> &str { "/quit" }
///
///     fn execute(&self, args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult {
///         Ok("Quitting...".to_string())
///     }
/// }
/// ```
pub trait Command: Send + Sync {
    /// Primary command name (e.g., "search")
    fn name(&self) -> &str;

    /// Alternative names/shortcuts (e.g., ["s", "find"])
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Short description for autocomplete
    fn description(&self) -> &str;

    /// Usage example (e.g., "/search <query>")
    fn usage(&self) -> &str;

    /// Argument hints for autocomplete
    fn arg_hints(&self) -> Vec<&str> {
        vec![]
    }

    /// Minimum number of arguments required
    fn min_args(&self) -> usize {
        0
    }

    /// Maximum number of arguments (None = unlimited)
    fn max_args(&self) -> Option<usize> {
        None
    }

    /// Execute the command with given arguments
    fn execute(&self, args: Vec<String>, ctx: &mut dyn CommandContext) -> CommandResult;

    /// Validate arguments before execution
    fn validate_args(&self, args: &[String]) -> Result<(), String> {
        let arg_count = args.len();

        if arg_count < self.min_args() {
            return Err(format!(
                "Too few arguments. Expected at least {}, got {}.\nUsage: {}",
                self.min_args(),
                arg_count,
                self.usage()
            ));
        }

        if let Some(max) = self.max_args() {
            if arg_count > max {
                return Err(format!(
                    "Too many arguments. Expected at most {}, got {}.\nUsage: {}",
                    max,
                    arg_count,
                    self.usage()
                ));
            }
        }

        Ok(())
    }
}

/// Command metadata for autocomplete and help
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub usage: String,
    pub arg_hints: Vec<String>,
}

/// Registry for managing all available commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
    aliases: HashMap<String, String>, // alias -> primary name
    metadata: Vec<CommandMetadata>,
}

impl CommandRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            metadata: Vec::new(),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: Box<dyn Command>) {
        let name = command.name().to_string();

        // Store metadata for autocomplete
        self.metadata.push(CommandMetadata {
            name: name.clone(),
            aliases: command.aliases().iter().map(|s| s.to_string()).collect(),
            description: command.description().to_string(),
            usage: command.usage().to_string(),
            arg_hints: command.arg_hints().iter().map(|s| s.to_string()).collect(),
        });

        // Register aliases -> primary name mapping
        for alias in command.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
        }

        // Register primary command
        self.commands.insert(name, command);
    }

    /// Get all command metadata (for autocomplete)
    pub fn get_metadata(&self) -> &[CommandMetadata] {
        &self.metadata
    }

    /// Find a command by name or alias
    pub fn find(&self, name: &str) -> Option<&dyn Command> {
        // Try direct lookup first
        if let Some(cmd) = self.commands.get(name) {
            return Some(cmd.as_ref());
        }

        // Try alias lookup
        if let Some(primary_name) = self.aliases.get(name) {
            return self.commands.get(primary_name).map(|cmd| cmd.as_ref());
        }

        None
    }

    /// Execute a command by name with arguments
    pub fn execute(
        &self,
        name: &str,
        args: Vec<String>,
        ctx: &mut dyn CommandContext,
    ) -> CommandResult {
        match self.find(name) {
            Some(cmd) => {
                // Validate arguments
                cmd.validate_args(&args)?;

                // Execute command
                cmd.execute(args, ctx)
            }
            None => Err(format!(
                "Unknown command: '{}'. Type /help for available commands.",
                name
            )),
        }
    }

    /// Get all command names (including aliases)
    pub fn all_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Get primary command names only (no aliases)
    pub fn primary_names(&self) -> Vec<String> {
        self.metadata.iter().map(|m| m.name.clone()).collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockContext;
    impl CommandContext for MockContext {}

    struct TestCommand;
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }
        fn aliases(&self) -> Vec<&str> {
            vec!["t"]
        }
        fn description(&self) -> &str {
            "Test command"
        }
        fn usage(&self) -> &str {
            "/test"
        }
        fn execute(&self, _args: Vec<String>, _ctx: &mut dyn CommandContext) -> CommandResult {
            Ok("executed".to_string())
        }
    }

    #[test]
    fn test_register_and_find() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand));

        assert!(registry.find("test").is_some());
        assert!(registry.find("t").is_some());
        assert!(registry.find("unknown").is_none());
    }

    #[test]
    fn test_execute_command() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand));

        let mut ctx = MockContext;
        let result = registry.execute("test", vec![], &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_command() {
        let registry = CommandRegistry::new();
        let mut ctx = MockContext;
        let result = registry.execute("unknown", vec![], &mut ctx);
        assert!(result.is_err());
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CabooseConfig {
    #[serde(default)]
    pub frontend: FrontendConfig,
    #[serde(default)]
    pub rails: RailsConfig,
    #[serde(default)]
    pub processes: HashMap<String, ProcessOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    /// Explicit path to frontend directory (overrides auto-detection)
    pub path: Option<String>,

    /// Disable frontend auto-detection
    #[serde(default)]
    pub disable_auto_detect: bool,

    /// Custom dev command (overrides framework default)
    pub dev_command: Option<String>,

    /// Custom port (overrides framework default)
    pub port: Option<u16>,

    /// Process name in Procfile (default: "frontend")
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RailsConfig {
    /// Rails server port (default: 3000)
    pub port: Option<u16>,

    /// Disable Rails auto-detection
    #[serde(default)]
    pub disable_auto_detect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOverride {
    /// Custom command for this process
    pub command: Option<String>,

    /// Environment variables for this process
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl CabooseConfig {
    /// Load configuration from .caboose.toml
    pub fn load() -> Self {
        Self::load_from(".caboose.toml")
            .or_else(|| Self::load_from("caboose.toml"))
            .unwrap_or_default()
    }

    fn load_from(path: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Create example configuration file
    pub fn create_example() -> String {
        r#"# Caboose Configuration File
# Save as .caboose.toml in your project root

[frontend]
# Explicit path to frontend directory (overrides auto-detection)
# path = "client"

# Disable auto-detection (useful if you have multiple frontend dirs)
# disable_auto_detect = false

# Custom dev command (overrides framework default)
# dev_command = "npm run dev -- --port 5173"

# Custom port
# port = 5173

# Custom process name in logs (default: "frontend")
# process_name = "ui"

[rails]
# Rails server port (default: 3000)
# port = 3000

# Disable Rails auto-detection
# disable_auto_detect = false

# Process-specific overrides
# [processes.web]
# command = "bundle exec puma -p 4000"
# env = { RAILS_ENV = "development" }

# [processes.frontend]
# command = "cd client && pnpm dev"
# env = { NODE_ENV = "development" }
"#
        .to_string()
    }
}

#[derive(Debug)]
pub struct Procfile {
    pub processes: Vec<ProcessConfig>,
}

impl Procfile {
    /// Parse a Procfile from the given path
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read Procfile: {}", e))?;

        Self::parse_content(&content)
    }

    /// Parse Procfile content
    pub fn parse_content(content: &str) -> Result<Self, String> {
        let mut processes = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse "name: command" format
            if let Some((name, command)) = line.split_once(':') {
                let name = name.trim().to_string();
                let command = command.trim().to_string();

                if name.is_empty() {
                    return Err(format!("Empty process name at line {}", line_num + 1));
                }
                if command.is_empty() {
                    return Err(format!(
                        "Empty command for process '{}' at line {}",
                        name,
                        line_num + 1
                    ));
                }

                processes.push(ProcessConfig { name, command });
            } else {
                return Err(format!(
                    "Invalid format at line {}: expected 'name: command'",
                    line_num + 1
                ));
            }
        }

        if processes.is_empty() {
            return Err("No processes found in Procfile".to_string());
        }

        Ok(Procfile { processes })
    }
}

/// Load environment variables from .env file
pub fn load_env<P: AsRef<Path>>(path: P) -> Result<HashMap<String, String>, String> {
    let mut env_vars = HashMap::new();

    if !path.as_ref().exists() {
        return Ok(env_vars);
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read .env file: {}", e))?;

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            env_vars.insert(key, value);
        } else {
            eprintln!("Warning: Invalid .env format at line {}", line_num + 1);
        }
    }

    Ok(env_vars)
}

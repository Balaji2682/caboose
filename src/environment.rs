//! Environment information detection (Powerlevel10k-style)
//!
//! Detects project environment information like language versions,
//! package managers, current path, etc.

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Environment information for the current project
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub current_path: String,
    pub ruby_version: Option<String>,
    pub node_version: Option<String>,
    pub package_manager: Option<PackageManagerInfo>,
    pub rails_version: Option<String>,
    pub database: Option<String>,
}

/// Package manager information
#[derive(Debug, Clone)]
pub struct PackageManagerInfo {
    pub name: String,
    pub version: String,
}

impl EnvironmentInfo {
    /// Detect all environment information
    pub fn detect() -> Self {
        Self {
            current_path: Self::get_current_path(),
            ruby_version: Self::detect_ruby_version(),
            node_version: Self::detect_node_version(),
            package_manager: Self::detect_package_manager(),
            rails_version: Self::detect_rails_version(),
            database: Self::detect_database(),
        }
    }

    /// Get current working directory (shortened)
    fn get_current_path() -> String {
        if let Ok(path) = env::current_dir() {
            // Get the last 2 components of the path for brevity
            let components: Vec<_> = path.components().collect();
            if components.len() > 2 {
                let last_two: PathBuf = components[components.len() - 2..].iter().collect();
                format!(".../{}", last_two.display())
            } else {
                path.display().to_string()
            }
        } else {
            "~".to_string()
        }
    }

    /// Detect Ruby version
    fn detect_ruby_version() -> Option<String> {
        Command::new("ruby")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok().and_then(|s| {
                        // Parse "ruby 3.2.0p0 (2023-03-30)" -> "3.2.0"
                        s.split_whitespace()
                            .nth(1)
                            .map(|v| v.split('p').next().unwrap_or(v).to_string())
                    })
                } else {
                    None
                }
            })
    }

    /// Detect Node.js version
    fn detect_node_version() -> Option<String> {
        Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().strip_prefix('v').unwrap_or(&s).to_string())
                } else {
                    None
                }
            })
    }

    /// Detect package manager and version
    fn detect_package_manager() -> Option<PackageManagerInfo> {
        // Check for lockfiles to determine package manager
        if std::path::Path::new("pnpm-lock.yaml").exists() {
            Self::get_pm_version("pnpm", "--version").map(|v| PackageManagerInfo {
                name: "pnpm".to_string(),
                version: v,
            })
        } else if std::path::Path::new("yarn.lock").exists() {
            Self::get_pm_version("yarn", "--version").map(|v| PackageManagerInfo {
                name: "yarn".to_string(),
                version: v,
            })
        } else if std::path::Path::new("bun.lockb").exists() {
            Self::get_pm_version("bun", "--version").map(|v| PackageManagerInfo {
                name: "bun".to_string(),
                version: v,
            })
        } else if std::path::Path::new("package-lock.json").exists() {
            Self::get_pm_version("npm", "--version").map(|v| PackageManagerInfo {
                name: "npm".to_string(),
                version: v,
            })
        } else {
            None
        }
    }

    /// Get package manager version
    fn get_pm_version(cmd: &str, arg: &str) -> Option<String> {
        Command::new(cmd).arg(arg).output().ok().and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
    }

    /// Detect Rails version
    fn detect_rails_version() -> Option<String> {
        Command::new("rails")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok().and_then(|s| {
                        // Parse "Rails 7.0.4" -> "7.0.4"
                        s.split_whitespace().nth(1).map(|v| v.to_string())
                    })
                } else {
                    None
                }
            })
    }

    /// Detect database from config/database.yml or Gemfile
    fn detect_database() -> Option<String> {
        // Try to read database.yml
        if let Ok(contents) = std::fs::read_to_string("config/database.yml") {
            if contents.contains("postgresql") || contents.contains("adapter: postgresql") {
                return Some("PostgreSQL".to_string());
            } else if contents.contains("mysql") {
                return Some("MySQL".to_string());
            } else if contents.contains("sqlite3") {
                return Some("SQLite".to_string());
            }
        }

        // Fallback to checking Gemfile
        if let Ok(contents) = std::fs::read_to_string("Gemfile") {
            if contents.contains("pg") {
                return Some("PostgreSQL".to_string());
            } else if contents.contains("mysql2") {
                return Some("MySQL".to_string());
            } else if contents.contains("sqlite3") {
                return Some("SQLite".to_string());
            }
        }

        None
    }

    /// Format as a compact segment (Powerlevel10k style)
    pub fn format_segment(&self) -> Vec<String> {
        let mut segments = Vec::new();

        // Path segment
        segments.push(format!("📁 {}", self.current_path));

        // Ruby segment
        if let Some(ref version) = self.ruby_version {
            segments.push(format!("💎 {}", version));
        }

        // Rails segment
        if let Some(ref version) = self.rails_version {
            segments.push(format!("🛤️ {}", version));
        }

        // Node segment
        if let Some(ref version) = self.node_version {
            segments.push(format!("⬢ {}", version));
        }

        // Package manager segment
        if let Some(ref pm) = self.package_manager {
            segments.push(format!("📦 {} {}", pm.name, pm.version));
        }

        // Database segment
        if let Some(ref db) = self.database {
            segments.push(format!("🗄️ {}", db));
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        let env = EnvironmentInfo::detect();
        assert!(!env.current_path.is_empty());
        // Other fields may or may not be present depending on environment
    }

    #[test]
    fn test_format_segment() {
        let env = EnvironmentInfo::detect();
        let segments = env.format_segment();
        assert!(!segments.is_empty());
    }
}

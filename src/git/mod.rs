use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub has_changes: bool,
    pub ahead: usize,
    pub behind: usize,
}

impl GitInfo {
    pub fn get() -> Self {
        let mut info = GitInfo::default();

        // Get current branch
        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
        {
            if output.status.success() {
                info.branch = String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string());
            }
        }

        // Check for uncommitted changes
        if let Ok(output) = Command::new("git").args(["status", "--porcelain"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                info.has_changes = !stdout.trim().is_empty();
            }
        }

        // Get ahead/behind counts
        if let Ok(output) = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
                if parts.len() == 2 {
                    info.ahead = parts[0].parse().unwrap_or(0);
                    info.behind = parts[1].parse().unwrap_or(0);
                }
            }
        }

        info
    }

    pub fn format_short(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref branch) = self.branch {
            parts.push(branch.clone());
        }

        if self.has_changes {
            parts.push("*".to_string());
        }

        if self.ahead > 0 {
            parts.push(format!("↑{}", self.ahead));
        }

        if self.behind > 0 {
            parts.push(format!("↓{}", self.behind));
        }

        if parts.is_empty() {
            "no git".to_string()
        } else {
            parts.join(" ")
        }
    }
}

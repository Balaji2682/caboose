use std::env;
/// Runtime icon detection and management
///
/// Detects terminal capabilities and switches between Nerd Fonts and ASCII icons
/// similar to how Claude Code handles terminal rendering.
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag for Nerd Font usage (thread-safe)
static USE_NERD_FONTS: AtomicBool = AtomicBool::new(false);

/// Icon manager for runtime detection and configuration
pub struct IconManager;

impl IconManager {
    /// Detect terminal capabilities and set icon mode automatically
    ///
    /// This should be called once at application startup.
    /// Detection strategy (in order of priority):
    /// 1. Environment variable override (CABOOSE_NERD_FONTS)
    /// 2. Known terminal programs (iTerm, Alacritty, kitty, etc.)
    /// 3. TERM capabilities (256color support)
    /// 4. Conservative fallback to ASCII
    pub fn detect() {
        let should_use_nerd_fonts = Self::can_use_nerd_fonts();
        USE_NERD_FONTS.store(should_use_nerd_fonts, Ordering::Relaxed);
    }

    /// Check if terminal supports Nerd Fonts
    fn can_use_nerd_fonts() -> bool {
        // Strategy 1: Check environment variable (user override)
        if let Ok(val) = env::var("CABOOSE_NERD_FONTS") {
            return val == "1" || val.to_lowercase() == "true";
        }

        // Strategy 2: Check TERM_PROGRAM (known good terminals)
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "iTerm.app" => return true,
                "WezTerm" => return true,
                "Alacritty" => return true,
                "kitty" => return true,
                "vscode" => return true, // VS Code integrated terminal
                "Hyper" => return true,
                _ => {}
            }
        }

        // Strategy 3: Check TERM for advanced capabilities
        if let Ok(term) = env::var("TERM") {
            // Terminals with 256color usually handle Unicode well
            if term.contains("256color") {
                return true;
            }
            // kitty terminal
            if term.contains("kitty") {
                return true;
            }
            // xterm-256color is common in modern terminals
            if term == "xterm-256color" {
                return true;
            }
        }

        // Strategy 4: Check for WSL (Windows Subsystem for Linux)
        // Modern Windows Terminal supports Nerd Fonts well
        if let Ok(wsl) = env::var("WSL_DISTRO_NAME") {
            if !wsl.is_empty() {
                // Check if using Windows Terminal
                if let Ok(term_program) = env::var("WT_SESSION") {
                    if !term_program.is_empty() {
                        return true;
                    }
                }
            }
        }

        // Default: Conservative - use ASCII for maximum compatibility
        false
    }

    /// Check if Nerd Fonts are currently enabled
    pub fn using_nerd_fonts() -> bool {
        USE_NERD_FONTS.load(Ordering::Relaxed)
    }

    /// Manually enable/disable Nerd Fonts (runtime toggle)
    pub fn set_nerd_fonts(enabled: bool) {
        USE_NERD_FONTS.store(enabled, Ordering::Relaxed);
    }

    /// Toggle between Nerd Fonts and ASCII
    pub fn toggle() -> bool {
        let new_val = !Self::using_nerd_fonts();
        Self::set_nerd_fonts(new_val);
        new_val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_manager_toggle() {
        let initial = IconManager::using_nerd_fonts();
        let toggled = IconManager::toggle();
        assert_eq!(toggled, !initial);

        // Toggle back
        let toggled_back = IconManager::toggle();
        assert_eq!(toggled_back, initial);
    }

    #[test]
    fn test_icon_manager_set() {
        IconManager::set_nerd_fonts(true);
        assert!(IconManager::using_nerd_fonts());

        IconManager::set_nerd_fonts(false);
        assert!(!IconManager::using_nerd_fonts());
    }
}

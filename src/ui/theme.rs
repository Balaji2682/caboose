/// Theme module - Color system and visual theming for the application
///
/// # Material Design 3 Color System
///
/// This module provides a Material Design 3 inspired color palette with:
/// - Primary and secondary brand colors
/// - Status colors (success, warning, danger, info)
/// - Background and surface colors
/// - Text hierarchy (primary, secondary, muted)
///
/// # Icons
///
/// Icons use ASCII by default for maximum compatibility.
/// To enable Nerd Fonts, change USE_NERD_FONTS constant to true.
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders};

/// Application color palette - Uses active theme from ThemeManager
pub struct Theme;

impl Theme {
    // ============================================================================
    // Dynamic Theme Colors (from active theme)
    // ============================================================================

    // Primary
    pub fn primary() -> Color {
        super::themes::ThemeManager::palette().primary
    }

    pub fn primary_variant() -> Color {
        super::themes::ThemeManager::palette().primary_variant
    }

    // Secondary
    pub fn secondary() -> Color {
        super::themes::ThemeManager::palette().secondary
    }

    // Backgrounds
    pub fn background() -> Color {
        super::themes::ThemeManager::palette().background
    }

    pub fn surface() -> Color {
        super::themes::ThemeManager::palette().surface
    }

    // Text
    pub fn text_primary() -> Color {
        super::themes::ThemeManager::palette().text_primary
    }

    pub fn text_secondary() -> Color {
        super::themes::ThemeManager::palette().text_secondary
    }

    pub fn text_muted() -> Color {
        super::themes::ThemeManager::palette().text_muted
    }

    // Status
    pub fn success() -> Color {
        super::themes::ThemeManager::palette().success
    }

    pub fn success_bright() -> Color {
        super::themes::ThemeManager::palette().success_bright
    }

    pub fn warning() -> Color {
        super::themes::ThemeManager::palette().warning
    }

    pub fn danger() -> Color {
        super::themes::ThemeManager::palette().danger
    }

    pub fn info() -> Color {
        super::themes::ThemeManager::palette().info
    }

    // Accents
    pub fn accent() -> Color {
        super::themes::ThemeManager::palette().accent
    }

    // ============================================================================
    // Legacy Constants (for backwards compatibility during migration)
    // ============================================================================

    // Will be removed after full migration to methods
    pub const PRIMARY: Color = Color::Rgb(139, 92, 246);
    pub const PRIMARY_VARIANT: Color = Color::Rgb(109, 40, 217);
    pub const SECONDARY: Color = Color::Rgb(236, 72, 153);
    pub const BACKGROUND: Color = Color::Rgb(17, 24, 39);
    pub const SURFACE: Color = Color::Rgb(31, 41, 55);
    pub const TEXT_PRIMARY: Color = Color::Rgb(243, 244, 246);
    pub const TEXT_SECONDARY: Color = Color::Rgb(156, 163, 175);
    pub const TEXT_MUTED: Color = Color::Rgb(75, 85, 99);
    pub const SUCCESS: Color = Color::Rgb(16, 185, 129);
    pub const SUCCESS_BRIGHT: Color = Color::Rgb(34, 197, 94);
    pub const WARNING: Color = Color::Rgb(245, 158, 11);
    pub const DANGER: Color = Color::Rgb(239, 68, 68);
    pub const INFO: Color = Color::Rgb(59, 130, 246);
    pub const ACCENT: Color = Color::Rgb(249, 115, 22);

    // ============================================================================
    // Dynamic Color Helpers
    // ============================================================================

    /// Get color based on health percentage (0-100)
    pub fn health_color(health: u8) -> Color {
        match health {
            90..=100 => Self::success_bright(),
            80..=89 => Self::success(),
            70..=79 => Self::warning(),
            40..=69 => Color::LightRed,
            _ => Self::danger(),
        }
    }

    /// Get color based on duration in milliseconds
    pub fn duration_color(duration: f64) -> Color {
        match duration {
            d if d < 50.0 => Self::success(),
            d if d < 100.0 => Self::warning(),
            d if d < 200.0 => Color::LightRed,
            _ => Self::danger(),
        }
    }

    /// Get color based on HTTP status code
    pub fn status_code_color(status: u16) -> Color {
        match status {
            200..=299 => Self::success(),
            300..=399 => Self::info(),
            400..=499 => Self::warning(),
            500..=599 => Self::danger(),
            _ => Self::text_secondary(),
        }
    }

    // ============================================================================
    // Block Styling (Claude-like appearance)
    // ============================================================================

    /// Get the default border type (rounded like Claude Code)
    pub fn border_type() -> BorderType {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            BorderType::Rounded  // Smooth rounded corners: ╭─╮│╰─╯
        } else {
            BorderType::Plain    // Simple ASCII: +--+|
        }
    }

    /// Create a styled block with title (Claude Code style)
    pub fn block<'a>(title: impl Into<ratatui::text::Line<'a>>) -> Block<'a> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(Self::border_type())
            .border_style(ratatui::style::Style::default().fg(Self::primary()))
    }

    /// Create a styled block without title
    pub fn block_plain<'a>() -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(Self::border_type())
            .border_style(ratatui::style::Style::default().fg(Self::text_muted()))
    }

    /// Create a focused/active block (brighter border)
    pub fn block_focused<'a>(title: impl Into<ratatui::text::Line<'a>>) -> Block<'a> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(Self::border_type())
            .border_style(ratatui::style::Style::default().fg(Self::accent()))
    }
}

/// Icon set with runtime detection
///
/// Automatically detects terminal capabilities and switches between
/// Nerd Fonts and ASCII icons. Works like Claude Code terminal rendering.
///
/// Detection is done automatically at startup, but you can also:
/// - Set environment variable: `export CABOOSE_NERD_FONTS=1`
/// - Use command: `/icons on` or `/icons off`
/// - Toggle at runtime: `/icons toggle`
pub struct Icons;

impl Icons {
    // ============================================================================
    // General
    // ============================================================================

    pub fn right_arrow() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "\u{f054}"
        } else {
            ">"
        }
    }

    pub fn right_triangle() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "▶"
        } else {
            ">"
        }
    }

    pub const SEPARATOR: &'static str = "|";

    // ============================================================================
    // Status
    // ============================================================================

    pub fn success() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "✔"
        } else {
            "[✓]"
        }
    }

    pub fn error() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "✖"
        } else {
            "[✗]"
        }
    }

    pub fn warning() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⚠"
        } else {
            "[!]"
        }
    }

    pub fn info() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "ℹ"
        } else {
            "[i]"
        }
    }

    pub fn running() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "●"
        } else {
            "[*]"
        }
    }

    pub fn stopped() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "○"
        } else {
            "[ ]"
        }
    }

    // ============================================================================
    // Categories
    // ============================================================================

    pub fn git() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⎇"  // Unicode: git branch symbol
        } else {
            "[git]"
        }
    }

    pub fn database() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⚑"  // Unicode: database/flag symbol
        } else {
            "[db]"
        }
    }

    pub fn test() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "✓"  // Unicode: test/check symbol
        } else {
            "[test]"
        }
    }

    pub fn query() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "↯"  // Unicode: query/lightning symbol
        } else {
            "[sql]"
        }
    }

    pub fn exception() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⚠"  // Unicode: warning triangle (same as warning())
        } else {
            "[err]"
        }
    }

    pub fn logs() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "◉"  // Unicode: log/document symbol
        } else {
            "[log]"
        }
    }

    // ============================================================================
    // Actions
    // ============================================================================

    pub fn quit() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "◼"  // Unicode: quit/stop symbol
        } else {
            "[q]"
        }
    }

    pub fn search() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⌕"  // Unicode: search/magnifying glass
        } else {
            "[/]"
        }
    }

    pub fn scroll() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "↕"  // Unicode: already good!
        } else {
            "[^v]"
        }
    }

    pub fn clear() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⌫"  // Unicode: clear/delete symbol
        } else {
            "[c]"
        }
    }

    pub fn toggle() -> &'static str {
        if super::icon_manager::IconManager::using_nerd_fonts() {
            "⇄"  // Unicode: toggle/switch symbol
        } else {
            "[t]"
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_color_ranges() {
        // Ensure Material Design theme is active for consistent test results
        use crate::ui::themes::{ThemeName, ThemeManager};
        ThemeManager::set(ThemeName::MaterialDesign);

        assert_eq!(Theme::health_color(100), Theme::success_bright());
        assert_eq!(Theme::health_color(85), Theme::success());
        assert_eq!(Theme::health_color(75), Theme::warning());
        assert_eq!(Theme::health_color(50), Color::LightRed);
        assert_eq!(Theme::health_color(20), Theme::danger());
    }

    #[test]
    fn test_duration_color() {
        // Ensure Material Design theme is active for consistent test results
        use crate::ui::themes::{ThemeName, ThemeManager};
        ThemeManager::set(ThemeName::MaterialDesign);

        assert_eq!(Theme::duration_color(25.0), Theme::success());
        assert_eq!(Theme::duration_color(75.0), Theme::warning());
        assert_eq!(Theme::duration_color(150.0), Color::LightRed);
        assert_eq!(Theme::duration_color(500.0), Theme::danger());
    }

    #[test]
    fn test_status_code_color() {
        // Ensure Material Design theme is active for consistent test results
        use crate::ui::themes::{ThemeName, ThemeManager};
        ThemeManager::set(ThemeName::MaterialDesign);

        assert_eq!(Theme::status_code_color(200), Theme::success());
        assert_eq!(Theme::status_code_color(404), Theme::warning());
        assert_eq!(Theme::status_code_color(500), Theme::danger());
    }

    #[test]
    fn test_icons_runtime_detection() {
        // Icons should respond to runtime detection
        // By default (without CABOOSE_NERD_FONTS env), should be ASCII
        use crate::ui::icon_manager::IconManager;

        // Ensure we're in ASCII mode for testing
        IconManager::set_nerd_fonts(false);
        assert_eq!(Icons::success(), "[✓]");
        assert_eq!(Icons::error(), "[✗]");
        assert_eq!(Icons::git(), "[git]");
        assert_eq!(Icons::database(), "[db]");

        // Test toggling to Nerd Fonts
        IconManager::set_nerd_fonts(true);
        assert_eq!(Icons::success(), "✔");
        assert_eq!(Icons::error(), "✖");

        // Reset to ASCII for other tests
        IconManager::set_nerd_fonts(false);
    }
}

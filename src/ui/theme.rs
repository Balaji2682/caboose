/// Theme module - Color system and visual theming for the application
use ratatui::style::Color;

/// Application color palette following Google's Material Design system.
pub struct Theme;

impl Theme {
    // ============================================================================
    // Material Design Color Palette
    // ============================================================================

    // Primary
    pub const PRIMARY: Color = Color::Rgb(139, 92, 246); // Violet 500
    pub const PRIMARY_VARIANT: Color = Color::Rgb(109, 40, 217); // Violet 600

    // Secondary
    pub const SECONDARY: Color = Color::Rgb(236, 72, 153); // Pink 500

    // Backgrounds
    pub const BACKGROUND: Color = Color::Rgb(17, 24, 39); // Gray 900
    pub const SURFACE: Color = Color::Rgb(31, 41, 55); // Gray 800

    // Text
    pub const TEXT_PRIMARY: Color = Color::Rgb(243, 244, 246); // Gray 100
    pub const TEXT_SECONDARY: Color = Color::Rgb(156, 163, 175); // Gray 400
    pub const TEXT_MUTED: Color = Color::Rgb(75, 85, 99); // Gray 600

    // Status
    pub const SUCCESS: Color = Color::Rgb(16, 185, 129); // Green 500
    pub const WARNING: Color = Color::Rgb(245, 158, 11); // Amber 500
    pub const DANGER: Color = Color::Rgb(239, 68, 68); // Red 500
    pub const INFO: Color = Color::Rgb(59, 130, 246); // Blue 500

    // Accents
    pub const ACCENT: Color = Color::Rgb(249, 115, 22); // Orange 500
}

/// Icon set using Nerd Fonts for a modern look.
pub struct Icons;

impl Icons {
    // General
    pub const RIGHT_ARROW: &'static str = "";
    pub const RIGHT_TRIANGLE: &'static str = "▶";
    pub const SEPARATOR: &'static str = "│";

    // Status
    pub const SUCCESS: &'static str = "✔";
    pub const ERROR: &'static str = "✖";
    pub const WARNING: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
    pub const RUNNING: &'static str = "●";
    pub const STOPPED: &'static str = "○";

    // Categories
    pub const GIT: &'static str = "";
    pub const DATABASE: &'static str = "";
    pub const TEST: &'static str = "";
    pub const QUERY: &'static str = "";
    pub const EXCEPTION: &'static str = "";
    pub const LOGS: &'static str = "";

    // Actions
    pub const QUIT: &'static str = "";
    pub const SEARCH: &'static str = "";
    pub const SCROLL: &'static str = "↕";
    pub const CLEAR: &'static str = "";
    pub const TOGGLE: &'static str = "";
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_color_ranges() {
        assert_eq!(Theme::health_color(100), Theme::SUCCESS_BRIGHT);
        assert_eq!(Theme::health_color(85), Theme::SUCCESS);
        assert_eq!(Theme::health_color(75), Theme::WARNING);
        assert_eq!(Theme::health_color(50), Color::LightRed);
        assert_eq!(Theme::health_color(20), Theme::DANGER);
    }

    #[test]
    fn test_duration_color() {
        assert_eq!(Theme::duration_color(25.0), Theme::SUCCESS);
        assert_eq!(Theme::duration_color(75.0), Theme::WARNING);
        assert_eq!(Theme::duration_color(150.0), Color::LightRed);
        assert_eq!(Theme::duration_color(500.0), Theme::DANGER);
    }

    #[test]
    fn test_status_code_color() {
        assert_eq!(Theme::status_code_color(200), Theme::SUCCESS);
        assert_eq!(Theme::status_code_color(404), Theme::WARNING);
        assert_eq!(Theme::status_code_color(500), Theme::DANGER);
    }
}

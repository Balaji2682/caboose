/// Theme definitions - 5 popular color schemes
///
/// Includes Material Design 3, Solarized Dark, Dracula, Nord, and Tokyo Night
use ratatui::style::Color;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Available theme names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    MaterialDesign,
    SolarizedDark,
    Dracula,
    Nord,
    TokyoNight,
    Catppuccin,
}

impl ThemeName {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeName::MaterialDesign => "material",
            ThemeName::SolarizedDark => "solarized",
            ThemeName::Dracula => "dracula",
            ThemeName::Nord => "nord",
            ThemeName::TokyoNight => "tokyo-night",
            ThemeName::Catppuccin => "catppuccin",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeName::MaterialDesign => "Material Design 3",
            ThemeName::SolarizedDark => "Solarized Dark",
            ThemeName::Dracula => "Dracula",
            ThemeName::Nord => "Nord",
            ThemeName::TokyoNight => "Tokyo Night",
            ThemeName::Catppuccin => "Catppuccin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "material" | "material-design" | "md3" => Some(ThemeName::MaterialDesign),
            "solarized" | "solarized-dark" => Some(ThemeName::SolarizedDark),
            "dracula" => Some(ThemeName::Dracula),
            "nord" => Some(ThemeName::Nord),
            "tokyo-night" | "tokyo" | "tokyonight" => Some(ThemeName::TokyoNight),
            "catppuccin" | "cat" => Some(ThemeName::Catppuccin),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            ThemeName::MaterialDesign,
            ThemeName::SolarizedDark,
            ThemeName::Dracula,
            ThemeName::Nord,
            ThemeName::TokyoNight,
            ThemeName::Catppuccin,
        ]
    }
}

/// Color palette for a theme
#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Primary colors
    pub primary: Color,
    pub primary_variant: Color,
    pub secondary: Color,

    // Backgrounds
    pub background: Color,
    pub surface: Color,

    // Text
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    // Status
    pub success: Color,
    pub success_bright: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,

    // Accent
    pub accent: Color,
}

impl ColorPalette {
    /// Material Design 3 (Default) - Modern purple and pink theme
    pub fn material_design() -> Self {
        Self {
            primary: Color::Rgb(139, 92, 246),         // Violet 500
            primary_variant: Color::Rgb(109, 40, 217), // Violet 600
            secondary: Color::Rgb(236, 72, 153),       // Pink 500
            background: Color::Rgb(17, 24, 39),        // Gray 900
            surface: Color::Rgb(31, 41, 55),           // Gray 800
            text_primary: Color::Rgb(243, 244, 246),   // Gray 100
            text_secondary: Color::Rgb(156, 163, 175), // Gray 400
            text_muted: Color::Rgb(75, 85, 99),        // Gray 600
            success: Color::Rgb(16, 185, 129),         // Green 500
            success_bright: Color::Rgb(34, 197, 94),   // Green 400
            warning: Color::Rgb(245, 158, 11),         // Amber 500
            danger: Color::Rgb(239, 68, 68),           // Red 500
            info: Color::Rgb(59, 130, 246),            // Blue 500
            accent: Color::Rgb(249, 115, 22),          // Orange 500
        }
    }

    /// Solarized Dark - Classic balanced dark theme
    pub fn solarized_dark() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),         // Blue
            primary_variant: Color::Rgb(42, 161, 152), // Cyan
            secondary: Color::Rgb(211, 54, 130),       // Magenta
            background: Color::Rgb(0, 43, 54),         // Base03
            surface: Color::Rgb(7, 54, 66),            // Base02
            text_primary: Color::Rgb(131, 148, 150),   // Base0
            text_secondary: Color::Rgb(88, 110, 117),  // Base01
            text_muted: Color::Rgb(101, 123, 131),     // Base00
            success: Color::Rgb(133, 153, 0),          // Green
            success_bright: Color::Rgb(133, 153, 0),   // Green (same as success)
            warning: Color::Rgb(181, 137, 0),          // Yellow
            danger: Color::Rgb(220, 50, 47),           // Red
            info: Color::Rgb(38, 139, 210),            // Blue
            accent: Color::Rgb(203, 75, 22),           // Orange
        }
    }

    /// Dracula - Popular dark theme with vibrant colors
    pub fn dracula() -> Self {
        Self {
            primary: Color::Rgb(189, 147, 249),         // Purple
            primary_variant: Color::Rgb(139, 233, 253), // Cyan
            secondary: Color::Rgb(255, 121, 198),       // Pink
            background: Color::Rgb(40, 42, 54),         // Background
            surface: Color::Rgb(68, 71, 90),            // Current Line
            text_primary: Color::Rgb(248, 248, 242),    // Foreground
            text_secondary: Color::Rgb(98, 114, 164),   // Comment
            text_muted: Color::Rgb(98, 114, 164),       // Comment
            success: Color::Rgb(80, 250, 123),          // Green
            success_bright: Color::Rgb(139, 233, 253),  // Cyan
            warning: Color::Rgb(241, 250, 140),         // Yellow
            danger: Color::Rgb(255, 85, 85),            // Red
            info: Color::Rgb(139, 233, 253),            // Cyan
            accent: Color::Rgb(255, 184, 108),          // Orange
        }
    }

    /// Nord - Arctic, north-bluish color palette
    pub fn nord() -> Self {
        Self {
            primary: Color::Rgb(136, 192, 208),        // Nord9 (Frost Blue)
            primary_variant: Color::Rgb(94, 129, 172), // Nord10 (Frost Dark)
            secondary: Color::Rgb(129, 161, 193),      // Nord8 (Frost Light)
            background: Color::Rgb(46, 52, 64),        // Nord0 (Polar Night)
            surface: Color::Rgb(59, 66, 82),           // Nord1
            text_primary: Color::Rgb(236, 239, 244),   // Nord6 (Snow Storm)
            text_secondary: Color::Rgb(216, 222, 233), // Nord4
            text_muted: Color::Rgb(76, 86, 106),       // Nord3
            success: Color::Rgb(163, 190, 140),        // Nord14 (Aurora Green)
            success_bright: Color::Rgb(143, 188, 187), // Nord7
            warning: Color::Rgb(235, 203, 139),        // Nord13 (Aurora Yellow)
            danger: Color::Rgb(191, 97, 106),          // Nord11 (Aurora Red)
            info: Color::Rgb(136, 192, 208),           // Nord9
            accent: Color::Rgb(208, 135, 112),         // Nord12 (Aurora Orange)
        }
    }

    /// Tokyo Night - Modern dark theme inspired by Tokyo at night
    pub fn tokyo_night() -> Self {
        Self {
            primary: Color::Rgb(122, 162, 247),         // Blue
            primary_variant: Color::Rgb(125, 207, 255), // Light Blue
            secondary: Color::Rgb(187, 154, 247),       // Purple
            background: Color::Rgb(26, 27, 38),         // Background
            surface: Color::Rgb(36, 40, 59),            // Surface
            text_primary: Color::Rgb(192, 202, 245),    // Foreground
            text_secondary: Color::Rgb(86, 95, 137),    // Comment
            text_muted: Color::Rgb(86, 95, 137),        // Comment
            success: Color::Rgb(158, 206, 106),         // Green
            success_bright: Color::Rgb(125, 207, 255),  // Cyan
            warning: Color::Rgb(224, 175, 104),         // Yellow
            danger: Color::Rgb(247, 118, 142),          // Red
            info: Color::Rgb(125, 207, 255),            // Cyan
            accent: Color::Rgb(255, 158, 100),          // Orange
        }
    }

    /// Catppuccin (Mocha) - Soothing dark theme for cozy coding
    pub fn catppuccin() -> Self {
        Self {
            primary: Color::Rgb(137, 180, 250),         // Blue
            primary_variant: Color::Rgb(116, 199, 236), // Sapphire
            secondary: Color::Rgb(203, 166, 247),       // Mauve
            background: Color::Rgb(30, 30, 46),         // Base
            surface: Color::Rgb(49, 50, 68),            // Surface0
            text_primary: Color::Rgb(205, 214, 244),    // Text
            text_secondary: Color::Rgb(166, 173, 200),  // Subtext0
            text_muted: Color::Rgb(108, 112, 134),      // Overlay0
            success: Color::Rgb(166, 227, 161),         // Green
            success_bright: Color::Rgb(166, 227, 161),  // Green
            warning: Color::Rgb(249, 226, 175),         // Yellow
            danger: Color::Rgb(243, 139, 168),          // Red
            info: Color::Rgb(116, 199, 236),            // Sapphire
            accent: Color::Rgb(250, 179, 135),          // Peach
        }
    }

    /// Get palette by theme name
    pub fn from_theme(theme: ThemeName) -> Self {
        match theme {
            ThemeName::MaterialDesign => Self::material_design(),
            ThemeName::SolarizedDark => Self::solarized_dark(),
            ThemeName::Dracula => Self::dracula(),
            ThemeName::Nord => Self::nord(),
            ThemeName::TokyoNight => Self::tokyo_night(),
            ThemeName::Catppuccin => Self::catppuccin(),
        }
    }
}

/// Global theme state (atomic for thread-safety)
static CURRENT_THEME: AtomicUsize = AtomicUsize::new(0); // 0 = MaterialDesign

/// Theme manager - handles theme switching and access
pub struct ThemeManager;

impl ThemeManager {
    /// Get current theme name
    pub fn current() -> ThemeName {
        let idx = CURRENT_THEME.load(Ordering::Relaxed);
        ThemeName::all()
            .get(idx)
            .copied()
            .unwrap_or(ThemeName::MaterialDesign)
    }

    /// Set current theme
    pub fn set(theme: ThemeName) {
        let idx = ThemeName::all()
            .iter()
            .position(|&t| t == theme)
            .unwrap_or(0);
        CURRENT_THEME.store(idx, Ordering::Relaxed);
    }

    /// Get current color palette
    pub fn palette() -> ColorPalette {
        ColorPalette::from_theme(Self::current())
    }

    /// Cycle to next theme
    pub fn next() {
        let current_idx = CURRENT_THEME.load(Ordering::Relaxed);
        let themes = ThemeName::all();
        let next_idx = (current_idx + 1) % themes.len();
        CURRENT_THEME.store(next_idx, Ordering::Relaxed);
    }

    /// Set theme from string name
    pub fn set_by_name(name: &str) -> Result<ThemeName, String> {
        match ThemeName::from_str(name) {
            Some(theme) => {
                Self::set(theme);
                Ok(theme)
            }
            None => {
                let available = ThemeName::all()
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(format!(
                    "Unknown theme: '{}'. Available themes: {}",
                    name, available
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_names() {
        assert_eq!(ThemeName::MaterialDesign.as_str(), "material");
        assert_eq!(ThemeName::Dracula.display_name(), "Dracula");
    }

    #[test]
    fn test_theme_from_str() {
        assert_eq!(
            ThemeName::from_str("material"),
            Some(ThemeName::MaterialDesign)
        );
        assert_eq!(ThemeName::from_str("dracula"), Some(ThemeName::Dracula));
        assert_eq!(ThemeName::from_str("invalid"), None);
    }

    #[test]
    fn test_theme_manager() {
        ThemeManager::set(ThemeName::Dracula);
        assert_eq!(ThemeManager::current(), ThemeName::Dracula);

        ThemeManager::next();
        assert_eq!(ThemeManager::current(), ThemeName::Nord);
    }

    #[test]
    fn test_all_themes_valid() {
        for theme in ThemeName::all() {
            let palette = ColorPalette::from_theme(theme);
            // Just ensure they all create valid palettes
            assert!(matches!(palette.background, Color::Rgb(_, _, _)));
        }
    }
}

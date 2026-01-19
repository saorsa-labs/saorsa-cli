//! Theme system for customizing the TUI appearance
//!
//! Themes define colors, borders, and styling for the entire application.
//! They can be loaded from TOML files or constructed programmatically.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    /// Theme display name
    pub name: String,
    /// Color palette
    pub colors: ThemeColors,
    /// Border styling
    pub borders: BorderStyle,
}

/// Color palette for the theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    /// Main background color
    #[serde(with = "color_serde")]
    pub background: Color,
    /// Main foreground/text color
    #[serde(with = "color_serde")]
    pub foreground: Color,
    /// Accent color for highlights
    #[serde(with = "color_serde")]
    pub accent: Color,
    /// Selection/cursor color
    #[serde(with = "color_serde")]
    pub selection: Color,
    /// Error indicators
    #[serde(with = "color_serde")]
    pub error: Color,
    /// Warning indicators
    #[serde(with = "color_serde")]
    pub warning: Color,
    /// Success indicators
    #[serde(with = "color_serde")]
    pub success: Color,
    /// Muted/secondary text
    #[serde(with = "color_serde")]
    pub muted: Color,
}

/// Border style for panels and widgets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    /// Rounded corners (default)
    #[default]
    Rounded,
    /// Square corners
    Square,
    /// Double-line borders
    Double,
    /// No borders
    None,
}

/// Custom serde module for ratatui Color
mod color_serde {
    use ratatui::style::Color;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match color {
            Color::Reset => "reset".to_string(),
            Color::Black => "black".to_string(),
            Color::Red => "red".to_string(),
            Color::Green => "green".to_string(),
            Color::Yellow => "yellow".to_string(),
            Color::Blue => "blue".to_string(),
            Color::Magenta => "magenta".to_string(),
            Color::Cyan => "cyan".to_string(),
            Color::Gray => "gray".to_string(),
            Color::DarkGray => "darkgray".to_string(),
            Color::LightRed => "lightred".to_string(),
            Color::LightGreen => "lightgreen".to_string(),
            Color::LightYellow => "lightyellow".to_string(),
            Color::LightBlue => "lightblue".to_string(),
            Color::LightMagenta => "lightmagenta".to_string(),
            Color::LightCyan => "lightcyan".to_string(),
            Color::White => "white".to_string(),
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::Indexed(i) => format!("indexed:{}", i),
        };
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_color(&s).map_err(serde::de::Error::custom)
    }

    fn parse_color(s: &str) -> Result<Color, String> {
        match s.to_lowercase().as_str() {
            "reset" => Ok(Color::Reset),
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "gray" | "grey" => Ok(Color::Gray),
            "darkgray" | "darkgrey" => Ok(Color::DarkGray),
            "lightred" => Ok(Color::LightRed),
            "lightgreen" => Ok(Color::LightGreen),
            "lightyellow" => Ok(Color::LightYellow),
            "lightblue" => Ok(Color::LightBlue),
            "lightmagenta" => Ok(Color::LightMagenta),
            "lightcyan" => Ok(Color::LightCyan),
            "white" => Ok(Color::White),
            s if s.starts_with('#') => {
                let hex = s.trim_start_matches('#');
                if hex.len() != 6 {
                    return Err(format!("invalid hex color: {}", s));
                }
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| format!("invalid hex color: {}", s))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| format!("invalid hex color: {}", s))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| format!("invalid hex color: {}", s))?;
                Ok(Color::Rgb(r, g, b))
            }
            s if s.starts_with("indexed:") => {
                let idx = s
                    .trim_start_matches("indexed:")
                    .parse::<u8>()
                    .map_err(|_| format!("invalid indexed color: {}", s))?;
                Ok(Color::Indexed(idx))
            }
            _ => Err(format!("unknown color: {}", s)),
        }
    }
}

impl Theme {
    /// Creates the default dark theme
    pub fn dark() -> Self {
        Theme {
            name: "Dark".to_string(),
            colors: ThemeColors {
                background: Color::Rgb(30, 30, 46),    // Catppuccin base
                foreground: Color::Rgb(205, 214, 244), // Catppuccin text
                accent: Color::Rgb(137, 180, 250),     // Catppuccin blue
                selection: Color::Rgb(88, 91, 112),    // Catppuccin surface2
                error: Color::Rgb(243, 139, 168),      // Catppuccin red
                warning: Color::Rgb(249, 226, 175),    // Catppuccin yellow
                success: Color::Rgb(166, 227, 161),    // Catppuccin green
                muted: Color::Rgb(147, 153, 178),      // Catppuccin subtext1
            },
            borders: BorderStyle::Rounded,
        }
    }

    /// Creates a light theme
    pub fn light() -> Self {
        Theme {
            name: "Light".to_string(),
            colors: ThemeColors {
                background: Color::Rgb(239, 241, 245), // Catppuccin latte base
                foreground: Color::Rgb(76, 79, 105),   // Catppuccin latte text
                accent: Color::Rgb(30, 102, 245),      // Catppuccin latte blue
                selection: Color::Rgb(188, 192, 204),  // Catppuccin latte surface2
                error: Color::Rgb(210, 15, 57),        // Catppuccin latte red
                warning: Color::Rgb(223, 142, 29),     // Catppuccin latte yellow
                success: Color::Rgb(64, 160, 43),      // Catppuccin latte green
                muted: Color::Rgb(108, 111, 133),      // Catppuccin latte subtext1
            },
            borders: BorderStyle::Rounded,
        }
    }

    /// Creates a Nord theme
    pub fn nord() -> Self {
        Theme {
            name: "Nord".to_string(),
            colors: ThemeColors {
                background: Color::Rgb(46, 52, 64),    // Nord polar night
                foreground: Color::Rgb(236, 239, 244), // Nord snow storm
                accent: Color::Rgb(136, 192, 208),     // Nord frost
                selection: Color::Rgb(67, 76, 94),     // Nord polar night lighter
                error: Color::Rgb(191, 97, 106),       // Nord aurora red
                warning: Color::Rgb(235, 203, 139),    // Nord aurora yellow
                success: Color::Rgb(163, 190, 140),    // Nord aurora green
                muted: Color::Rgb(76, 86, 106),        // Nord polar night lightest
            },
            borders: BorderStyle::Rounded,
        }
    }

    /// Parses a theme from TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML string is malformed or contains invalid values.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Serializes the theme to TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "Dark");
        assert_eq!(theme.borders, BorderStyle::Rounded);
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.name, "Light");
    }

    #[test]
    fn test_nord_theme() {
        let theme = Theme::nord();
        assert_eq!(theme.name, "Nord");
    }

    #[test]
    fn test_theme_toml_roundtrip() {
        let original = Theme::dark();
        let toml_str = original.to_toml().expect("serialization should work");
        let parsed = Theme::from_toml(&toml_str).expect("parsing should work");
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_border_style_default() {
        assert_eq!(BorderStyle::default(), BorderStyle::Rounded);
    }

    #[test]
    fn test_theme_default_is_dark() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Dark");
    }

    #[test]
    fn test_hex_color_parsing() {
        let toml_str = r##"
            name = "Test"
            borders = "rounded"
            
            [colors]
            background = "#1e1e2e"
            foreground = "#cdd6f4"
            accent = "#89b4fa"
            selection = "#585b70"
            error = "#f38ba8"
            warning = "#f9e2af"
            success = "#a6e3a1"
            muted = "#9399b2"
        "##;
        let theme = Theme::from_toml(toml_str).expect("should parse hex colors");
        assert_eq!(theme.colors.background, Color::Rgb(30, 30, 46));
    }

    #[test]
    fn test_named_color_parsing() {
        let toml_str = r##"
            name = "Simple"
            borders = "square"
            
            [colors]
            background = "black"
            foreground = "white"
            accent = "blue"
            selection = "gray"
            error = "red"
            warning = "yellow"
            success = "green"
            muted = "darkgray"
        "##;
        let theme = Theme::from_toml(toml_str).expect("should parse named colors");
        assert_eq!(theme.colors.background, Color::Black);
        assert_eq!(theme.colors.foreground, Color::White);
        assert_eq!(theme.borders, BorderStyle::Square);
    }

    #[test]
    fn test_indexed_color_parsing() {
        let toml_str = r##"
            name = "Indexed"
            borders = "none"
            
            [colors]
            background = "indexed:0"
            foreground = "indexed:15"
            accent = "indexed:4"
            selection = "indexed:8"
            error = "indexed:1"
            warning = "indexed:3"
            success = "indexed:2"
            muted = "indexed:7"
        "##;
        let theme = Theme::from_toml(toml_str).expect("should parse indexed colors");
        assert_eq!(theme.colors.background, Color::Indexed(0));
        assert_eq!(theme.colors.foreground, Color::Indexed(15));
        assert_eq!(theme.borders, BorderStyle::None);
    }

    #[test]
    fn test_all_border_styles() {
        for (toml_val, expected) in [
            ("rounded", BorderStyle::Rounded),
            ("square", BorderStyle::Square),
            ("double", BorderStyle::Double),
            ("none", BorderStyle::None),
        ] {
            let toml_str = format!(
                r##"
                name = "Test"
                borders = "{}"
                
                [colors]
                background = "black"
                foreground = "white"
                accent = "blue"
                selection = "gray"
                error = "red"
                warning = "yellow"
                success = "green"
                muted = "darkgray"
            "##,
                toml_val
            );
            let theme = Theme::from_toml(&toml_str).expect("should parse border style");
            assert_eq!(theme.borders, expected);
        }
    }

    #[test]
    fn test_light_theme_roundtrip() {
        let original = Theme::light();
        let toml_str = original.to_toml().expect("serialization should work");
        let parsed = Theme::from_toml(&toml_str).expect("parsing should work");
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_nord_theme_roundtrip() {
        let original = Theme::nord();
        let toml_str = original.to_toml().expect("serialization should work");
        let parsed = Theme::from_toml(&toml_str).expect("parsing should work");
        assert_eq!(original, parsed);
    }
}

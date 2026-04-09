use std::str::FromStr;
use thiserror::Error;

/// Defines the character set used to represent pixels in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetMode {
    #[default]
    /// High-resolution mode using half-blocks (▀/▄).
    Ansi,
    /// Flexible mode using either full or half blocks based on the render style.
    Unicode,
    /// Maximum density mode using 2x4 Braille dot patterns.
    Braille,
    /// A small 4-character ramp ( ░▒▓█) for a "faded" or shaded look.
    Fade,
    /// Traditional 92-character density ramp for classic ASCII art.
    Ascii,

    Kanji,

    Chinese,

    Sixel,
}

impl FromStr for CharsetMode {
    type Err = RenderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ansi" | "block" => Ok(Self::Ansi),
            "unicode" | "uni" => Ok(Self::Unicode),
            "braille" | "brl" => Ok(Self::Braille),
            "fade" | "grayscale" => Ok(Self::Fade),
            "kanji" | "jp" => Ok(Self::Kanji),
            "chinese" | "zh" | "hanzi" => Ok(Self::Chinese),
            "ascii" => Ok(Self::Ascii),
            _ => Err(RenderError::InvalidCharset(s.to_string())),
        }
    }
}

/// Aesthetic density settings for `--style ascii`
#[derive(Clone, Copy, Debug, Default)]
pub enum Density {
    #[default]
    Medium,
    Light,
    Heavy,
}

impl FromStr for Density {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We normalize to lowercase so "Medium", "medium", and "MEDIUM" all work
        match s.to_lowercase().as_str() {
            "medium" => Ok(Self::Medium),
            "light" => Ok(Self::Light),
            "heavy" => Ok(Self::Heavy),
            _ => Err(format!(
                "invalid density: '{s}'. (valid: light, medium, heavy)"
            )),
        }
    }
}

/// Combines physical character choice with layout logic (like full-width vs half-width).
#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
    /// If true, uses double-width characters (██) to force a 1:1 pixel aspect ratio.
    pub(crate) full: bool,
    pub(crate) density: Density,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            full: false,
            density: Density::Medium,
        }
    }
}

impl RenderStyle {
    /// Returns `true` if "full-block" rendering is enabled.
    ///
    /// # Note
    /// This setting is a specialty of [`CharsetMode::Unicode`]. When enabled,
    /// characters are doubled (e.g., `██`) to maintain a 1:1 square aspect ratio
    /// in terminal fonts. It has no effect on other charset modes like Braille.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.full
    }

    /// Returns the current [`Density`] level.
    ///
    /// # Note
    /// This setting is a specialty of [`CharsetMode::Ascii`]. It determines
    /// the complexity of the character ramp used to represent grayscale values.
    /// For all other modes, this value is ignored.
    #[must_use]
    pub const fn density(&self) -> Density {
        self.density
    }
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Invalid charset mode: {0}")]
    InvalidCharset(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(String),

    #[error("Invalid density: {0}. (valid: light, medium, heavy)")]
    InvalidDensity(String),
}

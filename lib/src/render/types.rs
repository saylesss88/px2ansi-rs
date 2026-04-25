use std::str::FromStr;
use thiserror::Error;

use crate::color::terminal_supports_truecolor;

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

    /// Uses Japanese Kanji characters to represent visual density.
    /// Often used for a "Matrix-style" or highly stylized terminal aesthetic.
    Kanji,

    /// Uses Chinese characters to represent visual density.
    /// Provides a unique texture and complex character patterns for image rendering.
    Chinese,

    /// A high-performance bitmap protocol that renders actual image pixels
    /// directly in the terminal (requires a compatible terminal emulator).
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

/// Aesthetic density settings for the ASCII rendering style.
///
/// This determines the character set "ramp" used to map image brightness
/// to character visual weight.
#[derive(Clone, Copy, Debug, Default)]
pub enum Density {
    /// A balanced character ramp providing good contrast and detail.
    #[default]
    Medium,
    /// A sparse character set that uses thinner, lighter characters for a minimalist look.
    Light,
    /// A dense character set that uses bold, "heavy" characters to maximize coverage.
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
    pub(crate) dither: bool,
    #[allow(dead_code)]
    pub(crate) wide: bool,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            full: false,
            density: Density::Medium,
            dither: false,
            wide: false,
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

    /// Returns `true` if "dithering" is enabled.
    #[must_use]
    pub const fn dither(&self) -> bool {
        self.dither
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

/// Errors that can occur during the image rendering or conversion process.
#[derive(Error, Debug)]
pub enum RenderError {
    /// Returned when a requested character set mode is not recognized.
    #[error("Invalid charset mode: {0}")]
    InvalidCharset(String),
    /// Errors originating from underlying disk I/O or terminal stream operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Errors occurring during image decoding, resizing, or pixel manipulation.
    #[error("Image processing error: {0}")]
    Image(String),
    /// Returned when an unsupported density string is provided via configuration.
    #[error("Invalid density: {0}. (valid: light, medium, heavy)")]
    InvalidDensity(String),
    /// Errors occurring during font loading or glyph rasterization via fontdue.
    #[error("Font error: {0}")]
    Font(String),
    /// Returned when ANSI input parses to zero cells, producing nothing to render.
    #[error("No cells to render")]
    EmptyCells,
    /// Errors occurring during JSON serialization of the image index.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Specifies the color depth and encoding used for terminal output.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Uses 24-bit `TrueColor` (RGB) ANSI escape sequences.
    /// Supported by most modern terminal emulators.
    #[default]
    TrueColor,
    /// Uses xterm-compatible 256-color escape sequences.
    /// Colors are quantized using the Oklab color space for better perceptual accuracy.
    Ansi256,
    /// Disables all color escape sequences, producing plain text output.
    None,
}

impl ColorMode {
    /// Auto-detect the best color mode the terminal supports.
    #[must_use]
    pub fn detect() -> Self {
        if terminal_supports_truecolor() {
            Self::TrueColor
        } else {
            Self::Ansi256
        }
    }
}
impl std::str::FromStr for ColorMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "truecolor" => Ok(Self::TrueColor),
            "ansi256" | "256" => Ok(Self::Ansi256),
            "none" => Ok(Self::None),
            _ => Err(format!("'{s}' is not a valid color mode")),
        }
    }
}

use clap::ValueEnum;
use std::str::FromStr;

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
}

impl FromStr for CharsetMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ansi" | "block" => Ok(Self::Ansi),
            "unicode" | "uni" => Ok(Self::Unicode),
            "braille" | "brl" => Ok(Self::Braille),
            "fade" | "grayscale" => Ok(Self::Fade),
            "kanji" | "jp" => Ok(Self::Kanji),
            "chinese" | "zh" | "hanzi" => Ok(Self::Chinese),
            "ascii" => Ok(Self::Ascii),
            _ => anyhow::bail!(
                "Invalid charset '{s}'. Use: ansi, unicode, braille, fade, ascii, kanji"
            ),
        }
    }
}

/// Aesthetic density settings for the rendered output.
/// WIP
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Density {
    #[default]
    Medium,
    Light,
    Heavy,
}

/// Combines physical character choice with layout logic (like full-width vs half-width).
#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
    /// If true, uses double-width characters (██) to force a 1:1 pixel aspect ratio.
    pub full: bool,
    pub density: Density,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            full: false,
            density: Density::Medium,
        }
    }
}

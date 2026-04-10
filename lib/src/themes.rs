//! Background themes for rasterized ANSI art.
//!
//! Themes define the background color used when rasterizing terminal art
//! to PNG images.

use image::Rgba;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A background color theme for rasterization.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RasterTheme {
    /// Tokyo Night theme (#1A1B26)
    #[default]
    TokyoNight,
    /// Dracula theme (#282A36)
    Dracula,
    /// Nord theme (#2E3440)
    Nord,
    /// Gruvbox Dark theme (#282828)
    GruvboxDark,
    /// One Dark theme (#1E1E1E)
    OneDark,
    /// Solarized Dark theme (#002B36)
    SolarizedDark,
    /// Pure Black (#000000)
    Black,
    /// Pure White (#FFFFFF)
    White,
}

impl RasterTheme {
    /// Returns the RGBA color for this theme.
    #[must_use]
    pub const fn color(self) -> Rgba<u8> {
        match self {
            Self::TokyoNight => Rgba([26, 27, 38, 255]),   // #1A1B26
            Self::Dracula => Rgba([40, 42, 54, 255]),      // #282A36
            Self::Nord => Rgba([46, 52, 64, 255]),         // #2E3440
            Self::GruvboxDark => Rgba([40, 40, 40, 255]),  // #282828
            Self::OneDark => Rgba([30, 30, 30, 255]),      // #1E1E1E
            Self::SolarizedDark => Rgba([0, 43, 54, 255]), // #002B36
            Self::Black => Rgba([0, 0, 0, 255]),           // #000000
            Self::White => Rgba([255, 255, 255, 255]),     // #FFFFFF
        }
    }

    /// Returns the hex color code for display purposes.
    #[must_use]
    pub const fn hex(self) -> &'static str {
        match self {
            Self::TokyoNight => "#1A1B26",
            Self::Dracula => "#282A36",
            Self::Nord => "#2E3440",
            Self::GruvboxDark => "#282828",
            Self::OneDark => "#1E1E1E",
            Self::SolarizedDark => "#002B36",
            Self::Black => "#000000",
            Self::White => "#FFFFFF",
        }
    }
}

// impl Default for RasterTheme {
//     fn default() -> Self {
//         Self::TokyoNight
//     }
// }

impl FromStr for RasterTheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "").as_str() {
            "tokyonight" => Ok(Self::TokyoNight),
            "dracula" => Ok(Self::Dracula),
            "nord" => Ok(Self::Nord),
            "gruvboxdark" => Ok(Self::GruvboxDark),
            "onedark" => Ok(Self::OneDark),
            "solarizeddark" => Ok(Self::SolarizedDark),
            "black" => Ok(Self::Black),
            "white" => Ok(Self::White),
            _ => Err(format!(
                "invalid theme: '{s}'. (valid: tokyo-night, dracula, nord, gruvbox-dark, one-dark, solarized-dark, black, white)"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_themes() {
        assert_eq!("tokyo-night".parse(), Ok(RasterTheme::TokyoNight));
        assert_eq!("dracula".parse(), Ok(RasterTheme::Dracula));
        assert_eq!("NORD".parse(), Ok(RasterTheme::Nord));
    }
}

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Specifies the visual style used to render output.
///
/// This determines which character sets or protocols are used to represent
/// image data or visual elements in the terminal.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderStylePreset {
    /// Standard ANSI color blocks and basic styling.
    #[default]
    Ansi,
    /// Uses Unicode box-drawing and geometric characters for higher resolution.
    Unicode,
    /// Uses 2x4 dot Braille patterns to significantly increase effective resolution.
    Braille,
    /// Uses varying character densities to simulate color gradients or shadows.
    Fade,
    /// Uses standard 7-bit ASCII characters (e.g., #, @, ., :).
    Ascii,
    /// Uses the full block character (U+2588) for solid color rendering.
    FullBlock,
    /// Uses high-density Unicode characters for a detailed grayscale effect.
    Dense,
    /// Renders using Kanji characters; often used for stylistic "matrix" effects.
    Kanji,
    /// Renders using Chinese characters for unique visual textures.
    Chinese,
    /// Uses the SIXEL bitmap protocol for true high-resolution graphics in supported terminals.
    Sixel,
}

impl FromStr for RenderStylePreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "").as_str() {
            "ansi" => Ok(Self::Ansi),
            "unicode" => Ok(Self::Unicode),
            "braille" => Ok(Self::Braille),
            "fade" => Ok(Self::Fade),
            "ascii" => Ok(Self::Ascii),
            "kanji" => Ok(Self::Kanji),
            "chinese" => Ok(Self::Chinese),
            "fullblock" => Ok(Self::FullBlock),
            "dense" => Ok(Self::Dense),
            "sixel" => Ok(Self::Sixel),
            _ => Err(format!(
                "invalid style: '{s}'. (valid: ansi, unicode, braille, fade, ascii, kanji, chinese, full-block, dense)"
            )),
        }
    }
}

/// Strategies used for resampling images when resizing.
///
/// Different filters balance the trade-off between processing speed and
/// visual quality (aliasing, sharpness, and blurring).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFilter {
    /// Nearest Neighbor interpolation.
    ///
    /// Fast but pixelated; preserves hard edges, making it ideal for pixel art.
    Nearest,
    /// Linear interpolation (Triangle filter).
    ///
    /// Provides a good balance between speed and quality for downscaling.
    Triangle,
    /// Catmull-Rom cubic interpolation.
    ///
    /// A sharp cubic filter that produces crisp results without excessive ringing.
    CatmullRom,
    /// Gaussian blurring filter.
    ///
    /// Useful for reducing noise or creating a softer look during resizing.
    Gaussian,
    /// Lanczos windowed sinc interpolation (size 3).
    ///
    /// Highest quality resampling; reduces aliasing significantly but is computationally expensive.
    Lanczos3,
}

impl FromStr for ResizeFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "").as_str() {
            "nearest" => Ok(Self::Nearest),
            "triangle" => Ok(Self::Triangle),
            "catmullrom" => Ok(Self::CatmullRom),
            "gaussian" => Ok(Self::Gaussian),
            "lanczos3" => Ok(Self::Lanczos3),
            _ => Err(format!(
                "invalid filter: '{s}'. (valid: nearest, triangle, catmull-rom, gaussian, lanczos3)"
            )),
        }
    }
}

impl From<ResizeFilter> for image::imageops::FilterType {
    fn from(f: ResizeFilter) -> Self {
        match f {
            ResizeFilter::Nearest => Self::Nearest,
            ResizeFilter::Triangle => Self::Triangle,
            ResizeFilter::CatmullRom => Self::CatmullRom,
            ResizeFilter::Gaussian => Self::Gaussian,
            ResizeFilter::Lanczos3 => Self::Lanczos3,
        }
    }
}

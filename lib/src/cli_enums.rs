use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderStylePreset {
    #[default]
    Ansi,
    Unicode,
    Braille,
    Fade,
    Ascii,
    FullBlock,
    Dense,
    Kanji,
    Chinese,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFilter {
    /// Nearest Neighbor (Best for pixel art)
    Nearest,
    /// Linear interpolation
    Triangle,
    /// Sharp cubic filter
    CatmullRom,
    /// Blurry cubic filter
    Gaussian,
    /// High-quality resampling (Slowest)
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

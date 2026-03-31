use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
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

use clap::{Parser, ValueEnum};
use image::imageops::FilterType;

#[derive(Parser)]
#[command(
    name = "px2ansi",
    version,
    about = "Convert pixel art to ANSI terminal art"
)]
pub struct Cli {
    /// Input image file
    pub filename: String,

    /// Output file (optional). If not provided, prints to stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Force a specific width (disables auto-resizing to terminal)
    #[arg(long)]
    pub width: Option<u32>,

    /// Resize filter to use (default: lanczos3).
    /// Use 'nearest' for pixel art to keep hard edges.
    #[arg(long, value_enum, default_value_t = ResizeFilter::Lanczos3)]
    pub filter: ResizeFilter,
}
// 1. Define an Enum for the CLI argument
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
pub enum ResizeFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

// 2. Add helper to convert CLI enum to image::FilterType
impl From<ResizeFilter> for FilterType {
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

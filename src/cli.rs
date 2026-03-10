use clap::{Parser, Subcommand, ValueEnum};
use image::imageops::FilterType;

#[derive(Parser)]
#[command(name = "px2ansi", version, about = "Pixel art tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress performance metrics and status messages
    #[arg(short, long, global = true)]
    pub silent: bool,
}
#[derive(Subcommand)]
pub enum Commands {
    /// Convert a single image to ANSI/Unicode
    Convert {
        /// Input image file
        filename: String,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<String>,

        /// Output mode:
        /// - 'ansi': Highest detail. Uses half-blocks to fit 2 pixels per cell.
        /// - 'unicode': Retro look. Uses '██' to represent 1 pixel as a square.
        #[arg(short, long, default_value = "ansi")]
        mode: String,

        #[arg(long)]
        full: bool,

        /// Force a specific width
        #[arg(long)]
        width: Option<u32>,

        /// Resize filter
        #[arg(long, value_enum, default_value_t = ResizeFilter::Lanczos3)]
        filter: ResizeFilter,
    },
    /// Create a JSON index of a directory
    Index {
        /// Directory to scan
        dir: String,
        /// Path to save the JSON index
        #[arg(short, long, default_value = "index.json")]
        output: String,
    },

    Show {
        /// The name of the image to show. Use 'random' to pick a surprise sprite!
        #[arg(help = "The name of the image (e.g., 'charizard') or 'random'")]
        name: String,
        /// Path to the index.json file
        #[arg(short, long, default_value = "index.json")]
        index: String,
        /// Output mode (ansi, unicode)
        #[arg(short, long, default_value = "ansi")]
        mode: String,

        /// Use double-width full blocks (██) for a retro, square look
        #[arg(long)]
        full: bool,

        #[arg(short, long, value_enum, default_value_t = ResizeFilter::Nearest)]
        filter: ResizeFilter,
    },
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

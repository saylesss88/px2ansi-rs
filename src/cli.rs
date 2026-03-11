use clap::{Parser, Subcommand, ValueEnum};
use clap_complete;
use image::imageops::FilterType;

#[derive(Parser)]
#[command(name = "px2ansi-rs", version, about = "Pixel art tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Show timing and execution metadata
    #[arg(short = 'l', long = "latency", global = true)] // Added global = true here
    pub latency: bool,
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
        /// - 'unicode': Uses half-blocks by default, opt-in for full block mode `--full`
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
        #[arg(default_value = "random")]
        name: String,
        /// Path to the index.json file
        #[arg(long, default_value = "index.json")]
        index: String,
        /// Output mode (ansi, unicode)
        #[arg(short, long, default_value = "ansi")]
        mode: String,

        /// Use double-width full blocks (██) for a retro, square look
        #[arg(long)]
        full: bool,

        #[arg(short, long, value_enum, default_value_t = ResizeFilter::Nearest)]
        filter: ResizeFilter,

        #[arg(short = 'i', long)]
        interactive: bool,
    },
    List {
        /// Path to the JSON index file
        #[arg(long, default_value = "index.json")]
        index: String,

        /// Number of entries to show (omit to show all)
        #[arg(short, long)]
        count: Option<usize>,
    },
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
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

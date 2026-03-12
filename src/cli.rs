use clap::{Parser, Subcommand, ValueEnum};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "px2ansi-rs", version, about = "Pixel art tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Show timing and execution metadata
    #[arg(short = 'l', long = "latency", global = true)]
    pub latency: bool,

    #[arg(short = 'I', long = "index", global = true)]
    pub index: Option<String>,
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
        #[arg(short, long)]
        mode: Option<String>,

        #[arg(long)]
        full: Option<bool>,

        /// Force a specific width
        #[arg(long)]
        width: Option<u32>,

        /// Resize filter
        #[arg(
            short,
            long,
            value_enum,
            help = "The resampling filter to use",
            long_help = "Nearest is best for pixel art. Lanczos3 is best for high-resolution images."
        )]
        filter: Option<ResizeFilter>,
    },
    /// Create a JSON index of a directory
    Index {
        /// Directory to scan
        dir: String,
        /// Path to save the JSON index
        #[arg(short, long)]
        output: Option<String>,
    },
    Show {
        /// The name of the image to show. Use 'random' to pick a surprise sprite!
        #[arg(default_value = "random")]
        name: String,
        /// Path to the index.json file
        // #[arg(short, long, default_value = "index.json")]
        // #[arg(short = 'I', long)]
        // index: Option<String>,
        /// Output mode (ansi, unicode)
        #[arg(short, long)]
        mode: Option<String>,

        /// Use double-width full blocks (██) for a retro, square look
        #[arg(long)]
        full: Option<bool>,

        #[arg(short, long, value_enum)]
        filter: Option<ResizeFilter>,

        #[arg(short = 'i', long)]
        interactive: bool,
    },
    List {
        // /// Path to the JSON index file
        // #[arg(short = 'I', long, default_value = "index.json")]
        // index: String,
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")] // For the config file (TOML)
#[clap(rename_all = "kebab-case")] // For the CLI flags
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

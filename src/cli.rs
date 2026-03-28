use clap::{Parser, Subcommand};
use px2ansi_rs::{RenderStylePreset, ResizeFilter};
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
    /// Convert a single image to ANSI/Unicode/Fade/Braille/Kanji/Full-block/Ascii
    Convert {
        /// Input image file
        filename: String,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<String>,

        #[arg(long, value_enum)]
        style: Option<RenderStylePreset>,

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
    /// Display a sprite from the index
    Show {
        /// The name of the image to show. Use 'random' to pick a surprise sprite!
        #[arg(default_value = "random")]
        name: String,

        #[arg(long, value_enum)]
        style: Option<RenderStylePreset>,

        #[arg(short, long, value_enum)]
        filter: Option<ResizeFilter>,

        #[arg(short = 'i', long)]
        interactive: bool,
    },
    /// List entries in the index
    List {
        /// Number of entries to show (omit to show all)
        #[arg(short, long)]
        count: Option<usize>,
    },
    /// Generate shell completions and add to your shell config.
    /// Example: `px2ansi-rs completions bash >> ~/.bashrc`
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

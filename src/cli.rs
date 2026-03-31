//! Command-line interface definition for `px2ansi-rs`.
//!
//! This module defines the `Cli` struct and `Commands` enum using `clap`.
//! It handles the mapping between user input and the internal data structures
//! used by the rendering and indexing engines.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::aot::Shell;

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
        input: PathBuf,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Save a rasterized preview instead of terminal escape codes
        #[arg(short = 'O', long = "output-image")]
        output_image: Option<PathBuf>,

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
        dir: PathBuf,
        /// Path to save the JSON index
        #[arg(short, long)]
        output: Option<PathBuf>,
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
    #[command(arg_required_else_help = true)]
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

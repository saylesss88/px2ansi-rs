//! Command-line interface definition for `px2ansi-rs`.
//!
//! This module defines the `Cli` struct and `Commands` enum using `clap`.
//! It handles the mapping between user input and the internal data structures
//! used by the rendering and indexing engines.

use clap::{Parser, Subcommand};
use clap_complete::aot::Shell;

use std::path::PathBuf;

use crate::{RenderStylePreset, ResizeFilter, render::Density};

#[derive(Parser)]
#[command(
    name = "px2ansi-rs",
    version,
    about = "High-fidelity terminal art engine and asset manager",
    long_about = "px2ansi-rs converts images to ANSI terminal art using multiple rendering \
                  styles including half-blocks, braille, ASCII density ramps, and CJK characters. \
                  It includes an image indexer with fuzzy search, interactive TUI browsing, \
                  and can export rendered art as PNG via a built-in rasterizer."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Show timing and execution metadata
    #[arg(short = 'l', long = "latency", global = true)]
    pub latency: bool,

    /// Path to the JSON index file (overrides config file setting)
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

        /// Character density for --style ascii and --style fade.
        /// Light: sparse 30-char ramp, Medium: full 92-char ramp, Heavy: block-heavy ramp.
        #[arg(long, value_enum)]
        density: Option<Density>,

        /// Disable ANSI color output (monochrome). Applies to ascii, fade, braille, kanji, and chinese modes.
        #[arg(long = "no-color")]
        no_color: bool,

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

        /// Disable ANSI color output (monochrome). Applies to ascii, fade, braille, kanji, and chinese modes.
        #[arg(long = "no-color")]
        no_color: bool,

        #[arg(long, value_enum)]
        density: Option<Density>,

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

// #![deny(missing_docs)]
//
pub mod cli;
pub mod commands;
pub mod config;

use colored::Colorize;

// --- RE-EXPORTS (The "Lobby") ---
pub use cli::{Cli, Commands};
pub use commands::convert::ConvertCmd;
pub use commands::handle_command;
pub use commands::index::IndexCmd;
pub use commands::list::ListCmd;
pub use commands::show::ShowCmd;
pub use config::Config;

pub use px2ansi::{Density, RenderOptions, RenderStylePreset, ResizeFilter};
use std::path::PathBuf;

/// A unified view of program settings, resolved from both CLI flags and config files.
#[derive(Debug)]
pub struct ResolvedOptions {
    /// Whether to print performance timing data.
    pub latency: bool,
    /// The final path to the image index JSON.
    pub index_path: PathBuf,
}

impl ResolvedOptions {
    /// Merges [Cli] flags and [Config] settings, prioritizing CLI input.
    ///
    /// # Examples
    ///
    /// If a user passes `--index local.json` via CLI, it will override
    /// whatever is defined in the configuration file.
    pub fn from_cli_and_config(cli: &Cli, cfg: &Config) -> Self {
        Self {
            latency: cli.latency || cfg.latency,
            index_path: cli
                .index
                .as_deref()
                .map_or_else(|| PathBuf::from(&cfg.index), PathBuf::from),
        }
    }
}

pub fn build_render_options(
    style: Option<RenderStylePreset>,
    density: Option<Density>,
    width: Option<u32>,
    filter: Option<ResizeFilter>,
    no_color: bool,
) -> RenderOptions {
    let mut builder = RenderOptions::builder();

    if let Some(s) = style {
        builder.preset(s);
    }
    if let Some(d) = density {
        builder.density(d);
    }
    if let Some(w) = width {
        builder.width(w);
    }
    if let Some(f) = filter {
        builder.filter(f);
    }
    if no_color {
        builder.color(false);
    }

    builder.build()
}

/// Prints a colored performance summary to stderr.
pub fn print_summary(duration: std::time::Duration) {
    eprintln!(
        "\n{} took {}ms",
        "Execution".bright_blue().bold(),
        duration.as_millis()
    );
}

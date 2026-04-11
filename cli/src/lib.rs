pub mod cli;
pub mod commands;
pub mod config;
pub mod output;
pub mod render;

// Re-export types
pub use crate::commands::Command;
pub use crate::render::build_render_options;
pub use cli::{Cli, Commands};
pub use commands::convert::ConvertCmd;
pub use commands::index::IndexCmd;
pub use commands::list::ListCmd;
pub use commands::show::ShowCmd;
pub use config::Config;
pub use px2ansi::{Density, RenderOptions, RenderStylePreset, ResizeFilter};

use std::path::PathBuf;

#[derive(Debug)]
pub struct ResolvedOptions {
    pub latency: bool,
    pub index_path: PathBuf,
}

impl ResolvedOptions {
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

#![allow(clippy::multiple_crate_versions)]
mod commands;
mod config;

use crate::commands::convert::ConvertCmd;
use crate::commands::index::IndexCmd;
use crate::commands::list::ListCmd;
use crate::commands::show::ShowCmd;
use crate::commands::{Command, handle_command};
use crate::config::Config;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use px2ansi_rs::{Cli, Commands, RenderOptions};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<()> {
    let start = Instant::now();
    let cli = Cli::parse();
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "px2ansi-rs", &mut std::io::stdout());
        return Ok(());
    }
    let cfg: Config = confy::load("px2ansi-rs", None)?;
    let opts = ResolvedOptions::from_cli_and_config(&cli, &cfg);
    let cmd = match cli.command {
        Commands::Convert {
            input,
            output,
            output_image,
            width,
            filter,
            style,
            density,
            no_color,
        } => {
            let render = RenderOptions::from_cli(style, density, width, filter, no_color)?;
            let output_image = output_image.or_else(|| cfg.output_image.as_ref().map(Into::into));
            Command::Convert(ConvertCmd {
                input,
                output,
                output_image,
                render,
            })
        }
        Commands::Index { dir, output } => {
            let output = output.map_or_else(|| opts.index_path.clone(), PathBuf::from);
            Command::Index(IndexCmd { dir, output })
        }
        Commands::List { count } => Command::List(ListCmd {
            index_path: opts.index_path,
            count,
        }),
        Commands::Show {
            name,
            filter,
            interactive,
            style,
            density,
            no_color,
        } => {
            let render = RenderOptions::from_cli(style, density, None, filter, no_color)?;
            Command::Show(ShowCmd {
                name,
                index_path: opts.index_path,
                render,
                interactive,
            })
        }
        Commands::Completions { .. } => unreachable!(),
    };
    handle_command(&cmd)?;
    if opts.latency {
        print_summary(start.elapsed());
    }
    Ok(())
}

#[derive(Debug)]
struct ResolvedOptions {
    latency: bool,
    index_path: PathBuf,
}

impl ResolvedOptions {
    fn from_cli_and_config(cli: &Cli, cfg: &Config) -> Self {
        Self {
            latency: cli.latency || cfg.latency,
            index_path: cli
                .index
                .as_deref()
                .map_or_else(|| PathBuf::from(&cfg.index), PathBuf::from),
        }
    }
}

fn print_summary(duration: std::time::Duration) {
    eprintln!(
        "\n{} took {}ms",
        "Execution".bright_blue().bold(),
        duration.as_millis()
    );
}

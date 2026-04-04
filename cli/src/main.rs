//! # px2ansi-rs CLI
//!
//! The binary entry point for `px2ansi-rs`. This module handles the command-line
//! interface, configuration merging, and execution flow.
//!
//! ## Execution Flow
//! 1. **Parse CLI**: Captures raw arguments using `clap`.
//! 2. **Load Config**: Loads persistent settings (like default index paths) via `confy`.
//! 3. **Resolve Options**: Merges CLI flags and Config settings into [`ResolvedOptions`].
//! 4. **Build Command**: Maps the CLI subcommand to a specific [`Command`] logic block.
//! 5. **Execute**: Calls the specialized `run()` method for the selected command.
//!
//! ## Examples
//!
//! ```bash
//! # Convert an image using the ANSI block style
//! px2ansi-rs convert input.png --style ansi
//!
//! # Show a previously indexed image by name
//! px2ansi-rs show "my_cool_avatar"

#![deny(missing_docs)]

mod cli;
mod commands;
mod config;

use crate::{
    cli::{Cli, Commands},
    commands::{
        Command, convert::ConvertCmd, handle_command, index::IndexCmd, list::ListCmd, show::ShowCmd,
    },
    config::Config,
};
use px2ansi::{Density, RenderStylePreset, ResizeFilter};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use px2ansi::RenderOptions;

use std::{path::PathBuf, time::Instant};

/// The entry point for the `px2ansi-rs` CLI tool.
///
/// It handles timing, configuration loading, and command delegation.
fn main() -> Result<()> {
    let start = Instant::now();
    let cli = Cli::parse();

    // Special handling for shell completions as they require the CommandFactory
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "px2ansi-rs", &mut std::io::stdout());
        return Ok(());
    }

    let cfg: Config = confy::load("px2ansi-rs", None)?;
    let opts = ResolvedOptions::from_cli_and_config(&cli, &cfg);

    // Convert the raw CLI args into a domain-specific Command
    let cmd = build_command(cli, &cfg, &opts)?;

    handle_command(&cmd)?;

    if opts.latency {
        print_summary(start.elapsed());
    }

    Ok(())
}

/// Translates the raw [Cli] arguments into a specific [Command] variant.
///
/// This function acts as a bridge between the command-line interface and
/// the internal command processing logic, merging user flags with
/// system configuration.
///
/// # Errors
///
/// Returns an error if the rendering options cannot be validated (e.g., invalid width).
fn build_command(cli: Cli, cfg: &Config, opts: &ResolvedOptions) -> Result<Command> {
    match cli.command {
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
            let render = build_render_options(style, density, width, filter, no_color);
            let output_image = output_image.or_else(|| cfg.output_image.as_ref().map(Into::into));
            Ok(Command::Convert(ConvertCmd {
                input,
                output,
                output_image,
                render,
            }))
        }
        Commands::Index { dir, output } => {
            let output = output.map_or_else(|| opts.index_path.clone(), PathBuf::from);
            Ok(Command::Index(IndexCmd { dir, output }))
        }
        Commands::List { count } => Ok(Command::List(ListCmd {
            index_path: opts.index_path.clone(),
            count,
        })),
        Commands::Show {
            name,
            filter,
            interactive,
            style,
            density,
            no_color,
        } => {
            let render = build_render_options(style, density, None, filter, no_color);
            Ok(Command::Show(ShowCmd {
                name,
                index_path: opts.index_path.clone(),
                render,
                interactive,
            }))
        }
        Commands::Completions { .. } => unreachable!(),
    }
}

/// A unified view of program settings, resolved from both CLI flags and config files.
#[derive(Debug)]
struct ResolvedOptions {
    /// Whether to print performance timing data.
    latency: bool,
    /// The final path to the image index JSON.
    index_path: PathBuf,
}

impl ResolvedOptions {
    /// Merges [Cli] flags and [Config] settings, prioritizing CLI input.
    ///
    /// # Examples
    ///
    /// If a user passes `--index local.json` via CLI, it will override
    /// whatever is defined in the configuration file.
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

fn build_render_options(
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
fn print_summary(duration: std::time::Duration) {
    eprintln!(
        "\n{} took {}ms",
        "Execution".bright_blue().bold(),
        duration.as_millis()
    );
}

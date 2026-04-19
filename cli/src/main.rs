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

use px2ansi_rs::{
    Cli, Command, Commands, Config, ConvertCmd, IndexCmd, ListCmd, ResolvedOptions, ShowCmd,
    commands, output, render,
};

use clap::{CommandFactory, Parser};

use anyhow::Result;

use std::{io::Write, path::PathBuf, time::Instant};

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

    let out = std::io::stdout();
    let mut lock = out.lock();

    let mut writer = std::io::BufWriter::with_capacity(128 * 1024, &mut lock);

    commands::handle_command(&cmd, &mut writer)?;

    writer.flush()?;

    if opts.latency {
        output::print_summary(start.elapsed());
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
            // no_color,
            raster_theme,
            color_mode,
        } => {
            let render_opts =
                render::build_render_options(style, density, width, filter, color_mode);

            let output_image = output_image.or_else(|| cfg.output_image.as_ref().map(Into::into));

            Ok(Command::Convert(ConvertCmd {
                input,
                output,
                output_image,
                render: render_opts,
                raster_theme: raster_theme.unwrap_or(cfg.raster_theme),
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
            // no_color,
            color_mode,
        } => {
            let render_opts =
                render::build_render_options(style, density, None, filter, color_mode);

            Ok(Command::Show(ShowCmd {
                name,
                index_path: opts.index_path.clone(),
                render: render_opts,
                interactive,
            }))
        }
        Commands::Completions { .. } => unreachable!(),
    }
}

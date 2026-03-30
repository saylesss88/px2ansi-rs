//! # px2ansi-rs CLI
//!
//! The command-line interface for `px2ansi-rs`.
//!
//! This crate serves as the binary entry point, handling:
//! 1. **Configuration**: Loading defaults from system-specific config paths.
//! 2. **CLI Parsing**: Resolving subcommands (convert, index, show, list).
//! 3. **Orchestration**: Routing data between the image processor, the asset indexer,
//!    and the ANSI-to-PNG rasterizer.
//!
//! ## Execution Logic
//! The CLI prioritizes settings in this order:
//! Explicit CLI Flags > Config File (`default-config.toml`) > Hardcoded Defaults.

#![allow(clippy::multiple_crate_versions)]

mod cli;
mod config;
mod indexer;

use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rand::prelude::IndexedRandom;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use px2ansi_rs::RenderOptions;

/// The main entry point. We parse the CLI args, start a stopwatch for the "speed"
/// flex at the end, and route the command to its specific handler.
fn main() -> Result<()> {
    let start = Instant::now();

    let cli = Cli::parse();

    // Early exit for completions so we don’t load config at all.
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
        } => {
            let render = RenderOptions::from_cli(style, width, filter)?;

            let output_image: Option<PathBuf> =
                output_image.or_else(|| cfg.output_image.as_ref().map(Into::into));

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
        } => {
            let render = RenderOptions::from_cli(style, None, filter)?;

            Command::Show(ShowCmd {
                name,
                index_path: opts.index_path,
                render,
                interactive,
                latency: opts.latency,
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

/// Parameters for converting a single image file to ANSI art
#[derive(Debug)]
struct ConvertCmd {
    /// Path to the source image.
    pub input: PathBuf,
    /// Optional path to save the ANSI text output. If None, prints to stdout.
    pub output: Option<PathBuf>,
    /// Optional path to save a PNG rasterization of the result.
    pub output_image: Option<PathBuf>,
    /// Visual settings (width, filter, style).
    pub render: RenderOptions,
}

impl ConvertCmd {
    /// Reads the input image, renders it to ANSI using the provided options,
    /// and handles routing the result to the filesystem or standard output.
    pub fn run(&self) -> Result<()> {
        // 1. Load and decode the image
        let img = image::ImageReader::open(&self.input)?.decode()?;

        // 2. Render to a buffer (we use a buffer so we can reuse it for PNG rasterization)
        let mut buf = Vec::new();
        self.render.render_centered(&img, &mut buf)?;

        // 3. Handle Output (File vs Stdout)
        if let Some(path) = &self.output {
            let file = std::fs::File::create(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&buf)?;
        } else {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout.lock());
            writer.write_all(&buf)?;
        }

        // 4. Handle optional PNG rasterization
        if let Some(png_path) = &self.output_image {
            let rasterized = px2ansi_rs::rasterize::rasterize_ansi(&buf)?;
            rasterized.save(png_path)?;
            println!("✅ Saved preview to {}", png_path.display());
        }

        Ok(())
    }
}

/// Parameters for creating a new asset index from a directory.
#[derive(Debug)]
struct IndexCmd {
    /// The source directory containing image files.
    pub dir: PathBuf,
    /// The destination path for the generated JSON index.
    pub output: PathBuf,
}

impl IndexCmd {
    /// Scans the source directory and writes a JSON manifest to the output path.
    fn run(&self) -> Result<()> {
        crate::indexer::build_index(&self.dir, &self.output)?;
        Ok(())
    }
}

/// Parameters for listing the contents of an index.
#[derive(Debug)]
struct ListCmd {
    /// Path to the JSON index file.
    pub index_path: PathBuf,
    /// Maximum number of entries to display.
    pub count: Option<usize>,
}

impl ListCmd {
    /// Parses the index file and prints a formatted list of available sprites.
    fn run(&self) -> Result<()> {
        let content = std::fs::read_to_string(&self.index_path)?;
        let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&content)?;

        let limit = self.count.unwrap_or(entries.len()).min(entries.len());

        println!(
            "{} Showing {} of {} entries:",
            "Index:".magenta().bold(),
            limit,
            entries.len()
        );
        for entry in entries.iter().take(limit) {
            println!(
                "  • {:<20} {}x{}px",
                entry.name.cyan(),
                entry.dimensions.0.to_string().dimmed(),
                entry.dimensions.1.to_string().dimmed()
            );
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ShowCmd {
    pub name: String,
    pub index_path: PathBuf,
    pub render: RenderOptions,
    pub interactive: bool,
    pub latency: bool,
}

impl ShowCmd {
    /// Locates and renders an image from a managed index.
    ///
    /// This is the primary way users interact with the tool. It manages the
    /// search-and-display logic, including fuzzy matching and random selection.
    ///
    /// It supports:
    /// 1. `interactive`: A fuzzy-search TUI for when you don't know the exact name.
    /// 2. `random`: For when you're feeling adventurous.
    /// 3. `name`: Tries an exact match, then falls back to a fuzzy search.
    fn run(&self) -> Result<()> {
        let start_time = Instant::now();
        let entries: Vec<crate::indexer::ImageEntry> =
            serde_json::from_str(&std::fs::read_to_string(&self.index_path)?)?;
        if entries.is_empty() {
            anyhow::bail!("Index is empty.");
        }

        let entry_opt = if self.interactive {
            prompt_search(&entries)?
        } else {
            search_index(&entries, &self.name)?
        };

        if let Some(e) = entry_opt {
            if self.interactive {
                println!("Showing: {}", e.name.cyan().bold());
            }

            let img = image::ImageReader::open(&e.path)?.decode()?;
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout.lock());
            self.render.render_centered(&img, &mut writer)?;
        }

        if self.latency {
            print_summary(start_time.elapsed());
        }
        Ok(())
    }
}

/// The internal representation of the action the user wants to perform.
/// This bridges the gap between raw CLI arguments and execution logic.
enum Command {
    Convert(ConvertCmd),
    Index(IndexCmd),
    List(ListCmd),
    Show(ShowCmd),
}

/// A unified configuration state.
/// These values are resolved by merging the `default-config.toml`
/// with any explicit overrides provided via CLI flags.
#[derive(Debug)]
struct ResolvedOptions {
    /// Whether to display execution timing at the end of the run.
    latency: bool,
    /// The primary index file used for 'show' and 'list' commands.
    index_path: PathBuf,
}

impl ResolvedOptions {
    /// Merges the raw CLI input with the persistent configuration file.
    /// CLI flags always take priority over config file values.
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

fn handle_command(cmd: &Command) -> Result<()> {
    match cmd {
        Command::Convert(c) => ConvertCmd::run(c),
        Command::Index(c) => IndexCmd::run(c),
        Command::List(c) => ListCmd::run(c),
        Command::Show(c) => ShowCmd::run(c),
    }
}

/// Searches the index for an image entry based on the provided name.
///
/// It follows a prioritized search strategy:
/// 1. **Random**: If the name is "random", it picks a random entry from the index.
/// 2. **Exact Match**: It first tries to find a name that matches the input perfectly.
/// 3. **Fuzzy Match**: If no exact match exists, it calculates a fuzzy similarity score.
///
/// If the fuzzy score is too low (30 or below), it errors out to avoid showing
/// something completely unrelated.
fn search_index<'a>(
    entries: &'a [crate::indexer::ImageEntry],
    name: &str,
) -> Result<Option<&'a crate::indexer::ImageEntry>> {
    if name.to_lowercase() == "random" {
        return Ok(entries.choose(&mut rand::rng()));
    }
    if let Some(e) = entries.iter().find(|e| e.name == name) {
        return Ok(Some(e));
    }

    let matcher = SkimMatcherV2::default();
    let best = entries
        .iter()
        .filter_map(|e| matcher.fuzzy_match(&e.name, name).map(|score| (score, e)))
        .max_by_key(|(score, _)| *score);

    match best {
        Some((score, e)) if score > 30 => {
            println!(
                "{} No exact match for '{}'. Showing: {} (score: {})",
                "Fuzzy:".yellow(),
                name,
                e.name.cyan(),
                score
            );
            Ok(Some(e))
        }
        Some((score, e)) => anyhow::bail!("Best match '{}' score ({}) too low.", e.name, score),
        None => anyhow::bail!("No match found for '{name}'"),
    }
}

/// Spawns an interactive terminal UI for browsing the image index.
///
/// This uses a fuzzy-search selector that allows the user to type and filter
/// through the entire index in real-time. It's particularly useful when you
/// can't remember the exact filename.
///
/// Returns `Ok(None)` if the user cancels the selection (e.g., by pressing Esc).
fn prompt_search(
    entries: &[crate::indexer::ImageEntry],
) -> Result<Option<&crate::indexer::ImageEntry>> {
    let items: Vec<&String> = entries.iter().map(|e| &e.name).collect();
    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Search for a sprite")
        .items(&items)
        .interact_opt()?;
    Ok(selection.map(|idx| &entries[idx]))
}

// Helper Functions
fn print_summary(duration: std::time::Duration) {
    eprintln!(
        "\n{} took {}ms",
        "Execution".bright_blue().bold(),
        duration.as_millis()
    );
}

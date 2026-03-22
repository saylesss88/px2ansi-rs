#![allow(clippy::multiple_crate_versions)]
mod cli;
mod config;
mod indexer;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rand::prelude::IndexedRandom;
use std::io::{self, BufWriter};
use std::time::Instant;

use px2ansi_rs::{OutputMode, RenderOptions};

/// The main entry point. We parse the CLI args, start a stopwatch for the "speed"
/// flex at the end, and route the command to its specific handler.
fn main() -> Result<()> {
    let start = Instant::now();
    // 1. Load config from ~/.config/px2ansi-rs/default-config.toml
    let cfg: Config = confy::load("px2ansi-rs", None)?;

    // 2. Parse CLI args
    let cli = Cli::parse();
    let active_latency = cli.latency || cfg.latency;

    let active_index = cli.index.as_deref().unwrap_or(&cfg.index);
    // 3. Apply Overrides
    // If the user didn't specify a mode on CLI, use the config mode

    match cli.command {
        Commands::Convert {
            filename,
            output,
            mode,
            width,
            filter,
            full,
        } => {
            let render_opts = RenderOptions {
                output_mode: mode
                    .unwrap_or_else(|| cfg.mode.clone())
                    .parse()
                    .unwrap_or(OutputMode::Ansi),
                target_width: width,
                filter: filter.unwrap_or(cfg.filter).into(),
                full: full.unwrap_or(cfg.full),
            };

            let params = ConvertParams {
                path: &filename,
                output: output.as_deref(),
                render: render_opts,
            };
            convert_image(&params)?;
        }
        Commands::Index { dir, output } => {
            // Priority: --output flag > global -I flag > config.toml
            let save_path = output.as_deref().unwrap_or(active_index);
            let params = IndexParams {
                dir: &dir,
                output: save_path,
            };

            create_index(&params)?;

            println!("✅ Created index of {dir} at {save_path}");
        }
        Commands::List { count } => {
            let params = ListParams {
                index_path: active_index,
                count,
            };
            list_index_entries(&params)?;
        }

        Commands::Show {
            name,
            mode,
            full,
            filter,
            interactive,
        } => {
            // Index file check
            if !std::path::Path::new(active_index).exists() {
                anyhow::bail!(
                    "Index file not found at: {active_index}\n\n\
    💡 Tip: You need to create an index before you can 'show' images.\n\
    Try running: px2ansi-rs index <folder_with_images> -o {active_index}"
                );
            }

            let render_opts = RenderOptions {
                output_mode: mode
                    .unwrap_or_else(|| cfg.mode.clone())
                    .parse()
                    .unwrap_or_default(),
                target_width: None,
                filter: filter.unwrap_or(cfg.filter).into(),
                full: full.unwrap_or(cfg.full),
            };

            let params = ShowParams {
                name: &name,
                index_path: active_index, // &str, not .to_string()
                render: render_opts,
                interactive,
                latency: active_latency,
            };
            show_index_entry(&params)?;
        }

        Commands::Completions { shell } => {
            let mut cmd = cli::Cli::command();
            clap_complete::generate(shell, &mut cmd, "px2ansi-rs", &mut std::io::stdout());
            return Ok(()); // Important: exit early so we don't run the engine logic
        }
    }
    if active_latency {
        print_summary(start.elapsed());
    }
    Ok(())
}

/// Parameters for converting a local file into terminal art.
///
/// This struct borrows data from the CLI parser to avoid unnecessary string
/// allocations during the conversion hand-off.
pub struct ConvertParams<'a> {
    /// The path to the source image file.
    pub path: &'a str,
    /// An optional file path to save the output. If `None`, prints to stdout.
    pub output: Option<&'a str>,
    /// Visual preferences for the final render.
    pub render: RenderOptions,
}

// impl<'a> ConvertParams<'a> {
//     #[must_use]
//     pub fn new(path: &'a str) -> Self {
//         Self {
//             path,
//             output: None,
//             render: RenderOptions::default(),
//         }
//     }
// }

/// Orchestrates the conversion of a standalone image.
///
/// It handles the high-level flow: opening the file, deciding between
/// file-save or terminal-render, and ensuring the image is decoded properly.
fn convert_image(params: &ConvertParams<'_>) -> Result<()> {
    let img = image::ImageReader::open(params.path)?.decode()?;

    if let Some(output_path) = params.output {
        // File output: use RenderOptions::render directly
        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        params.render.render(&img, &mut writer)?;
    } else {
        // Terminal output: use RenderOptions::render to stdout
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        params.render.render(&img, &mut writer)?;
    }
    Ok(())
}

#[derive(Debug)]
struct ListParams<'a> {
    index_path: &'a str,
    count: Option<usize>,
}
/// Reads the generated index file and displays the "sprite" entries.
/// We cap the output by `count` so we don't accidentally flood the
/// terminal if the index contains thousands of images.
fn list_index_entries(params: &ListParams<'_>) -> Result<()> {
    // eprintln!("DEBUG index path: {index_path}");

    let entries: Vec<crate::indexer::ImageEntry> =
        serde_json::from_str(&std::fs::read_to_string(params.index_path)?)?;
    let limit = params.count.unwrap_or(entries.len()).min(entries.len());

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

/// Parameters for retrieving and displaying an image from the pre-built index.
pub struct ShowParams<'a> {
    /// The name of the image to look up (supports exact or fuzzy matching).
    pub name: &'a str,
    /// Path to the JSON index file generated by the `index` command.
    pub index_path: &'a str,
    /// Visual preferences for the final render.
    pub render: RenderOptions,
    /// If true, launches a fuzzy-finder TUI instead of using the `name` field.
    pub interactive: bool,
    /// If true, prints performance metrics (latency) after rendering.
    pub latency: bool,
}

/// Locates and renders an image from a managed index.
///
/// This is the primary way users interact with the tool. It manages the
/// search-and-display logic, including fuzzy matching and random selection.
///
/// It supports:
/// 1. `interactive`: A fuzzy-search TUI for when you don't know the exact name.
/// 2. `random`: For when you're feeling adventurous.
/// 3. `name`: Tries an exact match, then falls back to a fuzzy search.
fn show_index_entry(params: &ShowParams<'_>) -> Result<()> {
    let start_time = std::time::Instant::now();
    let entries: Vec<crate::indexer::ImageEntry> =
        serde_json::from_str(&std::fs::read_to_string(params.index_path)?)?;
    if entries.is_empty() {
        anyhow::bail!("Index is empty.");
    }

    let entry_opt = if params.interactive {
        prompt_search(&entries)?
    } else {
        search_index(&entries, params.name)?
    };

    if let Some(e) = entry_opt {
        if params.interactive {
            println!("Showing: {}", e.name.cyan().bold());
        }

        let img = image::ImageReader::open(&e.path)?.decode()?;
        // Terminal output only for show command
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        params.render.render(&img, &mut writer)?;
    }

    if params.latency {
        print_summary(start_time.elapsed());
    }
    Ok(())
}

// --- Helpers ---
fn print_summary(duration: std::time::Duration) {
    eprintln!(
        "\n{} took {}ms",
        "Execution".bright_blue().bold(),
        duration.as_millis()
    );
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

struct IndexParams<'a> {
    dir: &'a str,
    output: &'a str,
}

fn create_index(params: &IndexParams<'_>) -> Result<()> {
    crate::indexer::build_index(params.dir, params.output)?;
    Ok(())
}

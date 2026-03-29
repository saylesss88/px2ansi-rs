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
use std::path::Path;
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
    // 1. Load config from ~/.config/px2ansi-rs/default-config.toml
    let cfg: Config = confy::load("px2ansi-rs", None)?;

    // 2. Parse CLI args
    let cli = Cli::parse();
    let active_latency = cli.latency || cfg.latency;

    let active_index = cli
        .index
        .as_deref()
        .map(Path::new)
        .map_or_else(|| Path::new(&cfg.index), Path::new);
    // 3. Apply Overrides
    // If the user didn't specify a mode on CLI, use the config mode

    match cli.command {
        Commands::Convert {
            input,
            output,
            output_image,
            width,
            filter,
            style,
        } => {
            // 1. Convert Option<PathBuf> -> Option<&Path>
            let cli_out_img = output_image.as_deref();

            // 2. Convert Option<String> -> Option<&Path>
            let cfg_out_img = cfg.output_image.as_deref().map(Path::new);

            // 3. Now both sides are Option<&Path>, so .or() works!
            let active_output_image = cli_out_img.or(cfg_out_img);

            let render_opts = RenderOptions::from_cli(style, width, filter)?;

            let params = ConvertParams {
                path: &input,
                output: output.as_deref(),
                output_image: active_output_image,
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

            println!(
                "✅ Created index of {} at {}",
                dir.display(),
                save_path.display()
            );
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
            filter,
            interactive,
            style,
        } => {
            let render_opts = RenderOptions::from_cli(style, None, filter)?;

            let params = ShowParams {
                name: &name,
                index_path: active_index,
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
    pub path: &'a Path,
    /// An optional file path to save the output. If `None`, prints to stdout.
    pub output: Option<&'a Path>,
    /// Visual preferences for the final render.
    pub render: RenderOptions,
    pub output_image: Option<&'a Path>,
}

/// Orchestrates the conversion of a standalone image.
///
/// It handles the high-level flow: opening the file, deciding between
/// file-save or terminal-render, and ensuring the image is decoded properly.
fn convert_image(params: &ConvertParams<'_>) -> Result<()> {
    let img = image::ImageReader::open(params.path)?.decode()?;

    // Render to buffer once, reuse for both outputs
    let mut buf = Vec::new();
    params.render.render_centered(&img, &mut buf)?;

    if let Some(output_path) = params.output {
        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&buf)?;
    } else {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        writer.write_all(&buf)?;
    }

    // Save rasterized PNG if requested
    if let Some(image_path) = params.output_image {
        save_ansi_as_image(params, image_path)?;
    }

    Ok(())
}
#[derive(Debug)]
struct ListParams<'a> {
    index_path: &'a Path,
    count: Option<usize>,
}
fn create_index(params: &IndexParams<'_>) -> Result<()> {
    // Assuming build_index is updated to accept &Path or converts it
    crate::indexer::build_index(params.dir, params.output)?;
    Ok(())
}
/// Reads the generated index file and displays the "sprite" entries.
/// We cap the output by `count` so we don't accidentally flood the
/// terminal if the index contains thousands of images.
fn list_index_entries(params: &ListParams<'_>) -> Result<()> {
    // eprintln!("DEBUG index path: {index_path}");
    let content = std::fs::read_to_string(params.index_path)?;
    let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&content)?;

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
    pub index_path: &'a Path,
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
        // params.render.render(&img, &mut writer)?;
        params.render.render_centered(&img, &mut writer)?;
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
    dir: &'a Path,
    output: &'a Path,
}

// fn save_ansi_as_image(params: &ConvertParams<'_>, image_path: &str) -> Result<()> {
//     // Re-open and prepare the image at the same dimensions the renderer used
//     let img = image::ImageReader::open(params.path)?.decode()?;
//     let prepared = params.render.prepare_image(&img);
//     prepared.save(image_path)?;
//     println!("✅ Saved preview to {image_path}");
//     Ok(())
// }
// fn save_ansi_as_image(params: &ConvertParams<'_>, image_path: &str) -> Result<()> {
//     let img = image::ImageReader::open(params.path)?.decode()?;
//     let mut buf = Vec::new();
//     params.render.render_centered(&img, &mut buf)?;

//     let rasterized = px2ansi_rs::rasterize::rasterize_ansi(&buf)?;
//     rasterized.save(image_path)?;
//     println!("✅ Saved preview to {image_path}");
//     Ok(())
// }
fn save_ansi_as_image(params: &ConvertParams<'_>, image_path: &Path) -> Result<()> {
    let img = image::ImageReader::open(params.path)?.decode()?;
    let mut buf = Vec::new();
    params.render.render_centered(&img, &mut buf)?;

    let rasterized = px2ansi_rs::rasterize::rasterize_ansi(&buf)?;
    rasterized.save(image_path)?; // .save() also accepts &Path
    println!("✅ Saved preview to {}", image_path.display()); // Use .display() to print paths
    Ok(())
}

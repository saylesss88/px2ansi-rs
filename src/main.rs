#![allow(clippy::multiple_crate_versions)]
mod cli;
mod config;
mod indexer;
use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use core::option::Option::None;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use image::imageops::FilterType;
use rand::prelude::IndexedRandom;
use std::io::{self, BufWriter, Write};
use std::time::Instant;
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::{AnsiArtOptions, OutputMode, write_ansi_art};

/// The main entry point. We parse the CLI args, start a stopwatch for the "speed"
/// flex at the end, and route the command to its specific handler.
fn main() -> Result<()> {
    let start = Instant::now();
    // 1. Load config from ~/.config/px2ansi-rs/default-config.toml
    let cfg: AppConfig = confy::load("px2ansi-rs", None)?;

    // 2. Parse CLI args
    let cli = Cli::parse();

    let active_index = cli.index.as_deref().unwrap_or(&cfg.index);
    // 3. Apply Overrides
    // If the user didn't specify a mode on CLI, use the config mode
    let active_latency = cli.latency || cfg.latency;

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
            handle_convert(&params)?;
        }
        Commands::Index { dir, output } => {
            // Priority: --output flag > global -I flag > config.toml
            let save_path = output.as_deref().unwrap_or(active_index);

            handle_index(&dir, save_path)?;

            println!("✅ Created index of {dir} at {save_path}");
        }
        Commands::List { count } => handle_list(active_index.to_string(), count)?,
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
            handle_show(&params)?;
        }

        Commands::Completions { shell } => {
            let mut cmd = cli::Cli::command();
            clap_complete::generate(shell, &mut cmd, "px2ansi-rs", &mut std::io::stdout());
            return Ok(()); // Important: exit early so we don't run the engine logic
        }
    }
    if active_latency {
        print_summary(start);
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

impl<'a> ConvertParams<'a> {
    #[must_use]
    pub fn new(path: &'a str) -> Self {
        Self {
            path,
            output: None,
            render: RenderOptions::default(),
        }
    }
}

/// Orchestrates the conversion of a standalone image.
///
/// It handles the high-level flow: opening the file, deciding between
/// file-save or terminal-render, and ensuring the image is decoded properly.
fn handle_convert(params: &ConvertParams<'_>) -> Result<()> {
    let output_mode = params.render.output_mode;
    let img = image::ImageReader::open(params.path)?.decode()?;

    if let Some(output_path) = params.output {
        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        write_ansi_art(
            &img,
            &mut writer,
            AnsiArtOptions {
                mode: output_mode,
                full_block: params.render.full,
            },
        )?;
    } else {
        process_and_render(img, params.render)?;
    }
    Ok(())
}

/// Reads the generated index file and displays the "sprite" entries.
/// We cap the output by `count` so we don't accidentally flood the
/// terminal if the index contains thousands of images.
fn handle_list(index_path: String, count: Option<usize>) -> Result<()> {
    // eprintln!("DEBUG index path: {index_path}");

    let entries: Vec<crate::indexer::ImageEntry> =
        serde_json::from_str(&std::fs::read_to_string(index_path)?)?;
    let limit = count.unwrap_or(entries.len()).min(entries.len());

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
fn handle_show(params: &ShowParams<'_>) -> Result<()> {
    // eprintln!("DEBUG index path: {index}");
    // Start the timer
    let start_time = std::time::Instant::now();
    let entries: Vec<crate::indexer::ImageEntry> =
        serde_json::from_str(&std::fs::read_to_string(params.index_path)?)?;
    if entries.is_empty() {
        anyhow::bail!("Index is empty.");
    }

    let entry_opt = if params.interactive {
        select_interactive(&entries)?
    } else {
        find_entry(&entries, params.name)?
    };

    // Ensure 'img' is created only if an entry was found
    if let Some(e) = entry_opt {
        if params.interactive {
            println!("Showing: {}", e.name.cyan().bold());
        }

        let img = image::ImageReader::open(&e.path)?.decode()?;
        process_and_render(img, params.render)?;
    }

    if params.latency {
        let duration = start_time.elapsed();
        println!("\n--- Metadata ---");
        println!("Render latency: {duration:?}");
    }

    Ok(())
}

// --- Helpers ---
/// Prints execution metadata if the user opted in via --latency.
fn print_summary(start: Instant) {
    let duration = start.elapsed();
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
fn find_entry<'a>(
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
fn select_interactive(
    entries: &[crate::indexer::ImageEntry],
) -> Result<Option<&crate::indexer::ImageEntry>> {
    let items: Vec<&String> = entries.iter().map(|e| &e.name).collect();
    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Search for a sprite")
        .items(&items)
        .interact_opt()?;
    Ok(selection.map(|idx| &entries[idx]))
}

/// Configuration for how an image should be processed and rendered to the terminal.
///
/// This handles the "look and feel" of the output, including the character set
/// (ANSI vs Unicode), scaling filters, and whether to use half-block positioning.#
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    // Determines the character set used for rendering (e.g., ASCII/ANSI or Unicode)
    pub output_mode: OutputMode,
    /// An optional fixed width. If `None`, the renderer will calculate the best fit
    /// based on the current terminal size.
    pub target_width: Option<u32>,

    /// The algorithm used for resizing. `Nearest` is best for pixel art,
    /// while `Lanczos3` provides the best results for high-res photos.
    pub filter: FilterType,
    /// If true, uses the full color/pixel density available for the chosen mode.
    pub full: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            output_mode: OutputMode::Ansi,
            target_width: None,
            filter: FilterType::Lanczos3, // Reasonable default
            full: false,
        }
    }
}

/// The engine room of the crate. This function calculates the optimal scale
/// for the image before it hits the ANSI writer.
///
/// It accounts for the "Terminal Aspect Ratio Problem": standard terminal
/// characters are roughly 1:2 (twice as tall as they are wide).
///
/// If `options.full` is enabled (Unicode mode), we effectively double our
/// vertical resolution because we can color the top and bottom of a character
/// cell independently. This function adjusts the target height accordingly
/// to ensure the image doesn't look "squashed" or "stretched."
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn process_and_render(mut img: image::DynamicImage, options: RenderOptions) -> Result<()> {
    const MAX_SAFE: u32 = 16384;
    let term_dims = terminal_size().map(|(Width(tw), Height(th))| (u32::from(tw), u32::from(th)));

    let (max_w, max_h) = if let Some((tw, th)) = term_dims {
        let mw = if options.output_mode == OutputMode::Unicode && options.full {
            tw / 2
        } else {
            tw
        }
        .saturating_sub(2);
        let mh = if options.output_mode == OutputMode::Unicode && options.full {
            th
        } else {
            th * 2
        }
        .saturating_sub(2);
        (mw, mh)
    } else {
        (80, 40)
    };

    let orig_w = img.width();
    let orig_h = img.height();

    let (render_w, render_h) = options.target_width.map_or_else(
        || {
            if options.filter == FilterType::Nearest && orig_w < 120 {
                // --- CRISP SPRITE LOGIC ---
                // Find the largest WHOLE NUMBER scale that fits the terminal
                let scale_w = (f64::from(max_w) / f64::from(orig_w)).floor();
                let scale_h = (f64::from(max_h) / f64::from(orig_h)).floor();

                // Use a scale of at least 1, but prefer a whole number like 2.0 or 3.0
                let scale = scale_w.min(scale_h).max(1.0);

                let rw = (f64::from(orig_w) * scale) as u32;
                let rh = (f64::from(orig_h) * scale) as u32;
                (rw, rh)
            } else {
                // --- NORMAL MODE ---
                let scale = (f64::from(max_w) / f64::from(orig_w))
                    .min(f64::from(max_h) / f64::from(orig_h));
                (
                    (f64::from(orig_w) * scale).round() as u32,
                    (f64::from(orig_h) * scale).round() as u32,
                )
            }
        },
        |tw| {
            let aspect = f64::from(orig_h) / f64::from(orig_w);
            (tw, (f64::from(tw) * aspect).round() as u32)
        },
    );

    // SAFETY: Clamp to a reasonable max for terminal art / image crate
    let render_w = render_w.clamp(1, MAX_SAFE);
    let render_h = render_h.clamp(1, MAX_SAFE);

    if render_w == MAX_SAFE || render_h == MAX_SAFE {
        let clamped_dims = if render_w == MAX_SAFE && render_h == MAX_SAFE {
            format!("{MAX_SAFE}x{MAX_SAFE}")
        } else if render_w == MAX_SAFE {
            format!("{MAX_SAFE}x{render_h}")
        } else {
            format!("{render_w}x{MAX_SAFE}")
        };
        eprintln!(
            "Warning: image dimensions clamped to {clamped_dims} to avoid excessive memory usage"
        );
    }

    img = img.resize_exact(render_w, render_h, options.filter);

    // Use resize_exact with Nearest to prevent sub-pixel shifting
    // img = img.resize_exact(render_w.max(1), render_h.max(1), filter);

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_ansi_art(
        &img,
        &mut writer,
        AnsiArtOptions {
            mode: options.output_mode,
            full_block: options.full,
        },
    )?;
    writer.flush()?;
    Ok(())
}

fn handle_index(dir: &str, output: &str) -> Result<()> {
    crate::indexer::build_index(dir, output)?;
    Ok(())
}

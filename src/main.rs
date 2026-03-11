#![allow(clippy::multiple_crate_versions)]
mod cli;
mod indexer;
use crate::cli::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rand::prelude::IndexedRandom;
use std::io::{self, BufWriter, Write};
use std::time::Instant;
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::{OutputMode, write_ansi_art};

/// The main entry point. We parse the CLI args, start a stopwatch for the "speed"
/// flex at the end, and route the command to its specific handler.
fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.command {
        Commands::Convert {
            filename,
            output,
            mode,
            width,
            filter,
            full,
        } => {
            handle_convert(filename, output, &mode, width, filter, full)?;
        }
        Commands::Index { dir, output } => {
            // Indexing is simple enough to keep inline: scan a dir, save the JSON.
            let json = crate::indexer::build_index(&dir)?;
            std::fs::write(output, json)?;

            // This part should always show unless you add a --quiet flag later
            println!("{} created successfully!", "Index".cyan().bold());

            // Only show the speed flex if they asked for --latency
            // if cli.latency {
            //     print_summary(start);
            // }
        }
        Commands::List { index, count } => handle_list(index, count)?,
        Commands::Show {
            name,
            index,
            mode,
            filter,
            full,
            interactive,
        } => {
            handle_show(&name, index, &mode, filter, full, interactive)?;
        }
    }

    if cli.latency {
        print_summary(start);
    }
    Ok(())
}

/// Takes a single image file and turns it into ANSI art.
///
/// If an output path is provided, it saves the raw text to a file.
/// Otherwise, it scales the image to fit your current terminal window
/// and dumps it to stdout.
fn handle_convert(
    path: String,
    out: Option<String>,
    mode: &str,
    w: Option<u32>,
    f: crate::cli::ResizeFilter,
    full: bool,
) -> Result<()> {
    let output_mode = parse_mode(mode);
    let img = image::ImageReader::open(path)?.decode()?;

    if let Some(output_path) = out {
        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        write_ansi_art(&img, &mut writer, output_mode, full)?;
    } else {
        process_and_render(img, output_mode, w, f.into(), full)?;
    }
    Ok(())
}

/// Reads the generated index file and displays the "sprite" entries.
/// We cap the output by `count` so we don't accidentally flood the
/// terminal if the index contains thousands of images.
fn handle_list(index_path: String, count: Option<usize>) -> Result<()> {
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

/// The "main event" command. It looks for a specific image in the index.
///
/// It supports:
/// 1. `interactive`: A fuzzy-search TUI for when you don't know the exact name.
/// 2. `random`: For when you're feeling adventurous.
/// 3. `name`: Tries an exact match, then falls back to a fuzzy search.
fn handle_show(
    name: &str,
    index: String,
    mode: &str,
    filter: crate::cli::ResizeFilter,
    full: bool,
    interactive: bool,
) -> Result<()> {
    let entries: Vec<crate::indexer::ImageEntry> =
        serde_json::from_str(&std::fs::read_to_string(index)?)?;
    if entries.is_empty() {
        anyhow::bail!("Index is empty.");
    }

    let entry_opt = if interactive {
        select_interactive(&entries)?
    } else {
        find_entry(&entries, name)?
    };

    // Ensure 'img' is created only if an entry was found
    if let Some(e) = entry_opt {
        if interactive {
            println!("Showing: {}", e.name.cyan().bold());
        }

        let img = image::ImageReader::open(&e.path)?.decode()?;
        process_and_render(img, parse_mode(mode), None, filter.into(), full)?;
    }

    Ok(())
}

// --- Helpers ---
/// A little helper to translate a string into our `OutputMode` enum.
/// Defaults to ANSI if it doesn't recognize the string.
fn parse_mode(mode: &str) -> OutputMode {
    if mode == "unicode" {
        OutputMode::Unicode
    } else {
        OutputMode::Ansi
    }
}

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
/// This function automatically detects the terminal size to prevent line-wrapping,
/// which is often the cause of horizontal gaps/lines in the rendered art.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn process_and_render(
    mut img: image::DynamicImage,
    output_mode: OutputMode,
    target_width: Option<u32>,
    filter: image::imageops::FilterType,
    full: bool,
) -> Result<()> {
    // Determine the best width for the image based on user input or terminal dimensions
    let final_width = target_width
        .or_else(|| {
            terminal_size().map(|(Width(tw), Height(th))| {
                let term_w = u32::from(tw);
                let term_h = u32::from(th);

                let max_w = if output_mode == OutputMode::Unicode && full {
                    term_w / 2
                } else {
                    term_w
                }
                .saturating_sub(2);

                let max_h = if output_mode == OutputMode::Unicode && full {
                    term_h
                } else {
                    term_h * 2
                }
                .saturating_sub(2);

                if img.width() > max_w || img.height() > max_h {
                    let scale = (f64::from(max_w) / f64::from(img.width()))
                        .min(f64::from(max_h) / f64::from(img.height()));

                    (f64::from(img.width()) * scale).round() as u32
                } else {
                    img.width()
                }
            })
        })
        .unwrap_or(80);

    let safe_w = final_width.max(1);
    let aspect_ratio = f64::from(img.height()) / f64::from(img.width());
    let new_height = (f64::from(safe_w) * aspect_ratio) as u32;

    img = img.resize(safe_w, new_height, filter);

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    write_ansi_art(&img, &mut writer, output_mode, full)?;
    writer.flush()?;
    Ok(())
}

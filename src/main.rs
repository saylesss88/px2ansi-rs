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
use rand::prelude::IndexedRandom;
use std::io::{self, BufWriter, Write};
use std::time::Instant;
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::{OutputMode, write_ansi_art};

/// The main entry point. We parse the CLI args, start a stopwatch for the "speed"
/// flex at the end, and route the command to its specific handler.
fn main() -> Result<()> {
    let start = Instant::now();
    // 1. Load config from ~/.config/px2ansi-rs/default-config.toml
    let cfg: AppConfig = confy::load("px2ansi-rs", None)?;

    // // Normalize cfg.index to an absolute path once
    // let index_path = PathBuf::from(&cfg.index);
    // if index_path.is_relative() {
    //     // choose whatever base dir you like; current_dir is the simplest
    //     let abs = std::env::current_dir()?.join(index_path);
    //     cfg.index = abs.to_string_lossy().into_owned();
    // }
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
            // MERGE LOGIC: CLI wins, then Config, then Hardcoded Default
            let active_mode = mode.unwrap_or_else(|| cfg.mode.clone());
            let active_full = full.unwrap_or(cfg.full);
            let active_filter = filter.unwrap_or(cfg.filter);

            handle_convert(
                filename,
                output,
                &active_mode,
                width,
                active_filter,
                active_full,
            )?;
        }
        Commands::Index { dir, output } => {
            // Priority: --output flag > global -I flag > config.toml
            let save_path = output.as_deref().unwrap_or(active_index);

            handle_index(&dir, save_path)?;

            println!("✅ Created index of {dir} at {save_path}");
        }
        // Commands::Index { dir, output } => {
        //     // Indexing is simple enough to keep inline: scan a dir, save the JSON.
        //     // For creating an index, use -o if provided, otherwise the global active_index
        //     let save_path = output.as_deref().unwrap_or(active_index);
        //     handle_index(&dir, save_path)?;
        //     println!("Index successfully created at: {}", save_path);

        //     // Only show the speed flex if they asked for --latency
        //     // if cli.latency {
        //     //     print_summary(start);
        //     // }
        // }
        Commands::List { count } => handle_list(active_index.to_string(), count)?,
        Commands::Show {
            name,
            mode,
            full,
            filter,
            interactive,
        } => {
            let mode_val = mode.unwrap_or(cfg.mode);
            // let filter_val = cfg.filter;
            let full_val = full.unwrap_or(cfg.full);
            let active_filter = filter.unwrap_or(cfg.filter);

            if !std::path::Path::new(active_index).exists() {
                anyhow::bail!(
                    "Index file not found at: {active_index}\n\n\
    💡 Tip: You need to create an index before you can 'show' images.\n\
    Try running: px2ansi-rs index <folder_with_images> -o {active_index}"
                );
            }

            handle_show(
                &name,
                active_index.to_string(), // This refers to the variable from the top of main()
                &mode_val,
                active_filter,
                // filter_val,
                full_val,
                interactive,
                active_latency,
            )?;
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
    latency: bool,
) -> Result<()> {
    // eprintln!("DEBUG index path: {index}");
    // Start the timer
    let start_time = std::time::Instant::now();
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

    if latency {
        let duration = start_time.elapsed();
        println!("\n--- Metadata ---");
        println!("Render latency: {duration:?}");
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
    let final_width = target_width
        .or_else(|| {
            terminal_size().map(|(Width(tw), Height(th))| {
                // Your existing sizing logic (keep it!)
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
                    img.width() // Native size for sprites ✓
                }
            })
        })
        .unwrap_or(80);

    let safe_w = final_width.max(1);
    let aspect_ratio = f64::from(img.height()) / f64::from(img.width());
    let new_height = (f64::from(safe_w) * aspect_ratio).round() as u32;

    // CRITICAL FIX: ALWAYS RESIZE (even 1x) → FILTER ALWAYS APPLIES
    img = img.resize_exact(safe_w, new_height, filter);

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_ansi_art(&img, &mut writer, output_mode, full)?;
    writer.flush()?;
    Ok(())
}
// fn process_and_render(
//     mut img: image::DynamicImage,
//     output_mode: OutputMode,
//     target_width: Option<u32>,
//     filter: image::imageops::FilterType,
//     full: bool,
// ) -> Result<()> {
//     // Determine the best width for the image based on user input or terminal dimensions
//     let final_width = target_width
//         .or_else(|| {
//             //     terminal_size().map(|(Width(tw), Height(th))| {
//             //         let term_w = u32::from(tw);
//             //         let term_h = u32::from(th);

//             //         // Define the bounding box of the terminal
//             //         let max_w = if output_mode == OutputMode::Unicode && full {
//             //             term_w / 2
//             //         } else {
//             //             term_w
//             //         }
//             //         .saturating_sub(2);

//             //         let max_h = if output_mode == OutputMode::Unicode && full {
//             //             term_h
//             //         } else {
//             //             term_h * 2
//             //         }
//             //         .saturating_sub(2);

//             //         // Calculate scale to fit the bounding box (upscale or downscale)
//             //         let scale = (f64::from(max_w) / f64::from(img.width()))
//             //             .min(f64::from(max_h) / f64::from(img.height()));

//             //         (f64::from(img.width()) * scale).round() as u32
//             //     })
//             // })
//             // .unwrap_or(80);
//             terminal_size().map(|(Width(tw), Height(th))| {
//                 let term_w = u32::from(tw);
//                 let term_h = u32::from(th);

//                 let max_w = if output_mode == OutputMode::Unicode && full {
//                     term_w / 2
//                 } else {
//                     term_w
//                 }
//                 .saturating_sub(2);

//                 let max_h = if output_mode == OutputMode::Unicode && full {
//                     term_h
//                 } else {
//                     term_h * 2
//                 }
//                 .saturating_sub(2);

//                 if img.width() > max_w || img.height() > max_h {
//                     let scale = (f64::from(max_w) / f64::from(img.width()))
//                         .min(f64::from(max_h) / f64::from(img.height()));

//                     (f64::from(img.width()) * scale).round() as u32
//                 } else {
//                     img.width()
//                 }
//             })
//         })
//         .unwrap_or(80);

//     let safe_w = final_width.max(1);
//     let aspect_ratio = f64::from(img.height()) / f64::from(img.width());
//     let new_height = (f64::from(safe_w) * aspect_ratio) as u32;

//     img = img.resize(safe_w, new_height, filter);

//     let stdout = io::stdout();
//     let mut writer = BufWriter::new(stdout.lock());

//     write_ansi_art(&img, &mut writer, output_mode, full)?;
//     writer.flush()?;
//     Ok(())
// }

fn handle_index(dir: &str, output: &str) -> Result<()> {
    crate::indexer::build_index(dir, output)?;
    Ok(())
}

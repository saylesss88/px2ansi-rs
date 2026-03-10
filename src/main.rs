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
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::time::Instant;
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::{OutputMode, write_ansi_art};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Start the clock
    let start = Instant::now();

    match cli.command {
        // Handle single-file conversion
        Commands::Convert {
            filename,
            output,
            mode,
            width,
            filter,
            full,
        } => {
            let output_mode = if mode == "unicode" {
                OutputMode::Unicode
            } else {
                OutputMode::Ansi
            };
            // Load and decode the image from the provided path
            let img = image::ImageReader::open(&filename)?.decode()?;

            // If saving to file, we skip the terminal-aware helper and go direct
            if let Some(path) = output {
                let file = File::create(path)?;
                let mut writer = BufWriter::new(file);
                let output_mode = if mode == "unicode" {
                    OutputMode::Unicode
                } else {
                    OutputMode::Ansi
                };

                write_ansi_art(&img, &mut writer, output_mode, full)?;
            } else {
                // If printing to stdout, we use the helper to fit the image to the current window
                process_and_render(img, output_mode, width, filter.into(), full)?;
            }
            if !cli.silent {
                let duration = start.elapsed();
                eprintln!(
                    "\n{} in {}ms",
                    "Finished".green().bold(),
                    duration.as_millis()
                );
            }
        }

        // Build a JSON index
        Commands::Index { dir, output } => {
            let json = crate::indexer::build_index(&dir)?;
            std::fs::write(output, json)?;
            if !cli.silent {
                let duration = start.elapsed();
                println!(
                    "{} created successfully in {}ms!",
                    "Index".cyan().bold(),
                    duration.as_millis()
                );
            }
        }

        Commands::List { index, count } => {
            let index_data = std::fs::read_to_string(&index)?;
            let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&index_data)?;

            let total = entries.len();
            let limit = count.unwrap_or(total);

            println!(
                "{} Showing {} of {} entries:",
                "Index:".magenta().bold(),
                limit.min(total),
                total
            );

            for entry in entries.iter().take(limit) {
                println!(
                    "  • {:<20} {}x{}px",
                    entry.name.cyan(),
                    entry.dimensions.0.to_string().dimmed(),
                    entry.dimensions.1.to_string().dimmed()
                );
            }
        }
        // Retrieve and display an image
        Commands::Show {
            name,
            index,
            mode,
            filter,
            full,
            interactive,
        } => {
            let output_mode = if mode == "unicode" {
                OutputMode::Unicode
            } else {
                OutputMode::Ansi
            };

            let index_data = std::fs::read_to_string(&index)?;
            let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&index_data)?;

            if entries.is_empty() {
                anyhow::bail!("Index is empty.");
            }

            // 1. Determine which entry to show (Interactive vs CLI)
            let entry = if interactive {
                // Interactive TUI selection
                let items: Vec<&String> = entries.iter().map(|e| &e.name).collect();
                let selection =
                    dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Search for a sprite")
                        .items(&items)
                        .interact_opt()?;

                match selection {
                    Some(idx) => &entries[idx],
                    None => return Ok(()), // User pressed Esc
                }
            } else {
                // Standard CLI logic: Random -> Exact -> Fuzzy
                if name.to_lowercase() == "random" {
                    entries.choose(&mut rand::rng()).unwrap()
                } else if let Some(e) = entries.iter().find(|e| e.name == name) {
                    e
                } else {
                    let matcher = SkimMatcherV2::default();
                    let best = entries
                        .iter()
                        .filter_map(|e| matcher.fuzzy_match(&e.name, &name).map(|score| (score, e)))
                        .max_by_key(|(score, _)| *score);

                    match best {
                        Some((score, e)) if score > 30 => {
                            println!(
                                "{} No exact match for '{}'. Showing: {} {} {}",
                                "Fuzzy:".yellow(),
                                name,
                                e.name.cyan(),
                                "score:".dimmed(),
                                score.to_string().magenta()
                            );
                            e
                        }
                        Some((score, e)) => {
                            anyhow::bail!(
                                "Best match was '{}' but score ({}) was too low. Try being more specific.",
                                e.name,
                                score
                            );
                        }
                        None => anyhow::bail!("No match foun for '{name}'"),
                    }
                }
            };

            // 2. Rener the chosen entry
            let img = image::ImageReader::open(&entry.path)?.decode()?;

            // If in interactive mode, we might want to print the name since user didn't type it
            if interactive {
                println!("Showing: {}", entry.name.cyan().bold());
            }

            process_and_render(img, output_mode, None, filter.into(), full)?;

            if !cli.silent {
                let duration = start.elapsed();
                eprintln!(
                    "\n{} in {}ms",
                    "Finished".green().bold(),
                    duration.as_millis()
                );
            }
        }
    }
    Ok(())
}
///
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

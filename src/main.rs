#![allow(clippy::multiple_crate_versions)]
mod cli;
mod indexer;
use crate::cli::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use px2ansi_rs::OutputMode;
use rand::prelude::IndexedRandom;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::time::Instant;
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::write_ansi_art;

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

        // Build a JSON index of an image directory for quick retrieval
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

        // Retrieve and display an image from a previously generated index
        Commands::Show {
            name,
            index,
            mode,
            filter,
            full,
        } => {
            let output_mode = if mode == "unicode" {
                OutputMode::Unicode
            } else {
                OutputMode::Ansi
            };

            // Parse the JSON index into a list of image entries
            let index_data = std::fs::read_to_string(&index)?;
            let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&index_data)?;

            if entries.is_empty() {
                anyhow::bail!("The index file is empty. Run 'index' first.");
            }

            // Handle the 'random' keyword or search for a specific filename (file stem)
            let entry = if name.to_lowercase() == "random" {
                entries.choose(&mut rand::rng()).ok_or_else(|| {
                    anyhow::anyhow!("Failed to pick a random entry (index is empty?)")
                })?
            } else {
                entries.iter().find(|e| e.name == name).ok_or_else(|| {
                    anyhow::anyhow!("Could not find entry '{name}' in index {index}")
                })?
            };

            let img = image::ImageReader::open(&entry.path)?.decode()?;
            println!("Showing: {}", entry.name);

            // Default to Nearest filter for 'Show' to keep pixel art crisp
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

/// Handles the logic for scaling an image to fit the terminal and writing it to stdout.
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

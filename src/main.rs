#![allow(clippy::multiple_crate_versions)]
mod cli;
mod indexer;
use crate::cli::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use px2ansi_rs::OutputMode;
use rand::prelude::IndexedRandom;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::write_ansi_art;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // Handle single-file conversion
        Commands::Convert {
            filename,
            output,
            mode,
            width,
            filter,
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
                write_ansi_art(&img, &mut writer, output_mode)?;
            } else {
                // If printing to stdout, we use the helper to fit the image to the current window
                process_and_render(img, output_mode, width, filter.into())?;
            }
        }

        // Build a JSON index of an image directory for quick retrieval
        Commands::Index { dir, output } => {
            let json = crate::indexer::build_index(&dir)?;
            std::fs::write(output, json)?;
            println!("Index created successfully!");
        }

        // Retrieve and display an image from a previously generated index
        Commands::Show {
            name,
            index,
            mode,
            filter,
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
            process_and_render(img, output_mode, None, filter.into())?;
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
) -> Result<()> {
    // Determine the best width for the image based on user input or terminal dimensions
    let final_width = target_width
        .or_else(|| {
            terminal_size().map(|(Width(tw), Height(th))| {
                let term_w = u32::from(tw);
                let term_h = u32::from(th);

                // In Unicode mode, pixels are 2 characters wide (██), so we halve the max width.
                let max_w = if output_mode == OutputMode::Unicode {
                    term_w / 2
                } else {
                    term_w
                }
                .saturating_sub(2); // Leave a small buffer for borders/padding

                // In Ansi mode, pixels are packed 2-per-character (half-blocks), doubling vertical resolution.
                let max_h = if output_mode == OutputMode::Unicode {
                    term_h
                } else {
                    term_h * 2
                }
                .saturating_sub(2);

                // Calculate the scale factor while maintaining the original aspect ratio
                if img.width() > max_w || img.height() > max_h {
                    let scale = (f64::from(max_w) / f64::from(img.width()))
                        .min(f64::from(max_h) / f64::from(img.height()));

                    (f64::from(img.width()) * scale).round() as u32
                } else {
                    img.width()
                }
            })
        })
        .unwrap_or(80); // Default to 80 chars if size detection fails

    let safe_w = final_width.max(1);
    let aspect_ratio = f64::from(img.height()) / f64::from(img.width());
    let new_height = (f64::from(safe_w) * aspect_ratio) as u32;

    // Perform the actual resize using the selected resampling filter
    img = img.resize(safe_w, new_height, filter);

    // Stream the ANSI codes to stdout using a buffered writer for performance
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_ansi_art(&img, &mut writer, output_mode)?;
    writer.flush()?;
    Ok(())
}

#![allow(clippy::multiple_crate_versions)]
mod cli;
mod indexer;
use crate::cli::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use px2ansi_rs::OutputMode;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::write_ansi_art;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
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

            // 1. Load image - Use 'filename' directly
            let mut reader = image::ImageReader::open(&filename)?;
            reader.no_limits();
            let mut img = reader.decode()?;

            // 2. Determine Target Size - Use 'width' directly
            let target_width = width.or_else(|| {
                if let Some((Width(term_w), Height(term_h))) = terminal_size() {
                    let max_w = u32::from(term_w);

                    // IF UNICODE: 1 char = 1 pixel (max height is term_h)
                    // IF ANSI: 1 char = 2 pixels (max height is term_h * 2)
                    let max_h = if output_mode == OutputMode::Unicode {
                        u32::from(term_h)
                    } else {
                        u32::from(term_h) * 2
                    };

                    let img_w = img.width();
                    let img_h = img.height();

                    if img_w > max_w || img_h > max_h {
                        let width_ratio = f64::from(max_w) / f64::from(img_w);
                        let height_ratio = f64::from(max_h) / f64::from(img_h);
                        let scale = width_ratio.min(height_ratio);
                        Some((f64::from(img_w) * scale).round() as u32)
                    } else {
                        None
                    }
                } else {
                    Some(100)
                }
            });

            // 3. Resize with Aspect Correction
            if let Some(w) = target_width {
                let safe_w = w.max(1);

                // Calculate height based on original aspect ratio
                let new_height =
                    (f64::from(img.height()) * (f64::from(safe_w) / f64::from(img.width()))) as u32;

                if output_mode == OutputMode::Unicode {
                    // new_height = (f64::from(new_height) * 0.5).max(1.0) as u32;
                    // Use resize_exact to force the squashed ratio
                    // img = img.resize_exact(safe_w, new_height, filter.into());
                    img = img.resize(safe_w, new_height, filter.into());
                } else {
                    // Ansi mode handles aspect ratio naturally with half-blocks
                    img = img.resize(safe_w, new_height, filter.into());
                }
            }
            // 4. Output - Use 'output' and 'output_mode'
            if let Some(output_path) = output {
                let file = File::create(output_path)?;
                let mut writer = BufWriter::new(file);
                write_ansi_art(&img, &mut writer, output_mode)?;
            } else {
                let stdout = io::stdout();
                let mut writer = BufWriter::new(stdout.lock());
                write_ansi_art(&img, &mut writer, output_mode)?;
                writer.flush()?;
            }
        }
        Commands::Index { dir, output } => {
            let json = crate::indexer::build_index(&dir)?;
            std::fs::write(output, json)?;
            println!("Index created successfully!");
        }

        Commands::Show { name, index, mode } => {
            // 1. Load the index
            let index_data = std::fs::read_to_string(&index)?;
            let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&index_data)?;

            // 2. Find the match
            if let Some(entry) = entries.iter().find(|e| e.name == name) {
                let output_mode = if mode == "unicode" {
                    OutputMode::Unicode
                } else {
                    OutputMode::Ansi
                };

                // 3. loading/rendering logic
                let reader = image::ImageReader::open(&entry.path)?;
                let img = reader.decode()?;

                let stdout = io::stdout();
                let mut writer = BufWriter::new(stdout.lock());
                write_ansi_art(&img, &mut writer, output_mode)?;
                writer.flush()?;
            } else {
                eprintln!("Error: Could not find '{}' in {}", name, index);
            }
        }
    }
    Ok(())
}

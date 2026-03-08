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

            let mut reader = image::ImageReader::open(&filename)?;
            reader.no_limits();
            let mut img = reader.decode()?;

            let target_width = width.or_else(|| {
                if let Some((Width(tw), Height(th))) = terminal_size() {
                    let term_w = u32::from(tw);
                    let term_h = u32::from(th);

                    let max_w = if output_mode == OutputMode::Unicode {
                        term_w / 2
                    } else {
                        term_w
                    }
                    .saturating_sub(2);

                    let max_h = if output_mode == OutputMode::Unicode {
                        term_h
                    } else {
                        term_h * 2
                    }
                    .saturating_sub(2);

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

            if let Some(w) = target_width {
                let safe_w = w.max(1);
                let new_height =
                    (f64::from(img.height()) * (f64::from(safe_w) / f64::from(img.width()))) as u32;
                // .into() converts your CLI filter to image::imageops::FilterType
                img = img.resize(safe_w, new_height, filter.into());
            }

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
            let index_data = std::fs::read_to_string(&index)?;
            let entries: Vec<crate::indexer::ImageEntry> = serde_json::from_str(&index_data)?;

            if let Some(entry) = entries.iter().find(|e| e.name == name) {
                let output_mode = if mode == "unicode" {
                    OutputMode::Unicode
                } else {
                    OutputMode::Ansi
                };

                let mut img = image::ImageReader::open(&entry.path)?.decode()?;

                if let Some((Width(tw), Height(th))) = terminal_size() {
                    let term_w = u32::from(tw);
                    let term_h = u32::from(th);

                    let max_w = if output_mode == OutputMode::Unicode {
                        term_w / 2
                    } else {
                        term_w
                    }
                    .saturating_sub(2);

                    let max_h = if output_mode == OutputMode::Unicode {
                        term_h
                    } else {
                        term_h * 2
                    }
                    .saturating_sub(2);

                    if img.width() > max_w || img.height() > max_h {
                        img = img.resize(max_w, max_h, image::imageops::FilterType::Nearest);
                    }
                }

                let stdout = io::stdout();
                let mut writer = BufWriter::new(stdout.lock());
                write_ansi_art(&img, &mut writer, output_mode)?;
                writer.flush()?;
            } else {
                eprintln!("Error: Could not find '{name}'");
            }
        }
    }
    Ok(())
}

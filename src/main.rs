#![allow(clippy::multiple_crate_versions)]
use anyhow::Result;
use clap::{Parser, ValueEnum}; // Added ValueEnum
use image::imageops::FilterType;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::write_ansi_art;

// 1. Define an Enum for the CLI argument
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
enum ResizeFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

// 2. Add helper to convert CLI enum to image::FilterType
impl From<ResizeFilter> for FilterType {
    fn from(f: ResizeFilter) -> Self {
        match f {
            ResizeFilter::Nearest => Self::Nearest,
            ResizeFilter::Triangle => Self::Triangle,
            ResizeFilter::CatmullRom => Self::CatmullRom,
            ResizeFilter::Gaussian => Self::Gaussian,
            ResizeFilter::Lanczos3 => Self::Lanczos3,
        }
    }
}

#[derive(Parser)]
#[command(
    name = "px2ansi",
    version,
    about = "Convert pixel art to ANSI terminal art"
)]
struct Cli {
    /// Input image file
    filename: String,

    /// Output file (optional). If not provided, prints to stdout.
    #[arg(short, long)]
    output: Option<String>,

    /// Force a specific width (disables auto-resizing to terminal)
    #[arg(long)]
    width: Option<u32>,

    /// Resize filter to use (default: lanczos3).
    /// Use 'nearest' for pixel art to keep hard edges.
    #[arg(long, value_enum, default_value_t = ResizeFilter::Lanczos3)]
    filter: ResizeFilter,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Load image
    let mut reader = image::ImageReader::open(&cli.filename)?;
    reader.no_limits();
    let mut img = reader.decode()?;

    // 2. Determine Target Size
    let target_width = cli.width.or_else(|| {
        if let Some((Width(term_w), Height(term_h))) = terminal_size() {
            let max_w = u32::from(term_w);
            let max_h = u32::from(term_h) * 2;
            let img_w = img.width();
            let img_h = img.height();

            if img_w > max_w || img_h > max_h {
                let width_ratio = f64::from(max_w) / f64::from(img_w);
                let height_ratio = f64::from(max_h) / f64::from(img_h);
                let scale = width_ratio.min(height_ratio);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some((f64::from(img_w) * scale).round() as u32)
            } else {
                None
            }
        } else if img.width() > 100 {
            // Fallback: Only resize if wider than 100 and no terminal size found
            Some(100)
        } else {
            None
        }
    });

    // 3. Resize if needed
    if let Some(w) = target_width {
        let safe_w = w.max(1);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let new_height =
            (f64::from(img.height()) * (f64::from(safe_w) / f64::from(img.width()))) as u32;

        // Use the user-selected filter
        img = img.resize(safe_w, new_height, cli.filter.into());
    }

    // 4. Output
    if let Some(output_path) = cli.output {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        write_ansi_art(&img, &mut writer)?;
    } else {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        write_ansi_art(&img, &mut writer)?;
        writer.flush()?;
    }

    Ok(())
}

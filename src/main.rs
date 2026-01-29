use anyhow::Result;
use clap::{Parser, ValueEnum}; // Added ValueEnum
use image::imageops::FilterType;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use terminal_size::{Height, Width, terminal_size};

use px2ansi_rs::write_ansi_art;

// 1. Define an Enum for the CLI argument
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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
            ResizeFilter::Nearest => FilterType::Nearest,
            ResizeFilter::Triangle => FilterType::Triangle,
            ResizeFilter::CatmullRom => FilterType::CatmullRom,
            ResizeFilter::Gaussian => FilterType::Gaussian,
            ResizeFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

#[derive(Parser)]
#[command(name = "px2ansi")]
#[command(about = "Convert pixel art to ANSI terminal art")]
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
    let target_width = if let Some(w) = cli.width {
        Some(w)
    } else if let Some((Width(term_w), Height(term_h))) = terminal_size() {
        let max_w = term_w as u32;
        let max_h = (term_h as u32) * 2;

        let img_w = img.width();
        let img_h = img.height();

        if img_w > max_w || img_h > max_h {
            let width_ratio = max_w as f64 / img_w as f64;
            let height_ratio = max_h as f64 / img_h as f64;
            let scale = width_ratio.min(height_ratio);
            Some((img_w as f64 * scale) as u32)
        } else {
            None
        }
    } else {
        // Fallback: Only resize the image if its wider than 100
        if img.width() > 100 { Some(100) } else { None }
    };

    // 3. Resize if needed
    if let Some(w) = target_width {
        let safe_w = w.max(1);
        let new_height = (img.height() as f64 * (safe_w as f64 / img.width() as f64)) as u32;

        // CHANGE: Use the user-selected filter
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

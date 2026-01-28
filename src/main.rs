use anyhow::Result;
use clap::Parser;
use px2ansi_rs::write_ansi_art;
use std::fs::File;
use std::io::{self, BufWriter, Write};

#[derive(Parser)]
#[command(name = "px2ansi")]
#[command(about = "Convert pixel art to ANSI terminal art")]
struct Cli {
    /// Input image file
    filename: String,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load image
    let img = image::ImageReader::open(&cli.filename)?.decode()?;

    // Output
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

use anyhow::Result;
use clap::Parser;
use px2ansi_rs::image_to_ansi;
use std::fs::File;
use std::io::Write;

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

    // Convert to ANSI
    let ansi_art = image_to_ansi(&img);

    // Output
    if let Some(output_path) = cli.output {
        let mut file = File::create(output_path)?;
        file.write_all(ansi_art.as_bytes())?;
    } else {
        print!("{}", ansi_art);
    }

    Ok(())
}

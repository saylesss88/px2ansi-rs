use anyhow::Result;
use px2ansi_rs::RenderOptions;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

/// Parameters for converting a single image file to ANSI art
#[derive(Debug)]
pub struct ConvertCmd {
    /// Path to the source image.
    pub input: PathBuf,
    /// Optional path to save the ANSI text output. If None, prints to stdout.
    pub output: Option<PathBuf>,
    /// Optional path to save a PNG rasterization of the result.
    pub output_image: Option<PathBuf>,
    /// Visual settings (width, filter, style).
    pub render: RenderOptions,
}

impl ConvertCmd {
    /// Reads the input image, renders it to ANSI using the provided options,
    /// and handles routing the result to the filesystem or standard output.
    pub fn run(&self) -> Result<()> {
        // 1. Load and decode the image
        let img = image::ImageReader::open(&self.input)?.decode()?;

        // 2. Render to a buffer (we use a buffer so we can reuse it for PNG rasterization)
        let mut buf = Vec::new();
        self.render.render_centered(&img, &mut buf)?;

        // 3. Handle Output (File vs Stdout)
        if let Some(path) = &self.output {
            let file = std::fs::File::create(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&buf)?;
        } else {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout.lock());
            writer.write_all(&buf)?;
        }

        // 4. Handle optional PNG rasterization
        if let Some(png_path) = &self.output_image {
            let rasterized = px2ansi_rs::rasterize::rasterize_ansi(&buf)?;
            rasterized.save(png_path)?;
            println!("✅ Saved preview to {}", png_path.display());
        }
        Ok(())
    }
}

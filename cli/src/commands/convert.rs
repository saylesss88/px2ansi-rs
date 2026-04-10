use anyhow::Result;
use px2ansi::render::RenderOptions;
use std::io::Write;
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
    /// Runs the command.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the provided writer fails or if
    /// the command logic encounters a processing error.
    pub fn run<W: Write>(&self, external_writer: &mut W) -> Result<()> {
        // 1. Load and decode
        let img = image::ImageReader::open(&self.input)?.decode()?;

        // 2. Setup the file writer if needed
        let mut file_writer = self
            .output
            .as_ref()
            .map(|path| std::fs::File::create(path).map(std::io::BufWriter::new))
            .transpose()?;

        // 3. Render and Rasterize logic
        if let Some(png_path) = self.output_image.as_ref() {
            // Buffer is required for PNG rasterization
            let mut buf = Vec::with_capacity(img.width() as usize * img.height() as usize * 2);
            self.render.render_centered(&img, &mut buf)?;

            // Resolve where to write the buffer (File or External)
            let target: &mut dyn Write = match file_writer.as_mut() {
                Some(fw) => fw,
                None => external_writer,
            };

            target.write_all(&buf)?;
            target.flush()?;

            // Handle optional PNG rasterization
            let rasterized = px2ansi::rasterize::rasterize_ansi(&buf)?;
            rasterized.save(png_path)?;

            // Log to terminal
            writeln!(
                external_writer,
                "✅ Saved preview to {}",
                png_path.display()
            )?;
        } else {
            // FAST PATH: No preview image needed, stream directly
            let mut target: &mut dyn Write = match file_writer.as_mut() {
                Some(fw) => fw,
                None => external_writer,
            };

            self.render.render_centered(&img, &mut target)?;
            target.flush()?;
        }

        Ok(())
    }
}

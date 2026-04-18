use super::types::{CharsetMode, Density, RenderStyle};
use crate::cli_enums::{RenderStylePreset, ResizeFilter};
use crate::render::{ColorMode, get_terminal_size};
use image::{DynamicImage, imageops::FilterType};
use std::io::Write;

/// The master configuration for the rendering pipeline.
///
/// This struct determines how an image is resized, which characters are used,
/// and how it eventually looks in the terminal.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    width: Option<u32>,
    filter: FilterType,
    charset: CharsetMode,
    style: RenderStyle,
    color: bool,
    color_mode: ColorMode,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: None,
            filter: FilterType::Nearest,
            charset: CharsetMode::Ansi,
            style: RenderStyle::default(),
            color: true, // color on by default
            color_mode: ColorMode::detect(),
        }
    }
}

impl From<RenderStylePreset> for RenderOptions {
    fn from(preset: RenderStylePreset) -> Self {
        let mut opts = Self::default();
        match preset {
            RenderStylePreset::Ansi => opts.charset = CharsetMode::Ansi,
            RenderStylePreset::Unicode => opts.charset = CharsetMode::Unicode,
            RenderStylePreset::Braille => opts.charset = CharsetMode::Braille,
            RenderStylePreset::Fade => opts.charset = CharsetMode::Fade,
            RenderStylePreset::Ascii => opts.charset = CharsetMode::Ascii,
            RenderStylePreset::Kanji => opts.charset = CharsetMode::Kanji,
            RenderStylePreset::Chinese => opts.charset = CharsetMode::Chinese,
            RenderStylePreset::FullBlock => {
                opts.charset = CharsetMode::Unicode;
                opts.style.full = true;
            }
            RenderStylePreset::Dense => {
                opts.charset = CharsetMode::Ascii;
                opts.style.density = Density::Heavy;
            }
            RenderStylePreset::Sixel => opts.charset = CharsetMode::Sixel,
        }
        opts
    }
}

/// A builder for constructing [`RenderOptions`] with a fluent interface.
///
/// This allows for optional overrides on top of a [`RenderStylePreset`].
#[derive(Default)]
pub struct RenderOptionsBuilder {
    preset: Option<RenderStylePreset>,
    density: Option<Density>,
    width: Option<u32>,
    filter: Option<ResizeFilter>,
    color: bool,
    color_mode: Option<ColorMode>,
}

impl RenderOptionsBuilder {
    /// Creates a new builder instance with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets a high-level preset, such as ANSI or Braille.
    /// Presets provide baseline charset and style defaults.
    #[must_use]
    pub fn preset(mut self, preset: RenderStylePreset) -> Self {
        self.preset = Some(preset);
        self
    }

    /// Sets the character density for the rendering output.
    #[must_use]
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Sets the target width for the rendered output.
    /// If `None`, the output may scale to the terminal width.
    #[must_use]
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the resampling filter used when resizing the input image.
    #[must_use]
    pub fn filter(mut self, filter: ResizeFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Enables or disables color output.
    #[must_use]
    pub fn color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Sets the specific color mode (e.g., `TrueColor`, 256-color) for the output.
    #[must_use]
    pub fn with_color_mode(mut self, color_mode: ColorMode) -> Self {
        self.color_mode = Some(color_mode);
        self
    }

    /// Finalizes the builder and returns a configured [`RenderOptions`].
    ///
    /// This follows a specific priority:
    /// 1. If a preset is provided, it provides the base configuration.
    /// 2. Any explicitly set fields on this builder will override the preset's values.
    #[must_use]
    pub fn build(self) -> RenderOptions {
        // 1. Start with the preset's defaults, or the global defaults if no preset
        let mut opts = self.preset.map(RenderOptions::from).unwrap_or_default();

        // 2. Apply explicit builder overrides
        if let Some(d) = self.density {
            opts.style.density = d;
        }
        if let Some(w) = self.width {
            opts.width = Some(w);
        }
        if let Some(f) = self.filter {
            opts.filter = f.into();
        }

        opts.color = self.color;
        opts
    }
}

impl RenderOptions {
    /// Returns a new builder to configure rendering options.
    #[must_use]
    pub fn builder() -> RenderOptionsBuilder {
        RenderOptionsBuilder {
            color: true, // default to color on
            ..Default::default()
        }
    }

    /// Creates options based on a predefined visual style.
    #[must_use]
    pub fn with_preset(preset: RenderStylePreset) -> Self {
        Self::from(preset)
    }

    /// Returns the target width for rendering, if set.
    #[must_use]
    pub const fn width(&self) -> Option<u32> {
        self.width
    }

    /// Returns the current image resizing filter.
    #[must_use]
    pub const fn filter(&self) -> FilterType {
        self.filter
    }

    /// Returns the character set mode used for the output.
    #[must_use]
    pub const fn charset(&self) -> CharsetMode {
        self.charset
    }

    /// Returns the specific rendering style configuration.
    #[must_use]
    pub const fn style(&self) -> RenderStyle {
        self.style
    }

    /// Returns true if color output is enabled.
    #[must_use]
    pub const fn color(&self) -> bool {
        self.color
    }

    /// Disables color output and returns the modified options.
    #[must_use]
    pub const fn no_color(mut self) -> Self {
        self.color = false;
        self
    }
    /// Returns the current color mode configuration.
    #[must_use]
    pub const fn color_mode(&self) -> ColorMode {
        self.color_mode
    }
    /// Prepares a `DynamicImage` for the terminal by resizing it to fit the
    /// calculated constraints.  
    #[must_use]
    pub fn prepare_image(&self, img: &DynamicImage) -> DynamicImage {
        let (width, height) = self.calculate_dimensions(img.width(), img.height());
        img.resize_exact(width, height, self.filter)
    }

    /// Renders a pre-processed image to the provided writer.
    ///
    /// This is a low-level method that bypasses automatic resizing and centering.
    /// It is ideal for power users who want to handle image scaling or
    /// layout (like custom padding) manually.
    ///
    /// # Arguments
    ///
    /// * `prepared_img` - A [`DynamicImage`] that should already be resized to the
    ///   desired terminal dimensions.
    /// * `writer` - Any type implementing [`std::io::Write`] (e.g., `stdout`, a file, or a `Vec<u8>`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use px2ansi::RenderOptions;
    /// use image::{DynamicImage, RgbaImage};
    ///
    /// let opts = RenderOptions::default();
    /// let mut buf = Vec::new();
    ///
    /// // Construct a tiny 4x4 synthetic image
    /// let raw = RgbaImage::new(4, 4);
    /// let img = DynamicImage::ImageRgba8(raw);
    /// opts.render(&img, &mut buf).unwrap();
    ///
    /// // Output should be non-empty ANSI bytes
    /// assert!(!buf.is_empty());
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// * The underlying rendering logic fails to process a specific pixel format.
    /// * An I/O error occurs while writing to the provided `writer`.
    pub fn render<W: Write>(
        &self,
        prepared_img: &DynamicImage,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        crate::render::write_ansi_art(prepared_img, writer, *self)?;
        Ok(())
    }

    /// This method calculates the horizontal padding required to center the output,
    /// captures the rendered ANSI art into an internal buffer, and then writes it
    /// line-by-line to the provided writer with the calculated offset.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The underlying rendering engine ([`write_ansi_art`][crate::write_ansi_art])
    ///   fails to process the image.
    /// * An allocation or I/O error occurs while buffering or writing the rendered output.
    pub fn render_centered<W: Write>(
        &self,
        img: &DynamicImage,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        const BYTES_PER_PIXEL_ESTIMATE: usize = 25;
        let prepared = self.prepare_image(img);

        // Get rendered width in terminal columns
        let rendered_cols = match self.charset {
            CharsetMode::Braille => prepared.width() / 2,
            CharsetMode::Unicode if self.style.full => prepared.width() * 2,
            CharsetMode::Kanji | CharsetMode::Chinese => prepared.width() * 2, // Kanji/Chinese is double-width
            _ => prepared.width(),
        };

        let (term_w, _) = get_terminal_size();

        let padding = if term_w > rendered_cols {
            (term_w - rendered_cols) / 2
        } else {
            0
        };

        let pad_str = " ".repeat(padding as usize);

        // Capture the render into a pre-sized buffer, then prefix each line
        // with padding.
        //
        // Capacity estimate: a truecolor ANSI cell (▀/▄) with both fg and bg
        // escape sequences is roughly `\x1b[38;2;255;255;255m\x1b[48;2;255;255;255m▀`
        // ≈ 40 bytes, and each cell represents 2 source pixels, giving ~20 bytes
        // per pixel.  A 25-byte estimate adds a comfortable margin for resets
        // ("\x1b[0m") and newlines without over-allocating for other modes.
        let estimated_capacity =
            prepared.width() as usize * prepared.height() as usize * BYTES_PER_PIXEL_ESTIMATE;
        let mut buf = Vec::with_capacity(estimated_capacity);
        crate::write_ansi_art(&prepared, &mut buf, *self)?;

        for line in buf.split(|&b| b == b'\n') {
            if !line.is_empty() {
                write!(writer, "{pad_str}")?;
                writer.write_all(line)?;
                writeln!(writer)?;
            }
        }

        Ok(())
    }
}

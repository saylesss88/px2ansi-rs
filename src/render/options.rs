use super::types::{CharsetMode, Density, RenderStyle};
use crate::cli_enums::{RenderStylePreset, ResizeFilter};
use crate::render::get_terminal_size;
use image::{DynamicImage, imageops::FilterType};
use std::io::Write;

/// The master configuration for the rendering pipeline.
///
/// This struct determines how an image is resized, which characters are used,
/// and how it eventually looks in the terminal.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    pub target_width: Option<u32>,
    pub filter: FilterType,
    pub charset: CharsetMode,
    pub style: RenderStyle,
    pub color: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            target_width: None,
            filter: FilterType::Lanczos3,
            charset: CharsetMode::Ansi,
            style: RenderStyle::default(),
            color: true, // color on by default
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
        }
        opts
    }
}
#[derive(Default)]
pub struct RenderOptionsBuilder {
    style: Option<RenderStylePreset>,
    density: Option<Density>,
    width: Option<u32>,
    filter: Option<ResizeFilter>,
    color: bool,
}

impl RenderOptionsBuilder {
    /// Sets the base preset style (e.g. Ansi, Braille).
    /// This provides the baseline defaults for the build process.
    #[must_use]
    pub const fn style(mut self, style: Option<RenderStylePreset>) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn density(mut self, density: Option<Density>) -> Self {
        self.density = density;
        self
    }

    #[must_use]
    pub const fn width(mut self, width: Option<u32>) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn filter(mut self, filter: Option<ResizeFilter>) -> Self {
        self.filter = filter;
        self
    }

    #[must_use]
    pub const fn color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub fn build(self) -> RenderOptions {
        // 1. Start with the preset's defaults, or the global defaults if no preset
        let mut opts = self.style.map(RenderOptions::from).unwrap_or_default();

        // 2. Apply granular overrides from CLI flags
        if let Some(d) = self.density {
            opts.style.density = d;
        }
        if let Some(w) = self.width {
            opts.target_width = Some(w);
        }
        if let Some(f) = self.filter {
            opts.filter = f.into();
        }

        opts.color = self.color;
        opts
    }
}

impl RenderOptions {
    #[must_use]
    pub fn builder() -> RenderOptionsBuilder {
        RenderOptionsBuilder {
            color: true, // default to color on
            ..Default::default()
        }
    }
}
impl RenderOptions {
    /// Prepares a `DynamicImage` for the terminal by resizing it to fit the
    /// calculated constraints.  
    #[must_use]
    pub fn prepare_image(&self, img: &DynamicImage) -> DynamicImage {
        let (width, height) = self.calculate_dimensions(img.width(), img.height());
        img.resize_exact(width, height, self.filter)
    }

    /// This method calculates the horizontal padding required to center the output,
    /// captures the rendered ANSI art into an internal buffer, and then writes it
    /// line-by-line to the provided writer with the calculated offset.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The underlying rendering engine ([`write_ansi_art`][crate::render::write_ansi_art])
    ///   fails to process the image.
    /// * An I/O error occurs while writing the padding or the image data to the `writer`.
    /// * The system fails to allocate memory for the internal buffer used to
    ///   calculate line breaks for centering.rendering if terminal width can't be determined.
    pub fn render_centered<W: Write>(
        &self,
        img: &DynamicImage,
        writer: &mut W,
    ) -> anyhow::Result<()> {
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

        // Capture the render into a buffer, then prefix each line with padding
        let mut buf = Vec::new();
        crate::render::write_ansi_art(&prepared, &mut buf, *self)?;

        for line in buf.split(|&b| b == b'\n') {
            if !line.is_empty() {
                write!(writer, "{pad_str}")?;
                writer.write_all(line)?;
                writeln!(writer)?;
            }
        }

        Ok(())
    }
    /// Creates a new configuration instance by overriding default values with
    /// optional CLI arguments.
    ///
    /// This serves as a bridge between raw command-line input and the internal
    /// configuration state, mapping presets to specific charset and style behaviors.
    ///
    /// # Arguments
    ///
    /// * `style` - An optional preset that defines the character set and rendering density.
    /// * `width` - An optional target width in columns.
    /// * `filter` - An optional sampling filter used for resizing the input.
    ///
    /// # Errors
    ///
    /// Currently, this function is infallible and will always return `Ok`. However,
    /// it returns a [`anyhow::Result`] to maintain API compatibility for future
    /// validations, such as:
    /// * Validating that `width` is within a supported range.
    /// * Checking for terminal capability conflicts with the selected `RenderStylePreset`.
    pub fn from_cli(
        style: Option<RenderStylePreset>,
        density: Option<Density>,
        width: Option<u32>,
        filter: Option<ResizeFilter>,
        no_color: bool,
    ) -> anyhow::Result<Self> {
        Ok(Self::builder()
            .style(style)
            .density(density)
            .width(width)
            .filter(filter)
            .color(!no_color)
            .build())
    }
}

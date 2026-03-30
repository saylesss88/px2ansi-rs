use std::io::Write;
use std::str::FromStr;

use image::DynamicImage;
use image::imageops::FilterType;
use terminal_size::{Height, Width, terminal_size};

use crate::cli_enums::{RenderStylePreset, ResizeFilter};

/// Defines the character set used to represent pixels in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetMode {
    #[default]
    /// High-resolution mode using half-blocks (▀/▄).
    Ansi,
    /// Flexible mode using either full or half blocks based on the render style.
    Unicode,
    /// Maximum density mode using 2x4 Braille dot patterns.
    Braille,
    /// A small 4-character ramp ( ░▒▓█) for a "faded" or shaded look.
    Fade,
    /// Traditional 92-character density ramp for classic ASCII art.
    Ascii,

    Kanji,
}

impl FromStr for CharsetMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ansi" | "block" => Ok(Self::Ansi),
            "unicode" | "uni" => Ok(Self::Unicode),
            "braille" | "brl" => Ok(Self::Braille),
            "fade" | "grayscale" => Ok(Self::Fade),
            "kanji" | "jp" => Ok(Self::Kanji),
            "ascii" => Ok(Self::Ascii),
            _ => anyhow::bail!(
                "Invalid charset '{s}'. Use: ansi, unicode, braille, fade, ascii, kanji"
            ),
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
            RenderStylePreset::FullBlock => {
                opts.charset = CharsetMode::Unicode;
                opts.style.full = true;
            }
            RenderStylePreset::Dense => {
                opts.charset = CharsetMode::Unicode;
                opts.style.density = Density::Heavy;
            }
        }
        opts
    }
}
/// Aesthetic density settings for the rendered output.
#[derive(Clone, Copy, Debug, Default)]
pub enum Density {
    #[default]
    Medium,
    Light,
    Heavy,
}

/// Combines physical character choice with layout logic (like full-width vs half-width).
#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
    /// If true, uses double-width characters (██) to force a 1:1 pixel aspect ratio.
    pub full: bool,
    pub density: Density,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            full: false,
            density: Density::Medium,
        }
    }
}

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
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            target_width: None,
            filter: FilterType::Lanczos3,
            charset: CharsetMode::Ansi,
            style: RenderStyle::default(),
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
            CharsetMode::Kanji => prepared.width() * 2, // Kanji is double-width
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
        width: Option<u32>,
        filter: Option<ResizeFilter>,
    ) -> anyhow::Result<Self> {
        let mut opts = style.map(Self::from).unwrap_or_default();
        if let Some(width) = width {
            opts.target_width = Some(width);
        }
        if let Some(filter) = filter {
            opts.filter = filter.into();
        }
        Ok(opts)
    }
    /// Calculates the optimal target dimensions for the terminal.
    ///
    /// This is the most complex part of the renderer, as it has to account for:
    /// 1. Terminal width/height (auto-detected).
    /// 2. Different character aspect ratios (Braille vs. Half-blocks).
    /// 3. User-defined width overrides.
    /// 4. Nearest-neighbor scaling for pixel art preservation.
    #[must_use]
    pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
        const MAX_SAFE: u32 = 16384;
        let (term_w, term_h) = get_terminal_size();
        let (max_w, max_h) = if term_w > 0 && term_h > 0 {
            match self.charset {
                CharsetMode::Braille => (term_w * 2, term_h * 4),
                CharsetMode::Unicode if self.style.full => (term_w / 2, term_h),
                // CharsetMode::Ascii | CharsetMode::Fade => (term_w.saturating_sub(2), term_h - 2),
                _ => (term_w.saturating_sub(2), term_h * 2 / 3),
            }
        } else {
            (80, 40)
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (render_w, render_h) = self.target_width.map_or_else(
            || {
                if self.filter == FilterType::Nearest && orig_w < 120 {
                    let scale_w = (f64::from(max_w) / f64::from(orig_w)).floor();
                    let scale_h = (f64::from(max_h) / f64::from(orig_h)).floor();
                    let scale = scale_w.min(scale_h).max(1.0);
                    (
                        (f64::from(orig_w) * scale) as u32,
                        (f64::from(orig_h) * scale) as u32,
                    )
                } else {
                    let scale = (f64::from(max_w) / f64::from(orig_w))
                        .min(f64::from(max_h) / f64::from(orig_h));
                    (
                        (f64::from(orig_w) * scale).round() as u32,
                        (f64::from(orig_h) * scale).round() as u32,
                    )
                }
            },
            |tw| {
                let aspect = f64::from(orig_h) / f64::from(orig_w);
                (tw, (f64::from(tw) * aspect).round() as u32)
            },
        );

        (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE))
    }
}

/// Use Env vars to get the terminal size
fn get_terminal_size() -> (u32, u32) {
    let ts = terminal_size();
    let env_cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());
    let env_rows = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());

    if let Some((Width(w), Height(h))) = ts {
        return (u32::from(w), u32::from(h));
    }
    if let (Some(c), Some(r)) = (env_cols, env_rows) {
        return (c, r);
    }
    (80, 24)
}

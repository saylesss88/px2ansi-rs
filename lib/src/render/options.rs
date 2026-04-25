use super::types::{CharsetMode, Density, RenderStyle};
use crate::cli_enums::{RenderStylePreset, ResizeFilter};
use crate::RenderError;
use crate::{get_terminal_size, ColorMode};
use image::{imageops::FilterType, DynamicImage};
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
    color_mode: ColorMode,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: None,
            filter: FilterType::Nearest,
            charset: CharsetMode::Ansi,
            style: RenderStyle::default(),
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
#[derive(Default, Clone)]
pub struct RenderOptionsBuilder {
    preset: Option<RenderStylePreset>,
    density: Option<Density>,
    width: Option<u32>,
    filter: Option<ResizeFilter>,
    color_mode: Option<ColorMode>,
    dither: Option<bool>,
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
    pub const fn preset(mut self, preset: RenderStylePreset) -> Self {
        self.preset = Some(preset);
        self
    }

    /// Sets the character density for the rendering output.
    #[must_use]
    pub const fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Whether to enable dithering or not
    #[must_use]
    pub const fn dither(mut self, enabled: bool) -> Self {
        self.dither = Some(enabled);
        self
    }

    /// Sets the target width for the rendered output.
    /// If `None`, the output may scale to the terminal width.
    #[must_use]
    pub const fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the resampling filter used when resizing the input image.
    #[must_use]
    pub const fn filter(mut self, filter: ResizeFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sets the specific color mode (e.g., `TrueColor`, 256-color) for the output.
    #[must_use]
    pub const fn color_mode(mut self, color_mode: ColorMode) -> Self {
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
        if let Some(cm) = self.color_mode {
            opts.color_mode = cm;
        }
        if let Some(dither_val) = self.dither {
            opts.style.dither = dither_val;
        }
        opts
    }
}

impl RenderOptions {
    /// Returns a new builder to configure rendering options.
    #[must_use]
    pub fn builder() -> RenderOptionsBuilder {
        RenderOptionsBuilder::default()
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

    /// Returns the current color mode configuration.
    #[must_use]
    pub const fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    /// Prepares a [`DynamicImage`] for terminal rendering through resizing and optional dithering.
    ///
    /// This method handles the core image transformation pipeline:
    /// 1. Resizes the image to fit calculated terminal dimensions using the configured filter.
    /// 2. If enabled, applies a Floyd-Steinberg dither to the luminance channel.
    /// 3. In color modes, performs a luminance-preserving remap to distribute dithered
    ///    high-frequency detail while maintaining original hue and saturation.
    ///
    /// If the `parallel` feature is enabled, the color remapping stage is executed
    /// across multiple threads using `rayon`.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// * The calculated image dimensions exceed the capacity of a `u32` (though
    ///   `calculate_dimensions` usually prevents this).
    /// * An internal state inconsistency occurs where a pixel index cannot be mapped
    ///   back to valid coordinate space during parallel processing.
    ///
    /// # Performance
    ///
    /// Dither generation is inherently sequential due to the error-diffusion nature
    /// of the Floyd-Steinberg algorithm. However, the color-scaling pass is
    /// "embarrassingly parallel" and will scale linearly with available CPU cores
    /// when the `parallel` feature is active.
    #[must_use]
    pub fn prepare_image(&self, img: &DynamicImage) -> DynamicImage {
        const LUMA_R: f32 = 0.2126;
        const LUMA_G: f32 = 0.7152;
        const LUMA_B: f32 = 0.0722;

        let (width, height) = self.calculate_dimensions(img.width(), img.height());
        let mut resized = img.resize_exact(width, height, self.filter);

        if self.style.dither {
            let mut luma_img = resized.to_luma8();
            image::imageops::dither(&mut luma_img, &image::imageops::BiLevel);

            if self.color_mode == ColorMode::None {
                resized = DynamicImage::ImageLuma8(luma_img);
            } else {
                // Bind as immutable initially for the parallel path
                let rgba = resized.to_rgba8();

                #[cfg(feature = "parallel")]
                {
                    use rayon::prelude::*;
                    let raw = rgba.as_raw();
                    let w_usize = width as usize;

                    let pixels: Vec<[u8; 4]> = raw
                        .par_chunks_exact(4)
                        .enumerate()
                        .map(|(i, px)| {
                            // let x = (i % w_usize) as u32;
                            let x = u32::try_from(i % w_usize);
                            let y = u32::try_from(i / w_usize);
                            // let y = (i / w_usize) as u32;
                            let [red, green, blue, alpha] = [px[0], px[1], px[2], px[3]];

                            let orig_luma = LUMA_R.mul_add(
                                f32::from(red),
                                LUMA_G.mul_add(f32::from(green), LUMA_B * f32::from(blue)),
                            );

                            let dither_v =
                                f32::from(luma_img.get_pixel(x.unwrap_or(0), y.unwrap_or(0)).0[0]);

                            #[expect(
                                clippy::cast_possible_truncation,
                                clippy::cast_sign_loss,
                                reason = "value is clamped to the 0..255 range prior to conversion"
                            )]
                            let (nr, ng, nb) = if orig_luma > 0.0 {
                                let factor = dither_v / orig_luma;
                                (
                                    (f32::from(red) * factor).clamp(0.0, 255.0) as u8,
                                    (f32::from(green) * factor).clamp(0.0, 255.0) as u8,
                                    (f32::from(blue) * factor).clamp(0.0, 255.0) as u8,
                                )
                            } else if dither_v > 0.0 {
                                (255, 255, 255)
                            } else {
                                (0, 0, 0)
                            };

                            [nr, ng, nb, alpha]
                        })
                        .collect();

                    let mut new_raw = Vec::with_capacity(pixels.len() * 4);
                    for p in pixels {
                        new_raw.extend_from_slice(&p);
                    }

                    if let Some(buf) = image::ImageBuffer::from_raw(width, height, new_raw) {
                        resized = DynamicImage::ImageRgba8(buf);
                    } else {
                        resized = DynamicImage::ImageRgba8(rgba);
                    }
                }

                #[cfg(not(feature = "parallel"))]
                {
                    let mut rgba = rgba;
                    let (w, h) = rgba.dimensions();
                    for y in 0..h {
                        for x in 0..w {
                            let p = rgba.get_pixel_mut(x, y);
                            let [r, g, b, a] = p.0;

                            let orig_luma = LUMA_R.mul_add(
                                f32::from(r),
                                LUMA_G.mul_add(f32::from(g), LUMA_B * f32::from(b)),
                            );

                            let dither_v = f32::from(luma_img.get_pixel(x, y).0[0]);

                            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            let (nr, ng, nb) = if orig_luma > 0.0 {
                                let factor = dither_v / orig_luma;
                                (
                                    (f32::from(r) * factor).clamp(0.0, 255.0) as u8,
                                    (f32::from(g) * factor).clamp(0.0, 255.0) as u8,
                                    (f32::from(b) * factor).clamp(0.0, 255.0) as u8,
                                )
                            } else if dither_v > 0.0 {
                                (255, 255, 255)
                            } else {
                                (0, 0, 0)
                            };

                            *p = image::Rgba([nr, ng, nb, a]);
                        }
                    }
                    resized = DynamicImage::ImageRgba8(rgba);
                }
            }
        }

        resized
    } // pub fn prepare_image(&self, img: &DynamicImage) -> DynamicImage {
      //     const LUMA_R: f32 = 0.2126;
      //     const LUMA_G: f32 = 0.7152;
      //     const LUMA_B: f32 = 0.0722;
      //     let (width, height) = self.calculate_dimensions(img.width(), img.height());
      //     let mut resized = img.resize_exact(width, height, self.filter);

    //     if self.style.dither {
    //         // 1. Generate the dithered luma (serial; Floyd–Steinberg)
    //         let mut luma_img = resized.to_luma8();
    //         image::imageops::dither(&mut luma_img, &image::imageops::BiLevel);

    //         if self.color_mode == ColorMode::None {
    //             // Keep pure B/W luma image when user requested no color
    //             resized = DynamicImage::ImageLuma8(luma_img);
    //         } else {
    //             // Color-preserving remap: scale original RGB so its luminance becomes
    //             // the dithered (0 or 255) value while preserving hue where possible.
    //             let rgba = resized.to_rgba8();

    //             // --- Parallel hot path ---
    //             #[cfg(feature = "parallel")]
    //             {
    //                 use rayon::prelude::*;
    //                 // Read-only raw bytes
    //                 let raw = rgba.as_raw();
    //                 let w_usize = width as usize;

    //                 // Map input pixels -> adjusted [r,g,b,a] in parallel
    //                 let pixels: Vec<[u8; 4]> = raw
    //                     .par_chunks_exact(4)
    //                     .enumerate()
    //                     .map(|(i, px)| {
    //                         // compute coordinates from linear index
    //                         let x = u32::try_from(i % w_usize).unwrap_or(0);
    //                         let y = u32::try_from(i / w_usize).unwrap_or(0);
    //                         let red = px[0];
    //                         let green = px[1];
    //                         let blue = px[2];
    //                         let alpha = px[3];

    //                         let luma_red = f32::from(red);
    //                         let luma_green = f32::from(green);
    //                         let luma_blue = f32::from(blue);

    //                         // Integer luma formula (same as elsewhere)
    //                         let orig_luma = LUMA_R
    //                             .mul_add(luma_red, LUMA_G.mul_add(luma_green, LUMA_B * luma_blue));

    //                         let dither_v = f32::from(luma_img.get_pixel(x, y).0[0]);

    //                         #[expect(
    //                             clippy::cast_possible_truncation,
    //                             clippy::cast_sign_loss,
    //                             reason = "value is clamped to the 0..255 range prior to conversion"
    //                         )]
    //                         let (nr, ng, nb) = if orig_luma > 0.0 {
    //                             let factor = dither_v / orig_luma;
    //                             (
    //                                 (f32::from(red) * factor).clamp(0.0, 255.0) as u8,
    //                                 (f32::from(green) * factor).clamp(0.0, 255.0) as u8,
    //                                 (f32::from(blue) * factor).clamp(0.0, 255.0) as u8,
    //                             )
    //                         } else if dither_v > 0.0 {
    //                             (255u8, 255u8, 255u8)
    //                         } else {
    //                             (0u8, 0u8, 0u8)
    //                         };

    //                         [nr, ng, nb, alpha]
    //                     })
    //                     .collect();

    //                 // Flatten to raw bytes
    //                 let mut new_raw = Vec::with_capacity(pixels.len() * 4);
    //                 for p in pixels {
    //                     new_raw.extend_from_slice(&p);
    //                 }

    //                 // Rebuild image from raw bytes (should succeed)
    //                 if let Some(buf) = image::ImageBuffer::from_raw(width, height, new_raw) {
    //                     resized = DynamicImage::ImageRgba8(buf);
    //                 } else {
    //                     // Fallback to original rgba if something unexpected occurs
    //                     resized = DynamicImage::ImageRgba8(rgba);
    //                 }
    //             }

    //             // --- Serial fallback ---
    //             #[cfg(not(feature = "parallel"))]
    //             {
    //                 let (w, h) = rgba.dimensions();
    //                 for y in 0..h {
    //                     for x in 0..w {
    //                         let p = rgba.get_pixel_mut(x, y);
    //                         let [r, g, b, a] = p.0;

    //                         let orig_luma = (2126u32 * u32::from(r)
    //                             + 7152u32 * u32::from(g)
    //                             + 722u32 * u32::from(b))
    //                             as f32
    //                             / 10000.0;

    //                         let dither_v = f32::from(luma_img.get_pixel(x, y).0[0]);

    //                         let (nr, ng, nb) = if orig_luma > 0.0 {
    //                             let factor = dither_v / orig_luma;
    //                             (
    //                                 (f32::from(r) * factor).clamp(0.0, 255.0) as u8,
    //                                 (f32::from(g) * factor).clamp(0.0, 255.0) as u8,
    //                                 (f32::from(b) * factor).clamp(0.0, 255.0) as u8,
    //                             )
    //                         } else if dither_v > 0.0 {
    //                             (255u8, 255u8, 255u8)
    //                         } else {
    //                             (0u8, 0u8, 0u8)
    //                         };

    //                         *p = image::Rgba([nr, ng, nb, a]);
    //                     }
    //                 }
    //                 resized = DynamicImage::ImageRgba8(rgba);
    //             }
    //         }
    //     }

    //     resized
    // }

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
    ) -> Result<(), RenderError> {
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
    ) -> Result<(), RenderError> {
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

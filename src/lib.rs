#![allow(clippy::multiple_crate_versions)]
use clap::ValueEnum;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Rgba};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;
use terminal_size::{Height, Width, terminal_size};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    #[default]
    Ansi, // Standard half-blocks
    Unicode, // Full blocks or specialized chars (like pokemon-colorscripts)
}

impl FromStr for OutputMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unicode" => Ok(Self::Unicode),
            "ansi" | "block" => Ok(Self::Ansi),
            _ => anyhow::bail!("Invalid output mode '{s}'. Use 'ansi' or 'unicode'"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetMode {
    #[default]
    Ansi, // half-block ▀/▄
    Unicode, // full-block / half-block based on style.full
    Braille,
    Fade,
    Ascii,
}

impl FromStr for CharsetMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ansi" | "block" => Ok(Self::Ansi),
            "unicode" | "uni" => Ok(Self::Unicode),
            "braille" | "brl" => Ok(Self::Braille),
            "fade" | "grayscale" => Ok(Self::Fade),
            "ascii" => Ok(Self::Ascii),
            _ => anyhow::bail!("Invalid charset '{s}'. Use: ansi, unicode, braille, fade, ascii"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Density {
    #[default]
    Medium,
    Light,
    Heavy,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderStyle {
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
/// Configuration for how an image should be processed and rendered to the terminal.
///
/// This handles the "look and feel" of the output, including the character set
/// (ANSI vs Unicode), scaling filters, and whether to use half-block positioning.#
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Resizes a `DynamicImage` based on the current options and terminal constraints.
    #[must_use]
    pub fn prepare_image(&self, img: &DynamicImage) -> DynamicImage {
        let (width, height) = self.calculate_dimensions(img.width(), img.height());
        img.resize_exact(width, height, self.filter)
    }

    /// Constructs a new `RenderOptions` by merging optional CLI arguments into the defaults.
    ///
    /// This method applies a hierarchy of configuration: it starts with [`RenderOptions::default()`],
    /// then overrides specific fields if the corresponding CLI argument is `Some`.
    ///
    /// Note that `style` presets (like `Dense` or `FullAnsi`) may modify multiple internal
    /// fields (e.g., both `charset` and `style.full`) simultaneously.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// * The `mode` string is provided but fails to parse into a valid [`OutputMode`].
    ///   Valid values are typically "ansi", "block", or "unicode".
    /// * Any internal parsing or conversion logic encountered via `anyhow` fails.
    pub fn from_cli(
        // mode: Option<String>,
        // full: Option<bool>,
        style: Option<RenderStylePreset>,
        width: Option<u32>,
        filter: Option<ResizeFilter>,
    ) -> anyhow::Result<Self> {
        let mut opts = Self::default();

        // if let Some(mode) = mode {
        //     opts.output_mode = mode.parse()?;
        // }

        // if let Some(full) = full {
        //     opts.full = full;
        //     opts.style.full = full;
        // }

        if let Some(style) = style {
            match style {
                RenderStylePreset::Ansi => opts.charset = CharsetMode::Ansi,
                RenderStylePreset::Unicode => opts.charset = CharsetMode::Unicode,
                RenderStylePreset::Braille => opts.charset = CharsetMode::Braille,
                RenderStylePreset::Fade => opts.charset = CharsetMode::Fade,
                RenderStylePreset::Ascii => opts.charset = CharsetMode::Ascii,
                RenderStylePreset::FullBlock => {
                    opts.charset = CharsetMode::Unicode;
                    opts.style.full = true;
                }
                RenderStylePreset::Dense => {
                    opts.charset = CharsetMode::Unicode;
                    opts.style.full = false;
                    opts.style.density = Density::Heavy;
                }
            }
        }

        if let Some(width) = width {
            opts.target_width = Some(width);
        }

        if let Some(filter) = filter {
            opts.filter = filter.into();
        }

        Ok(opts)
    }
    #[must_use]
    pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
        const MAX_SAFE: u32 = 16384;

        // 1. Determine the "canvas" size (Terminal or Default)
        let term_dims =
            terminal_size().map(|(Width(tw), Height(th))| (u32::from(tw), u32::from(th)));

        let (max_w, max_h) = if let Some((tw, th)) = term_dims {
            let (mw, mh) = match self.charset {
                CharsetMode::Braille => (tw / 4, th / 2), // 4x2 chars → 8x4 pixels
                CharsetMode::Unicode if self.style.full => (tw / 2, th),
                _ => (tw.saturating_sub(2), th * 2 / 3), // half-blocks
            };
            (mw, mh)
        } else {
            (80, 40)
        };
        // 2. Run the scaling logic
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let (render_w, render_h) = self.target_width.map_or_else(
            || {
                if self.filter == FilterType::Nearest && orig_w < 120 {
                    // --- CRISP SPRITE LOGIC ---
                    let scale_w = (f64::from(max_w) / f64::from(orig_w)).floor();
                    let scale_h = (f64::from(max_h) / f64::from(orig_h)).floor();
                    let scale = scale_w.min(scale_h).max(1.0);

                    (
                        (f64::from(orig_w) * scale) as u32,
                        (f64::from(orig_h) * scale) as u32,
                    )
                } else {
                    // --- NORMAL MODE ---
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

        // 3. Clamp and return
        (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE))
    }

    /// Renders a `DynamicImage` to the provided writer using ANSI escape codes.
    ///
    /// This method prepares the image according to the current configuration
    /// (scaling, filtering, etc.) and writes the resulting pixel art to `writer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The image preparation or transformation fails (e.g., issues with resizing).
    /// * Writing to the `writer` fails (e.g., a broken pipe or insufficient permissions).
    /// * The underlying ANSI conversion encounters an invalid color or pixel format.
    pub fn render<W: Write>(&self, img: &DynamicImage, writer: &mut W) -> anyhow::Result<()> {
        let prepared = self.prepare_image(img);
        write_ansi_art(&prepared, writer, *self)?; // ← *self, not AnsiArtOptions::from(*self)
        Ok(())
    }
}
/// Renders an image into the terminal using ANSI escape sequences.
///
/// Depending on the `mode`, this will either:
/// - **Ansi**: Squash two vertical pixels into one character cell using half-blocks (▀/▄).
/// - **Unicode**: Render each pixel as a double-width square block (██) for a retro look.
/// # Errors
///
/// This function returns an [`std::io::Result::Err`] if:
/// * The provided output writer `out` fails to write the generated bytes.
/// * There is an issue flushing the buffer to the terminal or file.
///
/// Note: This function does not currently validate image dimensions; however,
/// passing extremely large images may result in performance issues depending
/// Main rendering dispatch - handles all charset modes
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    writer: &mut W,
    options: RenderOptions,
) -> std::io::Result<()> {
    // eprintln!("DEBUG: using charset={:?}", options.charset);
    match options.charset {
        CharsetMode::Ansi => write_ansi_blocks(writer, img),
        CharsetMode::Unicode => write_unicode_blocks(writer, img, options.style.full),
        CharsetMode::Braille => write_braille(writer, img, options),
        CharsetMode::Fade => write_fade(writer, img, options),
        CharsetMode::Ascii => write_ascii(writer, img, options),
    }
}
/// ANSI half-blocks (2px per cell) - your existing logic
fn write_ansi_blocks<W: Write>(writer: &mut W, img: &DynamicImage) -> std::io::Result<()> {
    let (width, height) = img.dimensions();
    for y in (0..height).step_by(2) {
        for x in 0..width {
            let top = img.get_pixel(x, y);
            let bot = if y + 1 < height {
                img.get_pixel(x, y + 1)
            } else {
                Rgba([0, 0, 0, 0])
            };
            write_half_block(writer, top, bot)?;
        }
        writeln!(writer, "\x1b[0m")?;
    }
    Ok(())
}

/// Unicode blocks (full or half, based on `full` flag)
fn write_unicode_blocks<W: Write>(
    writer: &mut W,
    img: &DynamicImage,
    full_block: bool,
) -> std::io::Result<()> {
    let (width, height) = img.dimensions();
    if full_block {
        for y in 0..height {
            for x in 0..width {
                let px = img.get_pixel(x, y);
                write_full_block(writer, px)?;
            }
            writeln!(writer, "\x1b[0m")?;
        }
    } else {
        // Fallback to ANSI half-blocks
        write_ansi_blocks(writer, img)?;
    }
    Ok(())
}

/// A low-level helper that squashes two vertical pixels into a single terminal character cell.
///
/// It uses the 'upper half block' character (▀) and clever color manipulation:
/// - The **foreground** color is set to the `top` pixel.
/// - The **background** color is set to the `bot` pixel.
///
/// If one of the pixels is transparent (alpha = 0), it switches to a half-block
/// or space with a transparent background to let the terminal's own theme show through.
fn write_half_block<W: Write>(out: &mut W, top: Rgba<u8>, bot: Rgba<u8>) -> std::io::Result<()> {
    match (top[3] > 0, bot[3] > 0) {
        (true, true) => write!(
            out,
            "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
            top[0], top[1], top[2], bot[0], bot[1], bot[2]
        ),
        (true, false) => write!(out, "\x1b[38;2;{};{};{}m\x1b[49m▀", top[0], top[1], top[2]),
        (false, true) => write!(out, "\x1b[38;2;{};{};{}m\x1b[49m▄", bot[0], bot[1], bot[2]),
        (false, false) => write!(out, "\x1b[0m "),
    }
}

/// Renders a single pixel as a wide, solid block using two 'full block' characters (██).
///
/// Since terminal character cells are usually twice as tall as they are wide,
/// printing two characters for every one pixel preserves a square aspect ratio.
/// This is the technique used by tools like `pokemon-colorscripts`.
///
/// If the pixel is transparent, it simply prints two spaces.
fn write_full_block<W: Write>(out: &mut W, px: Rgba<u8>) -> std::io::Result<()> {
    if px[3] > 0 {
        // Print TWO blocks for every ONE pixel
        write!(out, "\x1b[38;2;{};{};{}m██", px[0], px[1], px[2])
    } else {
        write!(out, "  ") // Two spaces
    }
}

/// Braille rendering — 2x4 pixels per cell, with per-cell foreground color
fn write_braille<W: Write>(
    writer: &mut W,
    img: &image::DynamicImage,
    _options: RenderOptions,
) -> std::io::Result<()> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Braille dot layout (dx, dy, bitmask)
    // Unicode Braille dot order:
    // col0 col1
    //  1    4   (dy=0)
    //  2    5   (dy=1)
    //  3    6   (dy=2)
    //  7    8   (dy=3)
    let dots: [(u32, u32, u8); 8] = [
        (0, 0, 0x01),
        (0, 1, 0x02),
        (0, 2, 0x04),
        (1, 0, 0x08),
        (1, 1, 0x10),
        (1, 2, 0x20),
        (0, 3, 0x40),
        (1, 3, 0x80),
    ];

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(2) {
            let mut byte = 0u8;
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut lit_count = 0u32;

            for (dx, dy, bit) in dots {
                let px_x = x + dx;
                let px_y = y + dy;
                if px_x < width && px_y < height {
                    let px = rgba.get_pixel(px_x, px_y);
                    let [r, g, b, a] = px.0;

                    if a > 128 {
                        // Perceptual luminance (Rec. 709 coefficients)
                        #[allow(clippy::cast_possible_truncation)]
                        #[allow(clippy::cast_sign_loss)]
                        let luma = 0.2126f32.mul_add(
                            f32::from(r),
                            0.0722f32.mul_add(f32::from(b), 0.7152 * f32::from(g)),
                        );

                        if luma > 30.0 {
                            byte |= bit;
                            r_sum += u32::from(r);
                            g_sum += u32::from(g);
                            b_sum += u32::from(b);
                            lit_count += 1;
                        }
                    }
                }
            }

            if byte == 0 || lit_count == 0 {
                // Blank cell — reset color and print empty braille (U+2800)
                write!(writer, "\x1b[0m\u{2800}")?;
            } else {
                // Average color of lit dots
                let red = u8::try_from(r_sum / lit_count).unwrap_or(0);
                let green = u8::try_from(g_sum / lit_count).unwrap_or(0);
                let blue = u8::try_from(b_sum / lit_count).unwrap_or(0);
                let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or(' ');
                write!(writer, "\x1b[38;2;{red};{green};{blue}m{ch}")?;
            }
        }
        writeln!(writer, "\x1b[0m")?; // Reset at end of each row
    }

    Ok(())
}

/// Colored fade/density rendering using perceptual luminance + ANSI foreground color
fn write_fade<W: Write>(
    writer: &mut W,
    img: &image::DynamicImage,
    _options: RenderOptions,
) -> std::io::Result<()> {
    let charset = " ░▒▓█";
    render_charset_colored(writer, img, charset)
}

/// Shared colored charset renderer — maps each pixel to a char by luminance,
/// then colorizes it with the pixel's own RGB using ANSI truecolor.
fn render_charset_colored<W: Write>(
    writer: &mut W,
    img: &image::DynamicImage,
    charset: &str,
) -> std::io::Result<()> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let chars: Vec<char> = charset.chars().collect();
    let num_chars = chars.len();

    for y in 0..height {
        for x in 0..width {
            let px = rgba.get_pixel(x, y);
            let [red, green, blue, alpha] = px.0;

            if alpha < 128 {
                write!(writer, "\x1b[0m ")?;
                continue;
            }

            // Perceptual luminance → charset index
            let luma = 0.2126f32.mul_add(
                f32::from(red),
                0.0722f32.mul_add(f32::from(blue), 0.7152 * f32::from(green)),
            );
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            let idx = ((luma / 255.0) * (num_chars - 1) as f32).round() as usize;
            let idx = idx.min(num_chars - 1);
            let ch = chars[idx];

            write!(writer, "\x1b[38;2;{red};{green};{blue}m{ch}")?;
        }
        writeln!(writer, "\x1b[0m")?;
    }

    Ok(())
}

/// Colored ASCII art rendering using perceptual luminance + ANSI foreground color.
///
/// The charset is ordered by increasing visual density/weight as rendered in a
/// typical monospace terminal font. Finer granularity than the fade charset since
/// ASCII chars have more varied shapes that don't form a clean linear ramp.
fn write_ascii<W: Write>(
    writer: &mut W,
    img: &image::DynamicImage,
    _options: RenderOptions,
) -> std::io::Result<()> {
    // Carefully ordered by increasing visual density in monospace fonts.
    // Starts with true empty space, moves through punctuation and symbols,
    // up to the heaviest glyphs (@, #, M, W) at the bright end.
    let charset = " `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";
    render_charset_colored(writer, img, charset)
}
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize,
)]
pub enum RenderStylePreset {
    #[default]
    Ansi,
    Unicode,
    Braille,
    Fade,
    Ascii,
    FullBlock,
    Dense,
}
// 1. Define an Enum for the CLI argument
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")] // For the config file (TOML)
#[clap(rename_all = "kebab-case")] // For the CLI flags
pub enum ResizeFilter {
    /// Nearest Neighbor (Best for pixel art)
    Nearest,
    /// Linear interpolation
    Triangle,
    /// Sharp cubic filter
    CatmullRom,
    /// Blurry cubic filter
    Gaussian,
    /// High-quality resampling (Slowest)
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

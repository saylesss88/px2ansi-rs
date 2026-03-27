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

#[derive(Debug, Clone, Copy)]
pub struct AnsiArtOptions {
    pub mode: OutputMode,
    pub full_block: bool,
}

impl Default for AnsiArtOptions {
    fn default() -> Self {
        Self {
            mode: OutputMode::Ansi,
            full_block: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharsetMode {
    Ansi,    // half-block ▀/▄
    Unicode, // full-block / half-block based on style.full
    Braille,
    Fade,
    Ascii,
}

impl Default for CharsetMode {
    fn default() -> Self {
        CharsetMode::Ansi
    }
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
    pub output_mode: OutputMode, // keep old name for now if lots of call sites use it
    pub target_width: Option<u32>,
    pub filter: FilterType,
    pub full: bool, // deprecated, but keep until you fully migrate
    pub charset: CharsetMode,
    pub style: RenderStyle,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            output_mode: OutputMode::Ansi, // existing enum
            target_width: None,
            filter: FilterType::Lanczos3,
            full: false,
            charset: CharsetMode::Ansi,
            style: RenderStyle::default(),
        }
    }
}

impl From<RenderOptions> for AnsiArtOptions {
    fn from(opts: RenderOptions) -> Self {
        Self {
            mode: opts.output_mode,
            full_block: opts.full,
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

    pub fn from_cli(
        mode: Option<String>,
        full: Option<bool>,
        style: Option<RenderStylePreset>,
        width: Option<u32>,
        filter: Option<ResizeFilter>,
    ) -> anyhow::Result<Self> {
        let mut opts = Self::default();

        if let Some(mode) = mode {
            opts.output_mode = mode.parse()?;
        }

        if let Some(full) = full {
            opts.full = full;
            opts.style.full = full;
        }

        if let Some(style) = style {
            match style {
                RenderStylePreset::Ansi => opts.charset = CharsetMode::Ansi,
                RenderStylePreset::Unicode => opts.charset = CharsetMode::Unicode,
                RenderStylePreset::Braille => opts.charset = CharsetMode::Braille,
                RenderStylePreset::Fade => opts.charset = CharsetMode::Fade,
                RenderStylePreset::Ascii => opts.charset = CharsetMode::Ascii,
                RenderStylePreset::FullAnsi => {
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
    options: RenderOptions, // note: RenderOptions, not AnsiArtOptions
) -> std::io::Result<()> {
    eprintln!("DEBUG: using charset={:?}", options.charset);
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
        // Your existing full-block logic
        for y in 0..height {
            for x in 0..width {
                let px = img.get_pixel(x, y);
                write_full_block(writer, px)?;
            }
            writeln!(writer, "\x1b[0m")?;
        }
    } else {
        // Fallback to ANSI half-blocks (your existing logic)
        write_ansi_blocks(writer, img)?;
    }
    Ok(())
}
// on the writer's capability.
// pub fn write_ansi_art<W: Write>(
//     img: &DynamicImage,
//     writer: &mut W,
//     options: AnsiArtOptions,
// ) -> std::io::Result<()> {
//     let (width, height) = img.dimensions();

//     match options.mode {
//         OutputMode::Ansi => {
//             for y in (0..height).step_by(2) {
//                 for x in 0..width {
//                     let top = img.get_pixel(x, y);
//                     let bot = if y + 1 < height {
//                         img.get_pixel(x, y + 1)
//                     } else {
//                         Rgba([0, 0, 0, 0])
//                     };
//                     write_half_block(writer, top, bot)?;
//                 }
//                 writeln!(writer, "\x1b[0m")?;
//             }
//         }
//         OutputMode::Unicode => {
//             if options.full_block {
//                 for y in 0..height {
//                     for x in 0..width {
//                         let px = img.get_pixel(x, y);
//                         write_full_block(writer, px)?;
//                     }
//                     writeln!(writer, "\x1b[0m")?;
//                 }
//             } else {
//                 for y in (0..height).step_by(2) {
//                     for x in 0..width {
//                         let top = img.get_pixel(x, y);
//                         let bot = if y + 1 < height {
//                             img.get_pixel(x, y + 1)
//                         } else {
//                             Rgba([0, 0, 0, 0])
//                         };
//                         write_half_block(writer, top, bot)?;
//                     }
//                     writeln!(writer, "\x1b[0m")?;
//                 }
//             }
//         }
//     }
//     Ok(())
// }

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
/// Braille rendering (8x4 pixels → 1 char, ultra-dense)
// fn write_braille<W: Write>(
//     writer: &mut W,
//     img: &image::DynamicImage,
//     _options: RenderOptions,
// ) -> std::io::Result<()> {
//     let (width, height) = img.dimensions();
//     // Braille cells are 2px wide by 4px tall
//     for y in (0..height).step_by(4) {
//         for x in (0..width).step_by(2) {
//             let mut byte = 0u8;
//             // Map 2x4 pixels to the Braille bitmask
//             // Dot layout:
//             // 1 4
//             // 2 5
//             // 3 6
//             // 7 8
//             let dots = [
//                 (0, 0, 0x01),
//                 (0, 1, 0x02),
//                 (0, 2, 0x04),
//                 (1, 0, 0x08),
//                 (1, 1, 0x10),
//                 (1, 2, 0x20),
//                 (0, 3, 0x40),
//                 (1, 3, 0x80),
//             ];

//             for (dx, dy, bit) in dots {
//                 if x + dx < width && y + dy < height {
//                     let px = img.get_pixel(x + dx, y + dy);
//                     // If pixel is not transparent and not "black-ish"
//                     if px[3] > 128
//                         && (u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2])) / 3 > 30
//                     {
//                         byte |= bit;
//                     }
//                 }
//             }
//             // Unicode Braille starts at U+2800
//             let c = std::char::from_u32(0x2800 + u32::from(byte)).unwrap_or(' ');
//             write!(writer, "{}", c)?;
//         }
//         writeln!(writer)?;
//     }
//     Ok(())
// }

/// Grayscale fade chars (.,:;i1tfLCG08@)
// fn write_fade<W: Write>(
//     writer: &mut W,
//     img: &image::DynamicImage,
//     _options: RenderOptions,
// ) -> std::io::Result<()> {
//     let charset = " .:-=+*#%@"; // Simple brightness ramp
//     render_charset(writer, img, charset)
// }

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
                        let luma =
                            (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u32;

                        if luma > 30 {
                            byte |= bit;
                            r_sum += r as u32;
                            g_sum += g as u32;
                            b_sum += b as u32;
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
                let r = (r_sum / lit_count) as u8;
                let g = (g_sum / lit_count) as u8;
                let b = (b_sum / lit_count) as u8;
                let c = char::from_u32(0x2800 + byte as u32).unwrap_or(' ');
                write!(writer, "\x1b[38;2;{r};{g};{b}m{c}")?;
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
    // 32-char ramp: much finer brightness gradation than 10 chars
    // Chosen for increasing visual density across common terminal fonts
    let charset = " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";
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
    let n = chars.len();

    for y in 0..height {
        for x in 0..width {
            let px = rgba.get_pixel(x, y);
            let [r, g, b, a] = px.0;

            if a < 128 {
                write!(writer, "\x1b[0m ")?;
                continue;
            }

            // Perceptual luminance → charset index
            let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
            let idx = ((luma / 255.0) * (n - 1) as f32).round() as usize;
            let idx = idx.min(n - 1);
            let c = chars[idx];

            write!(writer, "\x1b[38;2;{r};{g};{b}m{c}")?;
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
// Helper to avoid repeating logic for ASCII/Fade
fn render_charset<W: Write>(
    writer: &mut W,
    img: &image::DynamicImage,
    charset: &str,
) -> std::io::Result<()> {
    let (width, height) = img.dimensions();
    let chars: Vec<char> = charset.chars().collect();

    for y in 0..height {
        for x in 0..width {
            let px = img.get_pixel(x, y);
            if px[3] == 0 {
                write!(writer, " ")?;
                continue;
            }
            // Standard Luminance Formula
            let avg =
                (0.2126 * f64::from(px[0]) + 0.7152 * f64::from(px[1]) + 0.0722 * f64::from(px[2]))
                    / 255.0;
            let idx = (avg * (chars.len() - 1) as f64).round() as usize;
            write!(writer, "{}", chars[idx])?;
        }
        writeln!(writer)?;
    }
    Ok(())
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum RenderStylePreset {
    Ansi,
    Unicode,
    Braille,
    Fade,
    Ascii,
    FullAnsi,
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

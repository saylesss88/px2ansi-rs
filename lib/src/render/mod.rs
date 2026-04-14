//! # Main Entry Point
//!
//! The primary way to use this module is through [`write_ansi_art`], which
//! handles the internal rendering state and dispatches the image data
//! to the appropriate strategy based on the provided [`RenderOptions`].
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
#[cfg(feature = "sixel")]
use viuer;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use std::fmt::Write as FmtWrite;
use std::{borrow::Cow, io::Write};

pub mod options;
pub mod types;
pub mod utils;

pub use options::*;
pub use types::*;
pub use utils::*;

/// The alpha threshold below which a pixel is considered transparent.
const ALPHA_THRESHOLD: u8 = 30;

/// A renderer that writes ANSI art to a `Write` target.
///
/// `Renderer` holds a mutable reference to the output writer and a reference
/// to the prepared image, so individual render methods don't need to pass
/// them around manually.
///
/// # Lifetime
///
/// `'img` is the lifetime of the source image.
/// `'w` is the lifetime of the writer borrow.
struct Renderer<'img, 'w, W: Write> {
    writer: &'w mut W,
    img: &'img DynamicImage,
    options: RenderOptions,
}

impl<'img, 'w, W: Write> Renderer<'img, 'w, W> {
    const fn new(writer: &'w mut W, img: &'img DynamicImage, options: RenderOptions) -> Self {
        Self {
            writer,
            img,
            options,
        }
    }

    /// Renders using ANSI half-block characters (▀/▄).
    /// Two vertical pixels are packed into one character cell.
    fn ansi_blocks(&mut self) -> std::io::Result<()> {
        let (width, height) = self.img.dimensions();
        for y in (0..height).step_by(2) {
            for x in 0..width {
                let top = self.img.get_pixel(x, y);
                let bot = if y + 1 < height {
                    self.img.get_pixel(x, y + 1)
                } else {
                    Rgba([0, 0, 0, 0])
                };
                write_half_block(self.writer, top, bot)?;
            }
            writeln!(self.writer, "\x1b[0m")?;
        }
        Ok(())
    }

    /// Renders using full-block (██) or half-block characters based on `full`.
    fn unicode_blocks(&mut self, full: bool) -> std::io::Result<()> {
        if full {
            let (width, height) = self.img.dimensions();
            for y in 0..height {
                for x in 0..width {
                    write_full_block(self.writer, self.img.get_pixel(x, y))?;
                }
                writeln!(self.writer, "\x1b[0m")?;
            }
        } else {
            self.ansi_blocks()?;
        }
        Ok(())
    }

    /// Renders using Braille dot patterns (U+2800–U+28FF).
    /// Each 2×4 pixel region maps to one Braille character cell.
    fn braille(&mut self) -> std::io::Result<()> {
        // let rgba = self.img.to_rgba8();
        // Borrow the inner RgbaImage when the image is already in that format
        // (e.g. PNG with alpha), avoiding an unnecessary clone/conversion.
        let rgba: Cow<'_, RgbaImage> = self
            .img
            .as_rgba8()
            .map_or_else(|| Cow::Owned(self.img.to_rgba8()), Cow::Borrowed);
        let (width, height) = rgba.dimensions();
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
                        if a > 10 {
                            let luma =
                                (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b))
                                    / 10000;
                            if luma > 2 {
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
                    if self.options.color() {
                        write!(self.writer, "\x1b[0m\u{2800}")?;
                    } else {
                        write!(self.writer, "\u{2800}")?;
                    }
                } else {
                    let red = u8::try_from(r_sum / lit_count).unwrap_or(0);
                    let green = u8::try_from(g_sum / lit_count).unwrap_or(0);
                    let blue = u8::try_from(b_sum / lit_count).unwrap_or(0);

                    let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or(' ');
                    let mut buf = [0u8; 4];
                    let glyph = ch.encode_utf8(&mut buf);
                    if self.options.color() {
                        write_colored_glyph(
                            self.writer,
                            glyph,
                            red,
                            green,
                            blue,
                            self.options.color_mode(),
                        )?;
                    } else {
                        write!(self.writer, "{ch}")?;
                    }
                }
            }

            if self.options.color() {
                writeln!(self.writer, "\x1b[0m")?;
            } else {
                writeln!(self.writer)?;
            }
        }

        Ok(())
    }
    /// Renders using a block-shade ramp (░▒▓█).
    fn fade(&mut self) -> std::io::Result<()> {
        self.charset_colored(&[" ", "░", "▒", "▓", "█"], false)
    }

    /// Renders using a 92-character ASCII density ramp.
    fn ascii(&mut self, density: Density) -> std::io::Result<()> {
        let charset: &[&str] = match density {
            Density::Light => &[
                " ", ".", "`", "\"", "\\", ":", "I", "!", ">", "~", "_", "?", "[", "{", "|", ")",
                "(", "/", "Y", "L", "p", "d", "a", "*", "W", "8", "%", "@", "$",
            ],
            Density::Medium => &[
                " ", ".", "'", "`", "^", "\"", ",", ":", ";", "I", "l", "!", "i", ">", "<", "~",
                "+", "_", "-", "?", "]", "[", "}", "{", "1", ")", "(", "|", "\\", "/", "t", "f",
                "j", "r", "x", "n", "u", "v", "c", "z", "X", "Y", "U", "J", "C", "L", "Q", "0",
                "O", "Z", "m", "w", "q", "p", "d", "b", "k", "h", "a", "o", "*", "#", "M", "W",
                "&", "8", "%", "B", "@", "$",
            ],
            Density::Heavy => &[" ", ".", ":", "o", "O", "0", "#", "M", "W", "@", "$"],
        };
        self.charset_colored(charset, false)
    }

    /// Renders using double-width Japanese(kanji) characters ordered by approximate visual density.
    fn kanji(&mut self) -> std::io::Result<()> {
        self.charset_colored(
            &[
                "\u{3000}", "一", "二", "十", "口", "日", "田", "目", "国", "風", "龍", "龘",
            ],
            true,
        )
    }

    /// Renders using double-width Chinese(hanzi) characters ordered by approximate visual density.
    fn chinese(&mut self) -> std::io::Result<()> {
        self.charset_colored(
            &[
                "\u{3000}", "一", "二", "十", "人", "丁", "口", "日", "目", "田", "国", "木", "金",
                "華", "黑", "龍", "龘",
            ],
            true,
        )
    }
    /// Universal colored charset renderer.
    ///
    /// Maps each pixel to a glyph by normalized perceptual luminance, then
    /// colorizes it with the pixel's own RGB using ANSI truecolor escapes.
    ///
    /// `wide` should be `true` for double-width glyphs (kanji, emoji) — this
    /// steps the x iterator by 2 and uses two spaces for transparent cells so
    /// the grid stays aligned.
    #[allow(clippy::too_many_lines)]
    fn charset_colored(&mut self, charset: &[&str], wide: bool) -> std::io::Result<()> {
        let rgba = self.img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let x_step = if wide { 2 } else { 1 };
        let blank = if wide { "  " } else { " " };
        let num_chars = charset.len();
        let num_chars_u32 = u32::try_from(num_chars).unwrap_or(1);

        // --- Pass 1: Luma Range (SIMD + Optional Rayon) ---
        #[cfg(feature = "parallel")]
        let (luma_min, luma_max) = if wide {
            // Wide mode fallback: Rayon handles rows, scalar handles pixels
            rgba.as_raw()
                .par_chunks_exact(width as usize * 4)
                .map(|row| {
                    let mut lo = u32::MAX;
                    let mut hi = u32::MIN;

                    let mut process_px = |px: &[u8]| {
                        if px[3] >= ALPHA_THRESHOLD {
                            let luma = crate::simd::luma_scalar(px[0], px[1], px[2]);
                            lo = lo.min(luma);
                            hi = hi.max(luma);
                        }
                    };

                    let chunks = row.chunks_exact(8);
                    let remainder = chunks.remainder();

                    for px in chunks {
                        // 8 bytes = 2 pixels. We only want the first pixel (x_step = 2)
                        process_px(&px[0..4]);
                    }

                    // If the width is odd, the remainder contains exactly 4 bytes (1 pixel)
                    if !remainder.is_empty() {
                        process_px(remainder);
                    }

                    (lo, hi)
                })
                .reduce(
                    || (u32::MAX, u32::MIN),
                    |(m1, x1), (m2, x2)| (m1.min(m2), x1.max(x2)),
                )
        } else {
            // Narrow mode: Rayon processes large chunks to maximize SIMD efficiency
            // 32768 bytes = 8192 pixels. Multiple of 32 bytes ensures SIMD never breaks alignment.
            rgba.as_raw()
                .par_chunks(32 * 1024)
                .map(crate::simd::find_luma_range_rgba_bytes)
                .reduce(
                    || (u32::MAX, u32::MIN),
                    |(m1, x1), (m2, x2)| (m1.min(m2), x1.max(x2)),
                )
        };

        #[cfg(not(feature = "parallel"))]
        let (luma_min, luma_max) = if wide {
            let mut lo = u32::MAX;
            let mut hi = u32::MIN;
            for y in 0..height {
                for x in (0..width).step_by(x_step) {
                    let [r, g, b, a] = rgba.get_pixel(x, y).0;
                    if a >= ALPHA_THRESHOLD {
                        let l = crate::simd::luma_scalar(r, g, b);
                        lo = lo.min(l);
                        hi = hi.max(l);
                    }
                }
            }
            (lo, hi)
        } else {
            crate::simd::find_luma_range_rgba_bytes(rgba.as_raw())
        };

        // --- Guard: Fully transparent ---
        if luma_min == u32::MAX {
            for _ in 0..height {
                for _ in (0..width).step_by(x_step) {
                    write!(self.writer, "{blank}")?;
                }
                writeln!(self.writer)?;
            }
            return Ok(());
        }

        let luma_range = (luma_max - luma_min).max(1);
        let color_enabled = self.options.color();
        let color_mode = self.options.color_mode();

        // --- Pass 2: Rendering (Optional Rayon) ---
        #[cfg(feature = "parallel")]
        let rows: Vec<String> = (0..height)
            .into_par_iter()
            .map(|y| {
                let mut row_str = String::with_capacity(width as usize * 12);
                for x in (0..width).step_by(x_step) {
                    let [r, g, b, a] = rgba.get_pixel(x, y).0;
                    if a < ALPHA_THRESHOLD {
                        if color_enabled {
                            row_str.push_str("\x1b[0m");
                        }
                        row_str.push_str(blank);
                        continue;
                    }
                    let luma = crate::simd::luma_scalar(r, g, b);
                    let norm = ((luma - luma_min) * 255) / luma_range;
                    let idx = ((norm * (num_chars_u32 - 1) / 255) as usize).min(num_chars - 1);
                    let glyph = charset[idx];

                    if color_enabled {
                        // Internal helper that writes to a String buffer instead of IO
                        write_colored_glyph_to_str(&mut row_str, glyph, r, g, b, color_mode);
                    } else {
                        row_str.push_str(glyph);
                    }
                }
                if color_enabled {
                    row_str.push_str("\x1b[0m\n");
                } else {
                    row_str.push('\n');
                }
                row_str
            })
            .collect();

        #[cfg(feature = "parallel")]
        for row in rows {
            self.writer.write_all(row.as_bytes())?;
        }

        #[cfg(not(feature = "parallel"))]
        {
            for y in 0..height {
                for x in (0..width).step_by(x_step) {
                    let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
                    if alpha < ALPHA_THRESHOLD {
                        if self.options.color() {
                            write!(self.writer, "\x1b[0m{blank}")?;
                        } else {
                            write!(self.writer, "{blank}")?;
                        }
                        continue;
                    }
                    let luma = crate::simd::luma_scalar(red, green, blue);
                    let normalized = ((luma - luma_min) * 255) / luma_range;
                    let idx =
                        ((normalized * (num_chars_u32 - 1) / 255) as usize).min(num_chars - 1);
                    let glyph = charset[idx];
                    if self.options.color() {
                        write_colored_glyph(
                            self.writer,
                            glyph,
                            red,
                            green,
                            blue,
                            self.options.color_mode(),
                        )?;
                    } else {
                        write!(self.writer, "{glyph}")?;
                    }
                }
                if self.options.color() {
                    writeln!(self.writer, "\x1b[0m")?;
                } else {
                    writeln!(self.writer)?;
                }
            }
        }

        Ok(())
    }
}

/// Renders a prepared image to `writer` using the mode specified in `options`.
///
/// This is the primary entry point for the rendering engine. It handles the
/// internal state management and dispatches the image data to the
/// appropriate rendering strategy based on the chosen [`CharsetMode`].
///
/// # Errors
///
/// Returns a [`std::io::Result`] error if the writer fails./ Returns a [`std::io::Result`] error if the writer fails.
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    writer: &mut W,
    options: RenderOptions,
) -> std::io::Result<()> {
    let mut renderer = Renderer::new(writer, img, options);
    match options.charset() {
        CharsetMode::Ansi => renderer.ansi_blocks(),
        CharsetMode::Unicode => renderer.unicode_blocks(options.style().full),
        CharsetMode::Braille => renderer.braille(),
        CharsetMode::Fade => renderer.fade(),
        CharsetMode::Ascii => renderer.ascii(options.style().density),
        CharsetMode::Kanji => renderer.kanji(),
        CharsetMode::Chinese => renderer.chinese(),
        #[cfg(feature = "sixel")]
        CharsetMode::Sixel => write_sixel(img),
        #[cfg(not(feature = "sixel"))]
        CharsetMode::Sixel => {
            eprintln!("Sixel support requires the 'sixel' feature.");
            eprintln!("Rebuild with: cargo build --features sixel");
            Ok(())
        }
    }
}
/// Writes a single half-block character cell representing two vertical pixels.
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

/// Writes a single pixel as a double-width full block (██) for 1:1 aspect ratio.
fn write_full_block<W: Write>(out: &mut W, px: Rgba<u8>) -> std::io::Result<()> {
    if px[3] > 0 {
        write!(out, "\x1b[38;2;{};{};{}m██", px[0], px[1], px[2])
    } else {
        write!(out, "  ")
    }
}

/// Renders an image using the Sixel graphics protocol.
///
/// Sixel encodes pixel data directly into the terminal escape sequence stream,
/// allowing true pixel-accurate images in supported terminals.
///
/// # Errors
///
/// This function will return an error if `viuer` fails to write to the terminal
/// buffer or if the image cannot be encoded into the Sixel format.
#[cfg(feature = "sixel")]
pub fn write_sixel(img: &image::DynamicImage) -> std::io::Result<()> {
    viuer::print(img, &viuer::Config::default())
        .map(|_| ())
        .map_err(|e| std::io::Error::other(e.to_string()))
}

fn write_colored_glyph<W: Write>(
    writer: &mut W,
    glyph: &str,
    r: u8,
    g: u8,
    b: u8,
    color_mode: ColorMode,
) -> std::io::Result<()> {
    match color_mode {
        ColorMode::TrueColor => {
            write!(writer, "\x1b[38;2;{r};{g};{b}m{glyph}")
        }
        ColorMode::Ansi256 => {
            let idx = crate::color::rgb_to_xterm256(r, g, b);
            write!(writer, "\x1b[38;5;{idx}m{glyph}")
        }
        ColorMode::None => {
            write!(writer, "{glyph}")
        }
    }
}
#[cfg(feature = "parallel")]
fn write_colored_glyph_to_str(
    buf: &mut String,
    glyph: &str,
    r: u8,
    g: u8,
    b: u8,
    color_mode: ColorMode,
) {
    match color_mode {
        ColorMode::TrueColor => {
            let _ = write!(buf, "\x1b[38;2;{r};{g};{b}m{glyph}");
        }
        ColorMode::Ansi256 => {
            let idx = crate::color::rgb_to_xterm256(r, g, b);
            let _ = write!(buf, "\x1b[38;5;{idx}m{glyph}");
        }
        ColorMode::None => {
            buf.push_str(glyph);
        }
    }
}

use image::{DynamicImage, GenericImageView, Rgba};
use std::io::Write;

use crate::options::{CharsetMode, RenderOptions};

/// The alpha threshold below which a pixel is considered transparent.
const ALPHA_THRESHOLD: u8 = 128;

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
}

impl<'img, 'w, W: Write> Renderer<'img, 'w, W> {
    const fn new(writer: &'w mut W, img: &'img DynamicImage) -> Self {
        Self { writer, img }
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
        let rgba = self.img.to_rgba8();
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
                    write!(self.writer, "\x1b[0m\u{2800}")?;
                } else {
                    let red = u8::try_from(r_sum / lit_count).unwrap_or(0);
                    let green = u8::try_from(g_sum / lit_count).unwrap_or(0);
                    let blue = u8::try_from(b_sum / lit_count).unwrap_or(0);
                    let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or(' ');
                    write!(self.writer, "\x1b[38;2;{red};{green};{blue}m{ch}")?;
                }
            }
            writeln!(self.writer, "\x1b[0m")?;
        }
        Ok(())
    }

    /// Renders using a block-shade ramp (░▒▓█).
    fn fade(&mut self) -> std::io::Result<()> {
        self.charset_colored(&[" ", "░", "▒", "▓", "█"], false)
    }

    /// Renders using a 92-character ASCII density ramp.
    fn ascii(&mut self) -> std::io::Result<()> {
        self.charset_colored(
            &[
                " ", "`", ".", "-", "'", ":", "_", ",", "^", "=", ";", ">", "<", "+", "!", "r",
                "c", "*", "/", "z", "?", "s", "L", "T", "v", ")", "J", "7", "(", "|", "F", "i",
                "{", "C", "}", "f", "I", "3", "1", "t", "l", "u", "[", "n", "e", "o", "Z", "5",
                "Y", "x", "j", "y", "a", "]", "2", "E", "S", "w", "q", "k", "P", "6", "h", "9",
                "d", "4", "V", "p", "O", "G", "b", "U", "A", "K", "X", "H", "m", "8", "R", "D",
                "#", "$", "B", "g", "0", "M", "N", "W", "Q", "%", "&", "@",
            ],
            false,
        )
    }

    /// Renders using double-width Kanji characters ordered by visual density.
    fn kanji(&mut self) -> std::io::Result<()> {
        self.charset_colored(
            &["　", "一", "口", "田", "目", "龍", "量", "首", "艦"],
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
    fn charset_colored(&mut self, charset: &[&str], wide: bool) -> std::io::Result<()> {
        let rgba = self.img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let num_chars_u32 = u32::try_from(charset.len()).unwrap_or(1);
        let num_chars = charset.len();
        let x_step = if wide { 2 } else { 1 };
        let blank = if wide { "  " } else { " " };

        // First pass: find the actual luma range of opaque pixels so we can
        // normalize dark images (like the NixOS logo) across the full charset.
        let mut luma_min = u32::MAX;
        let mut luma_max = u32::MIN;
        for y in 0..height {
            for x in (0..width).step_by(x_step) {
                let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
                if alpha >= ALPHA_THRESHOLD {
                    let luma =
                        (2126 * u32::from(red) + 7152 * u32::from(green) + 722 * u32::from(blue))
                            / 10000;
                    luma_min = luma_min.min(luma);
                    luma_max = luma_max.max(luma);
                }
            }
        }
        let luma_range = (luma_max - luma_min).max(1);

        // Second pass: render each pixel as a colored glyph.
        for y in 0..height {
            for x in (0..width).step_by(x_step) {
                let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
                if alpha < ALPHA_THRESHOLD {
                    write!(self.writer, "\x1b[0m{blank}")?;
                    continue;
                }
                let luma =
                    (2126 * u32::from(red) + 7152 * u32::from(green) + 722 * u32::from(blue))
                        / 10000;
                let normalized = ((luma - luma_min) * 255) / luma_range;
                let idx = ((normalized * (num_chars_u32 - 1) / 255) as usize).min(num_chars - 1);
                let glyph = charset[idx];
                write!(self.writer, "\x1b[38;2;{red};{green};{blue}m{glyph}")?;
            }
            writeln!(self.writer, "\x1b[0m")?;
        }
        Ok(())
    }
}

/// Renders a prepared image to `writer` using the mode specified in `options`.
///
/// This is the public entry point — it constructs a [`Renderer`] and dispatches
/// to the appropriate method based on [`CharsetMode`].
///
/// # Errors
///
/// Returns a [`std::io::Result`] error if the writer fails.
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    writer: &mut W,
    options: RenderOptions,
) -> std::io::Result<()> {
    let mut renderer = Renderer::new(writer, img);
    match options.charset {
        CharsetMode::Ansi => renderer.ansi_blocks(),
        CharsetMode::Unicode => renderer.unicode_blocks(options.style.full),
        CharsetMode::Braille => renderer.braille(),
        CharsetMode::Fade => renderer.fade(),
        CharsetMode::Ascii => renderer.ascii(),
        CharsetMode::Kanji => renderer.kanji(),
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

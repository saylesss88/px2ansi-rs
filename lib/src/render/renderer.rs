use std::borrow::Cow;
use std::io::{self, Write};

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use super::color::{write_colored_glyph, write_full_block, write_half_block, ColorState};
use super::options::RenderOptions;
use super::pixel::{luma_range_pass1, ColorParams, LumaParams, RenderCtx};
use super::serial::render_serial;
use super::types::{CharsetMode, Density};
use crate::ColorMode;

#[cfg(feature = "parallel")]
use super::parallel::render_parallel;

/// Internal engine that coordinates the conversion of image pixels into
/// terminal-friendly character output.
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

    /// Renders using the "Half-Block" method (▄), which allows for two vertical
    /// "pixels" per terminal cell by setting different foreground and background colors.
    fn ansi_blocks(&mut self) -> io::Result<()> {
        let (width, height) = self.img.dimensions();
        // We step by 2 because one terminal row covers two rows of image pixels.
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
            writeln!(self.writer, "\x1b[0m")?; // Reset color at EOL
        }
        Ok(())
    }

    /// Renders using full block characters (█). If 'full' is false, it
    /// defaults back to the more efficient half-block method.
    fn unicode_blocks(&mut self, full: bool) -> io::Result<()> {
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

    /// Renders using Unicode Braille patterns (U+2800 - U+28FF).
    /// This provides a 2x4 "sub-pixel" resolution within a single character cell.
    fn braille(&mut self) -> io::Result<()> {
        let rgba: Cow<'_, RgbaImage> = self
            .img
            .as_rgba8()
            .map_or_else(|| Cow::Owned(self.img.to_rgba8()), Cow::Borrowed);
        let (width, height) = rgba.dimensions();

        // Braille dot-to-bit mapping (standard 2x4 grid)
        // .1 .4
        // .2 .5
        // .3 .6
        // .7 .8
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

        let mut last_color = ColorState::default();
        let mode = self.options.color_mode(); // Cache the mode

        for y in (0..height).step_by(4) {
            for x in (0..width).step_by(2) {
                let mut byte = 0u8;
                let (mut r_sum, mut g_sum, mut b_sum, mut lit_count) = (0u32, 0u32, 0u32, 0u32);

                for (dx, dy, bit) in dots {
                    let px_x = x + dx;
                    let px_y = y + dy;
                    if px_x < width && px_y < height {
                        let px = rgba.get_pixel(px_x, px_y);
                        let [r, g, b, a] = px.0;
                        // Ignore transparent pixels; calculate luma for the rest
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

                // --- RENDERING LOGIC ---
                //
                // Render the accumulated 2x4 block
                if byte == 0 || lit_count == 0 {
                    // No dots are lit: Reset color if needed and print empty braille
                    if mode != ColorMode::None && last_color != ColorState::default() {
                        write!(self.writer, "\x1b[0m")?;
                        last_color = ColorState::default();
                    }
                    write!(self.writer, "\u{2800}")?; // Empty braille cell
                } else {
                    let r = u8::try_from(r_sum / lit_count).unwrap_or(0);
                    let g = u8::try_from(g_sum / lit_count).unwrap_or(0);
                    let b = u8::try_from(b_sum / lit_count).unwrap_or(0);

                    let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or(' ');

                    if mode == ColorMode::None {
                        write!(self.writer, "{ch}")?;
                    } else {
                        let mut buf = [0u8; 4];
                        let glyph = ch.encode_utf8(&mut buf);
                        write_colored_glyph(self.writer, glyph, r, g, b, mode, &mut last_color)?;
                    }
                }
            }

            if mode != ColorMode::None && last_color != ColorState::default() {
                write!(self.writer, "\x1b[0m")?;
                last_color = ColorState::default();
            }

            // Everyone gets a newline
            writeln!(self.writer)?;
        }

        Ok(())
    }

    /// Renders using a gradient of block characters (░▒▓█) based on luminance.
    fn fade(&mut self) -> io::Result<()> {
        self.charset_colored(&[" ", "░", "▒", "▓", "█"], false)
    }

    /// Renders using standard ASCII characters mapped by visual density.
    fn ascii(&mut self, density: Density) -> io::Result<()> {
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

    /// High-resolution rendering using Kanji characters of varying complexity.
    fn kanji(&mut self) -> io::Result<()> {
        self.charset_colored(
            &[
                "\u{3000}", "一", "二", "十", "口", "日", "田", "目", "国", "風", "龍", "龘",
            ],
            true,
        )
    }

    /// High-resolution rendering using Chinese characters of varying complexity.
    fn chinese(&mut self) -> io::Result<()> {
        self.charset_colored(
            &[
                "\u{3000}", "一", "二", "十", "人", "丁", "口", "日", "目", "田", "国", "木", "金",
                "華", "黑", "龍", "龘",
            ],
            true,
        )
    }

    /// Generic rendering path for custom character sets.
    /// Handles luminance normalization to ensure the full range of the charset is used.
    fn charset_colored(&mut self, charset: &[&str], wide: bool) -> io::Result<()> {
        let rgba = self.img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let x_step: usize = if wide { 2 } else { 1 };
        let blank: &str = if wide { "  " } else { " " };
        let num_chars_minus_1 = u32::try_from(charset.len()).unwrap_or(1) - 1;

        // Use parallel rendering if the image is large enough to justify the overhead
        let use_parallel = cfg!(feature = "parallel") && (width * height > 120_000);

        let (luma_min, luma_max) =
            luma_range_pass1(&rgba, width, height, x_step, wide, use_parallel);

        // If the image is completely transparent/black
        if luma_min == u32::MAX {
            for _ in 0..height {
                for _ in (0..width).step_by(x_step) {
                    write!(self.writer, "{blank}")?;
                }
                writeln!(self.writer)?;
            }
            return Ok(());
        }

        let lp = LumaParams {
            min: luma_min,
            range: (luma_max - luma_min).max(1),
            num_chars_minus_1,
        };
        let cp = ColorParams {
            enabled: self.options.color_mode() != ColorMode::None,
            mode: self.options.color_mode(),
            blank,
        };

        let ctx = RenderCtx::new(&rgba, charset, wide);

        if use_parallel {
            #[cfg(feature = "parallel")]
            render_parallel(self.writer, &ctx, lp, cp)?;
        } else {
            render_serial(self.writer, &ctx, lp, cp)?;
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
/// Returns a [`std::io::Result`] error if the writer fails.
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    writer: &mut W,
    options: RenderOptions,
) -> io::Result<()> {
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
        CharsetMode::Sixel => write_sixel(img, &options),
        #[cfg(not(feature = "sixel"))]
        CharsetMode::Sixel => {
            eprintln!("Sixel support requires the 'sixel' feature.");
            eprintln!("Rebuild with: cargo build --features sixel");
            Ok(())
        }
    }
}

/// Renders an image using the Sixel graphics protocol.
///
/// Sixel encodes pixel data directly into the terminal escape sequence stream,
/// allowing true pixel-accurate images in supported terminals.///
/// # Errors
///
/// This function will return an error if `viuer` fails to write to the terminal
/// buffer or if the image cannot be encoded into the Sixel format.
#[cfg(feature = "sixel")]
#[cfg_attr(docsrs, doc(cfg(feature = "sixel")))]
pub fn write_sixel(img: &image::DynamicImage, options: &RenderOptions) -> io::Result<()> {
    let base_cfg = viuer::Config {
        use_kitty: false,
        use_iterm: false,
        absolute_offset: false,
        x: 0,
        y: 0,
        width: options.width(),
        height: None,
        restore_cursor: false,
        truecolor: true,
        ..viuer::Config::default()
    };

    match options.bg_color() {
        Some(bg) => {
            let base = image::Rgba([bg[0], bg[1], bg[2], 255u8]);
            let mut composited = image::RgbaImage::from_pixel(img.width(), img.height(), base);
            image::imageops::overlay(&mut composited, &img.to_rgba8(), 0, 0);
            let composited = image::DynamicImage::ImageRgba8(composited);
            viuer::print(
                &composited,
                &viuer::Config {
                    transparent: false,
                    ..base_cfg
                },
            )
        }
        None => {
            // Terminal didn't respond to OSC 11: let it handle transparency natively
            viuer::print(
                img,
                &viuer::Config {
                    transparent: true,
                    ..base_cfg
                },
            )
        }
    }
    .map(|_| ())
    .map_err(|e| io::Error::other(e.to_string()))
}

use std::borrow::Cow;
use std::io::Write;

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use super::color::{ColorState, write_colored_glyph, write_full_block, write_half_block};
use super::options::RenderOptions;
use super::pixel::{ColorParams, LumaParams, RenderCtx, luma_range_pass1};
use super::serial::render_serial;
use super::types::{CharsetMode, Density};

#[cfg(feature = "parallel")]
use super::parallel::render_parallel;

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

    fn braille(&mut self) -> std::io::Result<()> {
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
        let mut last_color = ColorState::default();
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
                        last_color = ColorState::default();
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
                            &mut last_color,
                        )?;
                    } else {
                        write!(self.writer, "{ch}")?;
                    }
                }
            }

            if self.options.color() {
                writeln!(self.writer, "\x1b[0m")?;
                last_color = ColorState::default();
            } else {
                writeln!(self.writer)?;
            }
        }

        Ok(())
    }

    fn fade(&mut self) -> std::io::Result<()> {
        self.charset_colored(&[" ", "░", "▒", "▓", "█"], false)
    }

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

    fn kanji(&mut self) -> std::io::Result<()> {
        self.charset_colored(
            &[
                "\u{3000}", "一", "二", "十", "口", "日", "田", "目", "国", "風", "龍", "龘",
            ],
            true,
        )
    }

    fn chinese(&mut self) -> std::io::Result<()> {
        self.charset_colored(
            &[
                "\u{3000}", "一", "二", "十", "人", "丁", "口", "日", "目", "田", "国", "木", "金",
                "華", "黑", "龍", "龘",
            ],
            true,
        )
    }

    fn charset_colored(&mut self, charset: &[&str], wide: bool) -> std::io::Result<()> {
        let rgba = self.img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let x_step: usize = if wide { 2 } else { 1 };
        let blank: &str = if wide { "  " } else { " " };
        let num_chars_minus_1 = u32::try_from(charset.len()).unwrap_or(1) - 1;
        let use_parallel = cfg!(feature = "parallel") && (width * height > 120_000);

        let (luma_min, luma_max) =
            luma_range_pass1(&rgba, width, height, x_step, wide, use_parallel);

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
            enabled: self.options.color(),
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
/// allowing true pixel-accurate images in supported terminals.
///
/// # Errors
///
/// This function will return an error if `viuer` fails to write to the terminal
/// buffer or if the image cannot be encoded into the Sixel format.
#[cfg(feature = "sixel")]
pub fn write_sixel(img: &image::DynamicImage, options: &RenderOptions) -> std::io::Result<()> {
    use super::utils::get_terminal_size;

    let (term_w, term_h) = get_terminal_size();
    let width = options.width().unwrap_or(term_w);

    let cfg = viuer::Config {
        use_kitty: false,
        use_iterm: false,
        absolute_offset: false,
        x: 0,
        y: 0,
        width: Some(width),
        height: Some(term_h),
        restore_cursor: false,
        truecolor: true,
        ..viuer::Config::default()
    };

    let rgb = image::DynamicImage::ImageRgb8(img.to_rgb8());
    viuer::print(&rgb, &cfg)
        .map(|_| ())
        .map_err(|e| std::io::Error::other(e.to_string()))
}

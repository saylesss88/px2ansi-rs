#![allow(clippy::multiple_crate_versions)]
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Rgba};
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

/// Configuration for how an image should be processed and rendered to the terminal.
///
/// This handles the "look and feel" of the output, including the character set
/// (ANSI vs Unicode), scaling filters, and whether to use half-block positioning.#
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    // Determines the character set used for rendering (e.g., ASCII/ANSI or Unicode)
    pub output_mode: OutputMode,
    /// An optional fixed width. If `None`, the renderer will calculate the best fit
    /// based on the current terminal size.
    pub target_width: Option<u32>,

    /// The algorithm used for resizing. `Nearest` is best for pixel art,
    /// while `Lanczos3` provides the best results for high-res photos.
    pub filter: FilterType,
    /// If true, uses the full color/pixel density available for the chosen mode.
    pub full: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            output_mode: OutputMode::Ansi,
            target_width: None,
            filter: FilterType::Lanczos3, // Reasonable default
            full: false,
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
    pub fn new() -> Self {
        Self::default()
    }
    /// Resizes a DynamicImage based on the current options and terminal constraints.
    pub fn prepare_image(&self, img: DynamicImage) -> DynamicImage {
        let (width, height) = self.calculate_dimensions(img.width(), img.height());
        img.resize_exact(width, height, self.filter)
    }

    /// The "math" part of your old process_and_render
    pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
        const MAX_SAFE: u32 = 16384;

        // 1. Determine the "canvas" size (Terminal or Default)
        let term_dims =
            terminal_size().map(|(Width(tw), Height(th))| (u32::from(tw), u32::from(th)));

        let (max_w, max_h) = if let Some((tw, th)) = term_dims {
            // Adjust constraints based on rendering mode
            let mw = if self.output_mode == OutputMode::Unicode && self.full {
                tw / 2
            } else {
                tw
            }
            .saturating_sub(2);

            let mh = if self.output_mode == OutputMode::Unicode && self.full {
                th
            } else {
                th * 2
            }
            .saturating_sub(2);

            (mw, mh)
        } else {
            (80, 40) // Fallback for piped output or non-tty
        };

        // 2. Run the scaling logic
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

    /// High-level method to render directly to a writer
    pub fn render<W: Write>(&self, img: DynamicImage, writer: &mut W) -> anyhow::Result<()> {
        let prepared = self.prepare_image(img);
        let ansi_opts = AnsiArtOptions::from(*self);
        write_ansi_art(&prepared, writer, ansi_opts)?;
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
/// on the writer's capability.
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    writer: &mut W,
    options: AnsiArtOptions,
) -> std::io::Result<()> {
    let (width, height) = img.dimensions();

    match options.mode {
        OutputMode::Ansi => {
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
        }
        OutputMode::Unicode => {
            if options.full_block {
                for y in 0..height {
                    for x in 0..width {
                        let px = img.get_pixel(x, y);
                        write_full_block(writer, px)?;
                    }
                    writeln!(writer, "\x1b[0m")?;
                }
            } else {
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
            }
        }
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

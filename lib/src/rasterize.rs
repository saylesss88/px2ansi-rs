#![cfg(feature = "rasterize")]
use crate::themes::RasterTheme;

use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};

const FONT_SIZE: f32 = 14.0;
const CELL_W: u32 = 8;
/// Must match `FONT_SIZE` to avoid per-row gaps.
const CELL_H: u32 = 14;

const DEFAULT_FONT: &[u8] = include_bytes!("../assets/IosevkaCharonMono-Regular.ttf");

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Processes a raw byte slice of ANSI escape sequences and renders it into an
/// RGBA image buffer using the default Tokyo Night background theme.
///
/// # Errors
///
/// Returns an error if:
/// * The embedded font fails to initialize.
/// * The input produces an empty grid (no renderable content).
pub fn rasterize_ansi(ansi: &[u8]) -> anyhow::Result<RgbaImage> {
    rasterize_ansi_with_theme(ansi, RasterTheme::default())
}

/// Processes ANSI escape sequences into an image with a custom background theme.
///
/// Correctly handles the half-block (`▀`/`▄`) encoding produced by the default
/// `ansi_blocks` renderer: both the foreground **and** background colors of each
/// terminal cell are decoded and used to fill the top/bottom halves of each
/// pixel cell directly, without going through the font rasterizer.
///
/// # Examples
///
/// ```no_run
/// # use px2ansi::{rasterize_ansi_with_theme, RasterTheme};
/// # let ansi_bytes = b"Hello";
/// let img = rasterize_ansi_with_theme(ansi_bytes, RasterTheme::Dracula)?;
/// img.save("output.png")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * The embedded font fails to initialize.
/// * The input produces an empty grid (no renderable content).
pub fn rasterize_ansi_with_theme(ansi: &[u8], theme: RasterTheme) -> anyhow::Result<RgbaImage> {
    let font = Font::from_bytes(DEFAULT_FONT, FontSettings::default())
        .map_err(|e| anyhow::anyhow!("Font error: {e}"))?;

    let bg_color = theme.color();
    let cells = parse_ansi(ansi, bg_color);
    anyhow::ensure!(!cells.is_empty(), "No cells to render");

    let cols = cells.iter().map(std::vec::Vec::len).max().unwrap_or(0);
    let rows = cells.len();

    let img_w = u32::try_from(cols)
        .unwrap_or(u32::MAX)
        .saturating_mul(CELL_W);
    let img_h = u32::try_from(rows)
        .unwrap_or(u32::MAX)
        .saturating_mul(CELL_H);

    let mut img = RgbaImage::new(img_w, img_h);
    // Pre-fill entire image with the theme background.
    for pixel in img.pixels_mut() {
        *pixel = bg_color;
    }

    for (row_idx, row) in cells.iter().enumerate() {
        let Ok(row_u32) = u32::try_from(row_idx) else {
            continue;
        };
        let base_y = row_u32.saturating_mul(CELL_H);

        for (col_idx, cell) in row.iter().enumerate() {
            let Ok(col_u32) = u32::try_from(col_idx) else {
                continue;
            };
            let base_x = col_u32.saturating_mul(CELL_W);

            match cell {
                // Transparent / space — already filled with bg, nothing to do.
                Cell::Transparent => {}

                // Half-block ▀: top half = fg color, bottom half = bg color.
                Cell::HalfBlock { top, bot } => {
                    fill_rect(&mut img, base_x, base_y, CELL_W, CELL_H / 2, *top);
                    fill_rect(
                        &mut img,
                        base_x,
                        base_y + CELL_H / 2,
                        CELL_W,
                        CELL_H - CELL_H / 2,
                        *bot,
                    );
                }

                // Half-block ▄: bottom half only (top stays bg).
                Cell::HalfBlockBot { color } => {
                    fill_rect(
                        &mut img,
                        base_x,
                        base_y + CELL_H / 2,
                        CELL_W,
                        CELL_H - CELL_H / 2,
                        *color,
                    );
                }

                // Ordinary text glyph — rasterize through fontdue.
                Cell::Glyph(ch, [r, g, b]) => {
                    let (metrics, bitmap) = font.rasterize(*ch, FONT_SIZE);
                    if metrics.width == 0 {
                        continue; // glyph not in font, skip silently
                    }
                    let glyph_h = u32::try_from(metrics.height).unwrap_or(0);
                    let y_offset = CELL_H.saturating_sub(glyph_h) / 2;

                    for gy in 0..metrics.height {
                        for gx in 0..metrics.width {
                            let coverage = bitmap[gy * metrics.width + gx];
                            if coverage == 0 {
                                continue;
                            }
                            let Ok(gx_u32) = u32::try_from(gx) else {
                                continue;
                            };
                            let Ok(glyph_u32) = u32::try_from(gy) else {
                                continue;
                            };
                            let px_x = base_x.saturating_add(gx_u32);
                            let px_y = base_y.saturating_add(y_offset).saturating_add(glyph_u32);
                            if px_x < img_w && px_y < img_h {
                                img.put_pixel(
                                    px_x,
                                    px_y,
                                    blend_pixel([*r, *g, *b], coverage, bg_color),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(img)
}

// ---------------------------------------------------------------------------
// Cell representation
// ---------------------------------------------------------------------------

/// One parsed terminal cell.
#[derive(Debug, Clone)]
enum Cell {
    /// Transparent / space — keep theme background.
    Transparent,
    /// `▀` — top half fg color, bottom half bg color.
    HalfBlock { top: Rgba<u8>, bot: Rgba<u8> },
    /// `▄` — bottom half fg color only (top stays bg).
    HalfBlockBot { color: Rgba<u8> },
    /// Any other printable character with its foreground color.
    Glyph(char, [u8; 3]),
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parses ANSI escape sequences into a grid of [`Cell`]s.
///
/// Handles:
/// * SGR truecolor foreground (`ESC[38;2;R;G;Bm`)
/// * SGR truecolor background (`ESC[48;2;R;G;Bm`)
/// * SGR reset (`ESC[0m` or `ESC[m`) — resets fg to white, bg to theme color
/// * Half-block characters `▀` / `▄` (classified as [`Cell::HalfBlock`] /
///   [`Cell::HalfBlockBot`] using the current fg/bg colors)
/// * All other printable chars as [`Cell::Glyph`]
fn parse_ansi(input: &[u8], theme_bg: Rgba<u8>) -> Vec<Vec<Cell>> {
    let mut rows: Vec<Vec<Cell>> = Vec::new();
    let mut current_row: Vec<Cell> = Vec::new();
    let mut fg: [u8; 3] = [255, 255, 255];
    let mut bg: Rgba<u8> = theme_bg;
    let mut i = 0;

    while i < input.len() {
        if input[i] == 0x1b && input.get(i + 1) == Some(&b'[') {
            // ESC [ — start of a CSI sequence
            i += 2;
            let mut seq = Vec::new();
            while i < input.len() && !input[i].is_ascii_alphabetic() {
                seq.push(input[i]);
                i += 1;
            }
            let final_byte = input.get(i).copied().unwrap_or(0);
            i += 1;
            if final_byte == b'm' {
                let params = std::str::from_utf8(&seq).unwrap_or("");
                parse_color_params(params, &mut fg, &mut bg, theme_bg);
            }
            // Non-'m' CSI sequences (cursor movement, etc.) are ignored.
        } else if input[i] == b'\n' {
            rows.push(std::mem::take(&mut current_row));
            i += 1;
        } else {
            // Decode the next Unicode scalar.
            let ch = if input[i] < 128 {
                let c = input[i] as char;
                i += 1;
                c
            } else {
                let s = std::str::from_utf8(&input[i..]).unwrap_or(" ");
                let c = s.chars().next().unwrap_or(' ');
                i += c.len_utf8();
                c
            };

            let cell = match ch {
                ' ' | '\0' => Cell::Transparent,
                // ▀  U+2580  UPPER HALF BLOCK
                '\u{2580}' => Cell::HalfBlock {
                    top: Rgba([fg[0], fg[1], fg[2], 255]),
                    bot: bg,
                },
                // ▄  U+2584  LOWER HALF BLOCK
                '\u{2584}' => Cell::HalfBlockBot {
                    color: Rgba([fg[0], fg[1], fg[2], 255]),
                },
                _ => Cell::Glyph(ch, fg),
            };
            current_row.push(cell);
        }
    }

    if !current_row.is_empty() {
        rows.push(current_row);
    }

    rows
}

/// Parses an SGR parameter string and updates the current fg/bg colors.
///
/// * `38;2;R;G;B` → set foreground truecolor
/// * `48;2;R;G;B` → set background truecolor
/// * `0` or empty  → reset fg to white, bg to theme background
fn parse_color_params(params: &str, fg: &mut [u8; 3], bg: &mut Rgba<u8>, theme_bg: Rgba<u8>) {
    if params == "0" || params.is_empty() {
        *fg = [255, 255, 255];
        *bg = theme_bg;
        return;
    }

    // A single SGR sequence can contain multiple sub-commands separated by ';'.
    // We scan through them so compound sequences like "0;38;2;R;G;B" work too.
    let parts: Vec<u8> = params.split(';').filter_map(|s| s.parse().ok()).collect();
    let mut idx = 0;
    while idx < parts.len() {
        match parts[idx] {
            0 => {
                *fg = [255, 255, 255];
                *bg = theme_bg;
                idx += 1;
            }
            38 if parts.get(idx + 1) == Some(&2) && parts.len() > idx + 4 => {
                *fg = [parts[idx + 2], parts[idx + 3], parts[idx + 4]];
                idx += 5;
            }
            48 if parts.get(idx + 1) == Some(&2) && parts.len() > idx + 4 => {
                let [r, g, b] = [parts[idx + 2], parts[idx + 3], parts[idx + 4]];
                *bg = Rgba([r, g, b, 255]);
                idx += 5;
            }
            _ => {
                idx += 1;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pixel helpers
// ---------------------------------------------------------------------------

/// Fills a rectangular region of the image with a solid color, clamped to bounds.
fn fill_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    let (img_w, img_h) = img.dimensions(); // Automatically get dimensions

    for py in y..y.saturating_add(h).min(img_h) {
        for px in x..x.saturating_add(w).min(img_w) {
            img.put_pixel(px, py, color);
        }
    }
}

/// Alpha-blends a foreground color against a background color using glyph coverage.
fn blend_pixel([r, g, b]: [u8; 3], coverage: u8, bg: Rgba<u8>) -> Rgba<u8> {
    let alpha = f32::from(coverage) / 255.0;

    let blend = |fg: u8, bg_val: u8| -> u8 {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Linear interpolation of u8 color components via f32 is safe"
        )]
        {
            // Linear interpolation: bg * (1 - alpha) + fg * alpha
            f32::from(bg_val).mul_add(1.0 - alpha, f32::from(fg) * alpha) as u8
        }
    };

    let [br, bg_c, bb, _] = bg.0;
    Rgba([blend(r, br), blend(g, bg_c), blend(b, bb), 255])
}

use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};

const FONT_SIZE: f32 = 14.0;
const CELL_W: u32 = 8;
const CELL_H: u32 = 16;

/// Tokyo Night background color (#1A1B26)
const BG: Rgba<u8> = Rgba([26, 27, 38, 255]);

/// Processes a raw byte slice of ANSI escape sequences and renders it into an
/// RGBA image buffer.
///
/// This function uses a single embedded monospace font (Iosevka Charon) to
/// rasterize terminal art. It calculates the final image dimensions based on
/// the parsed grid of characters and applies a background-to-foreground
/// pixel blending pass.
///
/// # Errors
///
/// Returns an error if:
/// * The embedded font fails to initialize.
/// * The input produces an empty grid (no renderable content).
///
/// # Limitations
///
/// Character glyphs not present in the primary embedded font are skipped
/// silently during the rasterization process.
pub fn rasterize_ansi(ansi: &[u8]) -> anyhow::Result<RgbaImage> {
    let font = Font::from_bytes(
        include_bytes!("../assets/IosevkaCharonMono-Regular.ttf") as &[u8],
        FontSettings::default(),
    )
    .map_err(|e| anyhow::anyhow!("Font error: {e}"))?;

    let cells = parse_ansi(ansi);
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
    for pixel in img.pixels_mut() {
        *pixel = BG;
    }

    for (row_idx, row) in cells.iter().enumerate() {
        for (col_idx, &(ch, [r, g, b])) in row.iter().enumerate() {
            if ch == ' ' || ch == '\0' {
                continue;
            }

            let Ok(col_u32) = u32::try_from(col_idx) else {
                continue;
            };
            let Ok(row_u32) = u32::try_from(row_idx) else {
                continue;
            };

            let (metrics, bitmap) = font.rasterize(ch, FONT_SIZE);
            if metrics.width == 0 {
                continue; // glyph not in font, skip silently
            }

            let base_x = col_u32.saturating_mul(CELL_W);
            let base_y = row_u32.saturating_mul(CELL_H);
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
                        img.put_pixel(px_x, px_y, blend_pixel(r, g, b, coverage));
                    }
                }
            }
        }
    }

    Ok(img)
}
/// Alpha-blends a foreground color against the Tokyo Night background.
fn blend_pixel(r: u8, g: u8, b: u8, coverage: u8) -> Rgba<u8> {
    let alpha = f32::from(coverage) / 255.0;
    let blend = |fg: u8, bg: u8| -> u8 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let result = f32::from(bg).mul_add(1.0 - alpha, f32::from(fg) * alpha) as u8;
        result
    };
    let [br, bg_c, bb, _] = BG.0;
    Rgba([blend(r, br), blend(g, bg_c), blend(b, bb), 255])
}

/// Parses ANSI escape sequences into a grid of `(char, [r, g, b])` cells.
///
/// Only SGR truecolor sequences (`ESC[38;2;R;G;Bm`) are handled. All other
/// escape sequences are silently ignored. The default color is white.
fn parse_ansi(input: &[u8]) -> Vec<Vec<(char, [u8; 3])>> {
    let mut rows: Vec<Vec<(char, [u8; 3])>> = Vec::new();
    let mut current_row: Vec<(char, [u8; 3])> = Vec::new();
    let mut current_color: [u8; 3] = [255, 255, 255];
    let mut i = 0;

    while i < input.len() {
        if input[i] == 0x1b && input.get(i + 1) == Some(&b'[') {
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
                parse_color_params(params, &mut current_color);
            }
        } else if input[i] == b'\n' {
            rows.push(std::mem::take(&mut current_row));
            i += 1;
        } else {
            let ch = if input[i] < 128 {
                input[i] as char
            } else {
                let s = std::str::from_utf8(&input[i..]).unwrap_or(" ");
                let ch = s.chars().next().unwrap_or(' ');
                i += ch.len_utf8() - 1;
                ch
            };
            current_row.push((ch, current_color));
            i += 1;
        }
    }

    if !current_row.is_empty() {
        rows.push(current_row);
    }

    rows
}

/// Parses an SGR parameter string and updates the current color if a
/// truecolor sequence (`38;2;R;G;B`) is found. Resets to white on `0` or empty.
fn parse_color_params(params: &str, color: &mut [u8; 3]) {
    if params == "0" || params.is_empty() {
        *color = [255, 255, 255];
        return;
    }

    let parts: Vec<u8> = params.split(';').filter_map(|s| s.parse().ok()).collect();

    if parts.len() >= 5 && parts[0] == 38 && parts[1] == 2 {
        *color = [parts[2], parts[3], parts[4]];
    }
}

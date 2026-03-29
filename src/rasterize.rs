use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};

const FONT_SIZE: f32 = 14.0;
const CELL_W: u32 = 8; // pixels per character cell width
const CELL_H: u32 = 16; // pixels per character cell height

/// Parses ANSI escape sequences and rasterizes the output to a PNG buffer.
pub fn rasterize_ansi(ansi: &[u8]) -> anyhow::Result<RgbaImage> {
    let font_data = include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf");
    let fallback_data = include_bytes!("../assets/unifont-16.0.04.ttf");
    let font = Font::from_bytes(font_data as &[u8], FontSettings::default())
        .map_err(|e| anyhow::anyhow!("Font error: {e}"))?;
    let fallback = Font::from_bytes(fallback_data as &[u8], FontSettings::default())
        .map_err(|e| anyhow::anyhow!("Font error: {e}"))?;

    let (test_m, _) = fallback.rasterize('⣿', FONT_SIZE);
    eprintln!(
        "DEBUG unifont braille metrics: {}x{}",
        test_m.width, test_m.height
    );
    // Parse into a grid of (char, [r,g,b])
    let cells = parse_ansi(ansi);

    if cells.is_empty() {
        anyhow::bail!("No cells to render");
    }

    let cols = cells.iter().map(|r| r.len()).max().unwrap_or(0);
    let rows = cells.len();

    let img_w = cols as u32 * CELL_W;
    let img_h = rows as u32 * CELL_H;

    let mut img = RgbaImage::new(img_w, img_h);

    // Tokyo Night
    let bg = Rgba([26, 27, 38, 255]); // #1A1B26
    for pixel in img.pixels_mut() {
        *pixel = bg;
    }

    for (row_idx, row) in cells.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let (ch, [r, g, b]) = *cell;
            if ch == ' ' || ch == '\0' {
                continue;
            }

            let (metrics, bitmap) = {
                let is_braille = ('\u{2800}'..='\u{28FF}').contains(&ch);
                let is_box = ('\u{2500}'..='\u{259F}').contains(&ch);

                if is_braille || is_box {
                    // Always use unifont for braille and box drawing chars
                    fallback.rasterize(ch, FONT_SIZE)
                } else {
                    let (m, bmp) = font.rasterize(ch, FONT_SIZE);
                    if m.width == 0 {
                        fallback.rasterize(ch, FONT_SIZE)
                    } else {
                        (m, bmp)
                    }
                }
            };
            // let (metrics, bitmap) = font.rasterize(ch, FONT_SIZE);
            let base_x = col_idx as u32 * CELL_W;
            let base_y = row_idx as u32 * CELL_H;
            // Center glyph vertically in cell
            let y_offset = ((CELL_H as i32 - metrics.height as i32) / 2).max(0) as u32;

            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let coverage = bitmap[gy * metrics.width + gx];

                    if coverage > 0 {
                        let px_x = base_x + gx as u32;
                        let px_y = base_y + y_offset + gy as u32;
                        if px_x < img_w && px_y < img_h {
                            let alpha = coverage as f32 / 255.0;
                            let blend = |fg: u8, bg: u8| -> u8 {
                                (fg as f32 * alpha + bg as f32 * (1.0 - alpha)) as u8
                            };
                            let [br, bg_c, bb, _] = bg.0;
                            img.put_pixel(
                                px_x,
                                px_y,
                                Rgba([blend(r, br), blend(g, bg_c), blend(b, bb), 255]),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(img)
}

/// Parses ANSI escape sequences into a grid of (char, [r, g, b]) cells.
fn parse_ansi(input: &[u8]) -> Vec<Vec<(char, [u8; 3])>> {
    let mut rows: Vec<Vec<(char, [u8; 3])>> = Vec::new();
    let mut current_row: Vec<(char, [u8; 3])> = Vec::new();
    let mut current_color: [u8; 3] = [255, 255, 255]; // default white
    let mut i = 0;

    while i < input.len() {
        // ESC [ sequence
        if input[i] == 0x1b && i + 1 < input.len() && input[i + 1] == b'[' {
            i += 2;
            // Collect until final byte (letter)
            let mut seq = Vec::new();
            while i < input.len() && !input[i].is_ascii_alphabetic() {
                seq.push(input[i]);
                i += 1;
            }
            let final_byte = if i < input.len() { input[i] } else { 0 };
            i += 1;

            if final_byte == b'm' {
                let params = std::str::from_utf8(&seq).unwrap_or("");
                parse_color_params(params, &mut current_color);
            }
            // ignore other escape sequences (cursor movement etc.)
        } else if input[i] == b'\n' {
            rows.push(current_row.clone());
            current_row.clear();
            i += 1;
        } else {
            // Decode UTF-8 character
            let ch = if input[i] < 128 {
                input[i] as char
            } else {
                // Multi-byte UTF-8 — braille, block chars etc.
                let s = std::str::from_utf8(&input[i..]).unwrap_or(" ");
                let ch = s.chars().next().unwrap_or(' ');
                i += ch.len_utf8() - 1; // -1 because we add 1 below
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

/// Parses SGR color params like "38;2;255;100;50" into an RGB triple.
fn parse_color_params(params: &str, color: &mut [u8; 3]) {
    if params == "0" || params.is_empty() {
        *color = [255, 255, 255]; // reset to white
        return;
    }
    let parts: Vec<u8> = params.split(';').filter_map(|s| s.parse().ok()).collect();

    // Truecolor: 38;2;R;G;B
    if parts.len() >= 5 && parts[0] == 38 && parts[1] == 2 {
        *color = [parts[2], parts[3], parts[4]];
    }
}

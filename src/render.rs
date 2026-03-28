use crate::options::{CharsetMode, RenderOptions};
use image::{DynamicImage, GenericImageView, Rgba};
use std::io::Write;

/// This function takes a processed image and dispatches it to a specific
/// renderer based on the user's `CharsetMode` choice.
///
/// # Errors
///
/// Returns a [`std::io::Result`] if the `writer` fails to handle the stream of
/// ANSI escape sequences and characters.// Main rendering dispatch - handles all charset modes
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    writer: &mut W,
    options: RenderOptions,
) -> std::io::Result<()> {
    match options.charset {
        CharsetMode::Ansi => write_ansi_blocks(writer, img),
        CharsetMode::Unicode => write_unicode_blocks(writer, img, options.style.full),
        CharsetMode::Braille => write_braille(writer, img, options),
        CharsetMode::Fade => write_fade(writer, img, options),
        CharsetMode::Ascii => write_ascii(writer, img, options),
    }
}

/// Renders using ANSI "half-block" characters (`▀`).
///
/// This is a clever trick to double the vertical resolution: each terminal
/// character cell is split into a top and bottom half, allowing us to represent
/// two vertical pixels in the space of one.
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

/// If `full_block` is true, it uses two side-by-side full blocks (`██`) per pixel
/// to approximate a square shape. If false, it falls back to the more efficient
/// `write_ansi_blocks` (half-blocks).// Unicode blocks (full or half, based on `full` flag)
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
        write_ansi_blocks(writer, img)?;
    }
    Ok(())
}

/// Helper that writes a single character cell representing two pixels vertically.
///
/// It uses `TrueColor` (24-bit) ANSI escapes to set the foreground (top)
/// and background (bottom) colors.
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

/// Writes a single pixel as a double-width full block to maintain a 1:1 aspect ratio.
fn write_full_block<W: Write>(out: &mut W, px: Rgba<u8>) -> std::io::Result<()> {
    if px[3] > 0 {
        write!(out, "\x1b[38;2;{};{};{}m██", px[0], px[1], px[2])
    } else {
        write!(out, "  ")
    }
}

/// Renders using Braille patterns (U+2800 - U+28FF).
///
/// This provides the highest resolution (2x4 pixels per character cell).
/// It calculates the average color of all "lit" dots in a 2x4 grid to
/// determine the foreground color for that cell.
fn write_braille<W: Write>(
    writer: &mut W,
    img: &DynamicImage,
    _options: RenderOptions,
) -> std::io::Result<()> {
    let rgba = img.to_rgba8();
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
                        // let luma = 0.2126f32.mul_add(
                        //     f32::from(r),
                        //     0.0722f32.mul_add(f32::from(b), 0.7152 * f32::from(g)),
                        // );

                        let luma = (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b))
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
                write!(writer, "\x1b[0m\u{2800}")?;
            } else {
                let red = u8::try_from(r_sum / lit_count).unwrap_or(0);
                let green = u8::try_from(g_sum / lit_count).unwrap_or(0);
                let blue = u8::try_from(b_sum / lit_count).unwrap_or(0);
                let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or(' ');
                write!(writer, "\x1b[38;2;{red};{green};{blue}m{ch}")?;
            }
        }
        writeln!(writer, "\x1b[0m")?;
    }

    Ok(())
}

/// Renders using a short ramp of block shades (░▒▓█).
fn write_fade<W: Write>(
    writer: &mut W,
    img: &DynamicImage,
    _options: RenderOptions,
) -> std::io::Result<()> {
    // let charset: &[&str] = &[" ", "░", "▒", "▓", "█"];
    let charset: &[&str] = &["█", "▓", "▒", "░", " "];
    render_charset_colored(writer, img, charset)
}

fn write_ascii<W: Write>(
    writer: &mut W,
    img: &DynamicImage,
    _options: RenderOptions,
) -> std::io::Result<()> {
    let charset: &[&str] = &[
        " ", "`", ".", "-", "'", ":", "_", ",", "^", "=", ";", ">", "<", "+", "!", "r", "c", "*",
        "/", "z", "?", "s", "L", "T", "v", ")", "J", "7", "(", "|", "F", "i", "{", "C", "}", "f",
        "I", "3", "1", "t", "l", "u", "[", "n", "e", "o", "Z", "5", "Y", "x", "j", "y", "a", "]",
        "2", "E", "S", "w", "q", "k", "P", "6", "h", "9", "d", "4", "V", "p", "O", "G", "b", "U",
        "A", "K", "X", "H", "m", "8", "R", "D", "#", "$", "B", "g", "0", "M", "N", "W", "Q", "%",
        "&", "@",
    ];
    render_charset_colored(writer, img, charset)
}

/// This is our "Universal" renderer.
/// It doesn't care if the 'glyph' is a space, a letter, or a 2-column Emoji.
fn render_charset_colored<W: Write>(
    writer: &mut W,
    img: &DynamicImage,
    charset: &[&str],
) -> std::io::Result<()> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let num_chars_u32 = u32::try_from(charset.len()).unwrap_or(1);
    let num_chars = charset.len();

    // First pass: find the actual luma range of opaque pixels
    let mut luma_min = u32::MAX;
    let mut luma_max = u32::MIN;
    for y in 0..height {
        for x in 0..width {
            let px = rgba.get_pixel(x, y);
            let [red, green, blue, alpha] = px.0;
            if alpha >= 30 {
                let luma =
                    (2126 * u32::from(red) + 7152 * u32::from(green) + 722 * u32::from(blue))
                        / 10000;
                luma_min = luma_min.min(luma);
                luma_max = luma_max.max(luma);
            }
        }
    }

    // Avoid divide by zero if image is solid flat color
    let luma_range = (luma_max - luma_min).max(1);

    // Second pass: render using normalized luma
    for y in 0..height {
        for x in 0..width {
            let px = rgba.get_pixel(x, y);
            let [red, green, blue, alpha] = px.0;
            if alpha < 128 {
                write!(writer, "\x1b[0m ")?;
                continue;
            }
            let luma =
                (2126 * u32::from(red) + 7152 * u32::from(green) + 722 * u32::from(blue)) / 10000;

            // Normalize to 0..=255 based on actual image range
            let normalized = ((luma - luma_min) * 255) / luma_range;
            let idx = (normalized * (num_chars_u32 - 1) / 255) as usize;
            let idx = idx.min(num_chars - 1);
            let glyph = charset[idx];
            write!(writer, "\x1b[38;2;{red};{green};{blue}m{glyph}")?;
        }
        writeln!(writer, "\x1b[0m")?;
    }
    Ok(())
}

// fn render_charset_colored<W: Write>(
//     writer: &mut W,
//     img: &DynamicImage,
//     charset: &[&str], // Accepting a slice of strings is the secret sauce
// ) -> std::io::Result<()> {
//     let rgba = img.to_rgba8();
//     let (width, height) = rgba.dimensions();
//     let num_chars = charset.len();
//     let num_chars_u32 = u32::try_from(num_chars).unwrap_or(1);

//     for y in 0..height {
//         for x in 0..width {
//             let px = rgba.get_pixel(x, y);
//             let [red, green, blue, alpha] = px.0;

//             // If a pixel is transparent, we print a space.
//             // Note: If you use "Wide" charsets (like Emojis), you'll need
//             // to print TWO spaces here to keep the grid aligned.
//             if alpha < 128 {
//                 write!(writer, "\x1b[0m ")?;
//                 continue;
//             }

//             let luma =
//                 (2126 * u32::from(red) + 7152 * u32::from(green) + 722 * u32::from(blue)) / 10000;

//             // Map luma to our slice index
//             // let idx = (luma as usize * (num_chars - 1)) / 255;
//             let idx = (luma * (num_chars_u32 - 1) / 255) as usize;
//             let idx = idx.min(num_chars - 1);
//             let glyph = charset[idx];

//             write!(writer, "\x1b[38;2;{red};{green};{blue}m{glyph}")?;
//         }
//         writeln!(writer, "\x1b[0m")?;
//     }
//     Ok(())
// }

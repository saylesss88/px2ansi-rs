use image::{DynamicImage, GenericImageView, Rgba};
use std::io::Write;

/// This function iterates over the image pixels, processing them in vertical pairs
/// to utilize the "Upper Half Block" (▀) character. This effectively gives us
/// two vertical "sub-pixels" per character cell.
///
/// # Arguments
/// * `img` - The image to convert. Should be resized to fit the terminal before calling.
/// * `out` - A writable output (e.g., stdout or a file).
pub fn write_ansi_art<W: Write>(img: &DynamicImage, out: &mut W) -> std::io::Result<()> {
    let (width, height) = img.dimensions();

    // Iterate 2 rows at a time because each ANSI block character represents 2 vertical pixels.
    for y in (0..height).step_by(2) {
        for x in 0..width {
            let px1 = img.get_pixel(x, y);

            // Handle edge case: odd height images have no bottom pixel on the last row
            let px2 = if y + 1 < height {
                img.get_pixel(x, y + 1)
            } else {
                Rgba([0, 0, 0, 0]) // Treat as transparent
            };

            write_pixels(out, px1, px2)?;
        }
        // Reset color at the end of the row and add a newline
        writeln!(out, "\x1b[0m")?;
    }

    Ok(())
}

/// Helper to write the ANSI sequence for a single character block (2 vertical pixels).
fn write_pixels<W: Write>(out: &mut W, top: Rgba<u8>, bot: Rgba<u8>) -> std::io::Result<()> {
    let top_alpha = top[3];
    let bot_alpha = bot[3];

    if top_alpha > 0 {
        // CASE 1: Top pixel is visible
        // Set foreground color to Top Pixel
        write!(out, "\x1b[38;2;{};{};{}m", top[0], top[1], top[2])?;

        if bot_alpha > 0 {
            // CASE 1.1: Both pixels visible
            // Set background to Bottom Pixel and print "Upper Half Block" (▀).
            write!(out, "\x1b[48;2;{};{};{}m▀", bot[0], bot[1], bot[2])
        } else {
            // CASE 1.2: Only top visible
            // Reset background (transparent) and print "Upper Half Block" (▀).
            write!(out, "\x1b[49m▀")
        }
    } else {
        // CASE 2: Top pixel is transparent
        if bot_alpha > 0 {
            // CASE 2.1: Only bottom visible
            // Set foreground to Bottom Pixel and print "Lower Half Block" (▄).
            // We use reset background (\x1b[49m) to ensure top half is transparent.
            write!(out, "\x1b[38;2;{};{};{}m\x1b[49m▄", bot[0], bot[1], bot[2])
        } else {
            // CASE 2.2: Both transparent
            // Print a space with reset colors
            write!(out, "\x1b[0m ")
        }
    }
}

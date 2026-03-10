#![allow(clippy::multiple_crate_versions)]
use image::{DynamicImage, GenericImageView, Rgba};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Ansi,    // Standard half-blocks
    Unicode, // Full blocks or specialized chars (like pokemon-colorscripts)
}

/// Renders an image into the terminal using ANSI escape sequences.
///
/// Depending on the `mode`, this will either:
/// - **Ansi**: Squash two vertical pixels into one character cell using half-blocks (▀/▄).
/// - **Unicode**: Render each pixel as a double-width square block (██) for a retro look.
pub fn write_ansi_art<W: Write>(
    img: &DynamicImage,
    out: &mut W,
    mode: OutputMode,
    full_block: bool,
) -> std::io::Result<()> {
    let (width, height) = img.dimensions();

    match mode {
        OutputMode::Ansi => {
            // Ansi mode uses a "vertical pairing" trick to double the effective resolution.
            // By using the foreground for the top pixel and background for the bottom,
            // we can fit two pixels into a single character's space.
            for y in (0..height).step_by(2) {
                for x in 0..width {
                    let top = img.get_pixel(x, y);
                    let bot = if y + 1 < height {
                        img.get_pixel(x, y + 1)
                    } else {
                        Rgba([0, 0, 0, 0])
                    };
                    write_half_block(out, top, bot)?;
                }
                writeln!(out, "\x1b[0m")?;
            }
        }
        OutputMode::Unicode => {
            if full_block {
                // The "Pokemon-Colorscripts" look: 1 pixel = 2 wide characters
                for y in 0..height {
                    for x in 0..width {
                        let px = img.get_pixel(x, y);
                        write_full_block(out, px)?;
                    }
                    writeln!(out, "\x1b[0m")?;
                }
            } else {
                // The "Crisp Unicode" look: Uses half-blocks but on
                // a per-row basis or specialized logic
                for y in (0..height).step_by(2) {
                    for x in 0..width {
                        let top = img.get_pixel(x, y);
                        let bot = if y + 1 < height {
                            img.get_pixel(x, y + 1)
                        } else {
                            Rgba([0, 0, 0, 0])
                        };
                        write_half_block(out, top, bot)?;
                    }
                    writeln!(out, "\x1b[0m")?;
                }
            }
        }
    }
    Ok(())
}

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

// Emulating the "solid" look of some scripts
fn write_full_block<W: Write>(out: &mut W, px: Rgba<u8>) -> std::io::Result<()> {
    if px[3] > 0 {
        // Print TWO blocks for every ONE pixel
        write!(out, "\x1b[38;2;{};{};{}m██", px[0], px[1], px[2])
    } else {
        write!(out, "  ") // Two spaces
    }
}

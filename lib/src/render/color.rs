#[cfg(feature = "parallel")]
use std::fmt::Write as FmtWrite;
use std::io::Write;

use super::types::ColorMode;

/// Tracks the terminal's current SGR (Select Graphic Rendition) state.
///
/// This is used to optimize the output stream by skipping redundant color
/// escape codes if the next pixel has the same color as the previous one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ColorState {
    #[default]
    None,
    /// 24-bit `TrueColor`: (R, G, B)
    TrueColor(u8, u8, u8),
    /// Xterm-style 256 color index
    Ansi256(u8),
}

/// Writes a single character (glyph) with the specified foreground color.
///
/// This function is "state-aware"—it checks `last` to see if a color
/// change is actually necessary before writing the ANSI sequence.
pub(super) fn write_colored_glyph<W: Write>(
    writer: &mut W,
    glyph: &str,
    r: u8,
    g: u8,
    b: u8,
    color_mode: ColorMode,
    last: &mut ColorState,
) -> std::io::Result<()> {
    match color_mode {
        ColorMode::TrueColor => {
            // Only emit the 24-bit color sequence (\x1b[38;2;R;G;Bm) if the color changed
            if !matches!(last, ColorState::TrueColor(lr, lg, lb) if *lr == r && *lg == g && *lb == b)
            {
                write!(writer, "\x1b[38;2;{r};{g};{b}m")?;
                *last = ColorState::TrueColor(r, g, b);
            }
            writer.write_all(glyph.as_bytes())
        }
        ColorMode::Ansi256 => {
            // Convert RGB to the closest 8-bit color index (\x1b[38;5;Idxm)
            let idx = crate::color::rgb_to_xterm256(r, g, b);
            if !matches!(last, ColorState::Ansi256(li) if *li == idx) {
                write!(writer, "\x1b[38;5;{idx}m")?;
                *last = ColorState::Ansi256(idx);
            }
            writer.write_all(glyph.as_bytes())
        }
        ColorMode::None => writer.write_all(glyph.as_bytes()),
    }
}

/// Renders two vertical pixels using half-block characters (▀ or ▄).
///
/// This technique uses the foreground color for the top half and the
/// background color for the bottom half, effectively doubling vertical resolution.
pub(super) fn write_half_block<W: Write>(
    out: &mut W,
    top: image::Rgba<u8>,
    bot: image::Rgba<u8>,
) -> std::io::Result<()> {
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

/// Renders a single pixel as a "double-wide" block (██).
///
/// This is used for modes that don't support sub-pixel resolution,
/// providing a chunky, square-pixel look.
pub(super) fn write_full_block<W: Write>(out: &mut W, px: image::Rgba<u8>) -> std::io::Result<()> {
    if px[3] > 0 {
        write!(out, "\x1b[38;2;{};{};{}m██", px[0], px[1], px[2])
    } else {
        write!(out, "  ")
    }
}

/// A specialized version of `write_colored_glyph` for parallel rendering.
///
/// Since parallel threads render chunks into independent Strings, we don't
/// track state between them (it would require complex locking). Instead,
/// every glyph gets its own color sequence.
#[cfg(feature = "parallel")]
pub(super) fn write_colored_glyph_to_str(
    buf: &mut String,
    glyph: &str,
    r: u8,
    g: u8,
    b: u8,
    color_mode: ColorMode,
) {
    match color_mode {
        ColorMode::TrueColor => {
            let _ = write!(buf, "\x1b[38;2;{r};{g};{b}m{glyph}");
        }
        ColorMode::Ansi256 => {
            let idx = crate::color::rgb_to_xterm256(r, g, b);
            let _ = write!(buf, "\x1b[38;5;{idx}m{glyph}");
        }
        ColorMode::None => {
            buf.push_str(glyph);
        }
    }
}

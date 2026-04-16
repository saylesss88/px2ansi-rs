#[cfg(feature = "parallel")]
use std::fmt::Write as FmtWrite;
use std::io::Write;

use super::types::ColorMode;

/// Tracks the last-written color to suppress redundant ANSI escape sequences.
#[derive(Default)]
pub(super) enum ColorState {
    #[default]
    None,
    TrueColor(u8, u8, u8),
    Ansi256(u8),
}

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
            if !matches!(last, ColorState::TrueColor(lr, lg, lb) if *lr == r && *lg == g && *lb == b)
            {
                write!(writer, "\x1b[38;2;{r};{g};{b}m")?;
                *last = ColorState::TrueColor(r, g, b);
            }
            writer.write_all(glyph.as_bytes())
        }
        ColorMode::Ansi256 => {
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

pub(super) fn write_full_block<W: Write>(out: &mut W, px: image::Rgba<u8>) -> std::io::Result<()> {
    if px[3] > 0 {
        write!(out, "\x1b[38;2;{};{};{}m██", px[0], px[1], px[2])
    } else {
        write!(out, "  ")
    }
}

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

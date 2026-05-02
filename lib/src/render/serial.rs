use std::io;
use std::io::Write;

use super::color::ColorState;
use super::pixel::{ColorParams, LumaParams, PixelRgba, RenderCtx, write_pixel_scalar};

/// Renders the image to the provided writer in a single-threaded, serial fashion.
///
/// Iterates row-by-row, mapping luma to charset indices and emitting terminal
/// escape codes for color. Wide mode samples every other pixel column.
///
/// ### Error Handling
/// Returns `io::Result::Err` if the writer fails to accept bytes.
pub(super) fn render_serial<W: Write>(
    writer: &mut W,
    ctx: &RenderCtx<'_>,
    lp: LumaParams,
    cp: ColorParams<'_>,
) -> io::Result<()> {
    let raw = ctx.rgba.as_raw();
    let mut last_color = ColorState::default();

    for y in 0..ctx.height {
        let row_start = (y * ctx.width) as usize * 4;
        let row_bytes = &raw[row_start..row_start + ctx.width as usize * 4];

        if ctx.wide {
            for x in (0..ctx.width as usize).step_by(2) {
                let base = x * 4;
                let px = PixelRgba {
                    r: row_bytes[base],
                    g: row_bytes[base + 1],
                    b: row_bytes[base + 2],
                    a: row_bytes[base + 3],
                };
                write_pixel_scalar(writer, ctx.charset, px, lp, cp, &mut last_color)?;
            }
        } else {
            for px in row_bytes.chunks_exact(4) {
                let px = PixelRgba {
                    r: px[0],
                    g: px[1],
                    b: px[2],
                    a: px[3],
                };
                write_pixel_scalar(writer, ctx.charset, px, lp, cp, &mut last_color)?;
            }
        }

        if cp.enabled {
            writeln!(writer, "\x1b[0m")?;
            last_color = ColorState::default();
        } else {
            writeln!(writer)?;
        }
    }

    Ok(())
}

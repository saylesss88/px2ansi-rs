use std::io;
use std::io::Write;

use super::color::ColorState;
use super::pixel::{ColorParams, LumaParams, PixelRgba, RenderCtx, write_pixel_scalar};

#[cfg(feature = "simd")]
use super::pixel::write_pixel;

/// Renders the image to the provided writer in a single-threaded, serial fashion.
///
/// This function iterates through the image row-by-row, handling character mapping,
/// color state transitions, and terminal escape codes. It automatically selects
/// between SIMD-accelerated paths and scalar fallbacks based on the crate features
/// and the specific rendering context (e.g., "wide" mode).
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
    // last_color tracks the terminal's current ANSI state to avoid redundant escape codes.
    let mut last_color = ColorState::default();

    for y in 0..ctx.height {
        let row_start = (y * ctx.width) as usize * 4;
        let row_bytes = &raw[row_start..row_start + ctx.width as usize * 4];

        if ctx.wide {
            // Wide mode uses a simplified scalar path, skipping every other pixel
            // to maintain the aspect ratio on terminal grids.
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
            // Standard mode processes chunks for better throughput.
            let chunks = row_bytes.chunks_exact(32);
            let remainder = chunks.remainder();

            #[cfg(feature = "simd")]
            {
                // SIMD path: Processes batches of pixels to calculate luminance and
                // character indices before writing to the buffer.
                let mut x_off = 0usize;
                for chunk in chunks {
                    let Ok(chunk32) = chunk.try_into() else {
                        continue;
                    };
                    let pairs = crate::simd::compute_charset_indices(
                        chunk32,
                        lp.min,
                        lp.range,
                        lp.num_chars_minus_1,
                    );
                    for (idx, opaque) in pairs {
                        let base = x_off * 4;
                        let px = PixelRgba {
                            r: row_bytes[base],
                            g: row_bytes[base + 1],
                            b: row_bytes[base + 2],
                            a: row_bytes[base + 3],
                        };
                        write_pixel(
                            writer,
                            ctx.charset,
                            idx as usize,
                            px,
                            opaque,
                            cp,
                            &mut last_color,
                        )?;
                        x_off += 1;
                    }
                }
            }
            #[cfg(not(feature = "simd"))]
            for chunk in chunks {
                for px in chunk.chunks_exact(4) {
                    let px = PixelRgba {
                        r: px[0],
                        g: px[1],
                        b: px[2],
                        a: px[3],
                    };
                    write_pixel_scalar(writer, ctx.charset, px, lp, cp, &mut last_color)?;
                }
            }
            // Clean up any pixels that didn't fit into a full 32-byte chunk.
            for px in remainder.chunks_exact(4) {
                let px = PixelRgba {
                    r: px[0],
                    g: px[1],
                    b: px[2],
                    a: px[3],
                };
                write_pixel_scalar(writer, ctx.charset, px, lp, cp, &mut last_color)?;
            }
        }
        // Handle line endings. If color is enabled, we reset the SGR state
        // at the end of every line to prevent bleed in certain terminal emulators.
        if cp.enabled {
            writeln!(writer, "\x1b[0m")?;
            last_color = ColorState::default();
        } else {
            writeln!(writer)?;
        }
    }
    Ok(())
}

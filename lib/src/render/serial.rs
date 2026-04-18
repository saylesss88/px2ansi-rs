use std::io::Write;

use super::color::ColorState;
use super::pixel::{ColorParams, LumaParams, PixelRgba, RenderCtx, write_pixel_scalar};

#[cfg(feature = "simd")]
use super::pixel::write_pixel;

pub(super) fn render_serial<W: Write>(
    writer: &mut W,
    ctx: &RenderCtx<'_>,
    lp: LumaParams,
    cp: ColorParams<'_>,
) -> std::io::Result<()> {
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
            let chunks = row_bytes.chunks_exact(32);
            let remainder = chunks.remainder();

            #[cfg(feature = "simd")]
            {
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

        if cp.enabled {
            writeln!(writer, "\x1b[0m")?;
            last_color = ColorState::default();
        } else {
            writeln!(writer)?;
        }
    }
    Ok(())
}

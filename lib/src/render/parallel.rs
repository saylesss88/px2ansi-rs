use std::io::Write;

use rayon::prelude::*;

use super::color::write_colored_glyph_to_str;
use super::pixel::{ALPHA_THRESHOLD, ColorParams, LumaParams, RenderCtx};

pub(super) fn render_parallel<W: Write>(
    writer: &mut W,
    ctx: &RenderCtx,
    lp: LumaParams,
    cp: ColorParams<'_>,
) -> std::io::Result<()> {
    let rows: Vec<String> = (0..ctx.height)
        .into_par_iter()
        .map(|y| {
            let mut row = String::with_capacity(ctx.width as usize * 12);
            for x in (0..ctx.width).step_by(ctx.x_step) {
                let [r, g, b, a] = ctx.rgba.get_pixel(x, y).0;
                if a < ALPHA_THRESHOLD {
                    if cp.enabled {
                        row.push_str("\x1b[0m");
                    }
                    row.push_str(cp.blank);
                    continue;
                }
                let luma = crate::simd::luma_scalar(r, g, b);
                let norm = ((luma - lp.min) * 255) / lp.range;
                let idx = ((norm * lp.num_chars_minus_1 / 255) as usize).min(ctx.charset.len() - 1);

                if cp.enabled {
                    write_colored_glyph_to_str(&mut row, ctx.charset[idx], r, g, b, cp.mode);
                } else {
                    row.push_str(ctx.charset[idx]);
                }
            }
            if cp.enabled {
                row.push_str("\x1b[0m\n");
            } else {
                row.push('\n');
            }
            row
        })
        .collect();

    for row in rows {
        writer.write_all(row.as_bytes())?;
    }
    Ok(())
}

use image::RgbaImage;
use std::io::Write;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::color::{ColorState, write_colored_glyph};
use super::types::ColorMode;

pub(super) const ALPHA_THRESHOLD: u8 = 30;

#[derive(Clone, Copy)]
pub(super) struct LumaParams {
    pub(super) min: u32,
    pub(super) range: u32,
    pub(super) num_chars_minus_1: u32,
}

#[derive(Clone, Copy)]
pub(super) struct ColorParams<'a> {
    pub(super) enabled: bool,
    pub(super) mode: ColorMode,
    pub(super) blank: &'a str,
}

pub(super) struct RenderCtx<'a> {
    pub(super) rgba: &'a RgbaImage,
    pub(super) charset: &'a [&'a str],
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) x_step: usize,
    pub(super) wide: bool,
}

impl<'a> RenderCtx<'a> {
    pub(super) fn new(rgba: &'a RgbaImage, charset: &'a [&'a str], wide: bool) -> Self {
        let (width, height) = rgba.dimensions();
        Self {
            rgba,
            charset,
            width,
            height,
            x_step: if wide { 2 } else { 1 },
            wide,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PixelRgba {
    pub(super) r: u8,
    pub(super) g: u8,
    pub(super) b: u8,
    pub(super) a: u8,
}

pub(super) fn luma_range_pass1(
    rgba: &RgbaImage,
    width: u32,
    height: u32,
    x_step: usize,
    wide: bool,
    use_parallel: bool,
) -> (u32, u32) {
    if use_parallel {
        #[cfg(feature = "parallel")]
        {
            if wide {
                return rgba
                    .as_raw()
                    .par_chunks_exact(width as usize * 4)
                    .map(|row| {
                        let mut lo = u32::MAX;
                        let mut hi = u32::MIN;
                        let chunks = row.chunks_exact(8);
                        let rem = chunks.remainder();
                        for px in chunks {
                            if px[3] >= ALPHA_THRESHOLD {
                                let l = crate::simd::luma_scalar(px[0], px[1], px[2]);
                                lo = lo.min(l);
                                hi = hi.max(l);
                            }
                        }
                        if rem.len() >= 4 && rem[3] >= ALPHA_THRESHOLD {
                            let l = crate::simd::luma_scalar(rem[0], rem[1], rem[2]);
                            lo = lo.min(l);
                            hi = hi.max(l);
                        }
                        (lo, hi)
                    })
                    .reduce(
                        || (u32::MAX, u32::MIN),
                        |(m1, x1), (m2, x2)| (m1.min(m2), x1.max(x2)),
                    );
            }
            return rgba
                .as_raw()
                .par_chunks(32 * 1024)
                .map(crate::simd::find_luma_range_rgba_bytes)
                .reduce(
                    || (u32::MAX, u32::MIN),
                    |(m1, x1), (m2, x2)| (m1.min(m2), x1.max(x2)),
                );
        }
        #[cfg(not(feature = "parallel"))]
        let _ = (width, height, x_step, wide);
    }

    if wide {
        let mut lo = u32::MAX;
        let mut hi = u32::MIN;
        for y in 0..height {
            for x in (0..width).step_by(x_step) {
                let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
                if alpha >= ALPHA_THRESHOLD {
                    let l = crate::simd::luma_scalar(red, green, blue);
                    lo = lo.min(l);
                    hi = hi.max(l);
                }
            }
        }
        (lo, hi)
    } else {
        crate::simd::find_luma_range_rgba_bytes(rgba.as_raw())
    }
}

/// Writes one pixel whose charset index was pre-computed by SIMD.
#[cfg(feature = "simd")]
#[inline]
pub(super) fn write_pixel<W: Write>(
    writer: &mut W,
    charset: &[&str],
    idx: usize,
    px: PixelRgba,
    opaque: bool,
    cp: ColorParams<'_>,
    last: &mut ColorState,
) -> std::io::Result<()> {
    if !opaque || px.a < ALPHA_THRESHOLD {
        write_blank(writer, cp, last)
    } else {
        let glyph = charset[idx.min(charset.len() - 1)];
        write_glyph(writer, glyph, px.r, px.g, px.b, cp, last)
    }
}

/// Scalar luma → index → write for one pixel.
#[inline]
pub(super) fn write_pixel_scalar<W: Write>(
    writer: &mut W,
    charset: &[&str],
    px: PixelRgba,
    lp: LumaParams,
    cp: ColorParams<'_>,
    last: &mut ColorState,
) -> std::io::Result<()> {
    if px.a < ALPHA_THRESHOLD {
        return write_blank(writer, cp, last);
    }
    let luma = crate::simd::luma_scalar(px.r, px.g, px.b);
    let norm = ((luma - lp.min) * 255) / lp.range;
    let idx = ((norm * lp.num_chars_minus_1 / 255) as usize).min(charset.len() - 1);
    write_glyph(writer, charset[idx], px.r, px.g, px.b, cp, last)
}

/// Writes a transparent cell (blank + optional reset escape).
#[inline]
pub(super) fn write_blank<W: Write>(
    writer: &mut W,
    cp: ColorParams<'_>,
    last: &mut ColorState,
) -> std::io::Result<()> {
    if cp.enabled {
        write!(writer, "\x1b[0m{}", cp.blank)?;
        *last = ColorState::default();
    } else {
        write!(writer, "{}", cp.blank)?;
    }
    Ok(())
}

/// Writes a colored or plain glyph.
#[inline]
pub(super) fn write_glyph<W: Write>(
    writer: &mut W,
    glyph: &str,
    r: u8,
    g: u8,
    b: u8,
    cp: ColorParams<'_>,
    last: &mut ColorState,
) -> std::io::Result<()> {
    if cp.enabled {
        write_colored_glyph(writer, glyph, r, g, b, cp.mode, last)
    } else {
        write!(writer, "{glyph}")
    }
}

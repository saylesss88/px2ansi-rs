//! SIMD-accelerated pixel processing utilities using the `wide` crate.
//!
//! Falls back to scalar when the `simd` feature is disabled.
#[cfg(feature = "simd")]
use wide::CmpGt;

/// Alpha threshold below which a pixel is considered transparent.
pub const ALPHA_THRESHOLD: u8 = 30;

/// Compute Rec.709 luma for a single pixel (scalar).
#[inline]
#[must_use]
pub fn luma_scalar(r: u8, g: u8, b: u8) -> u32 {
    (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b)) / 10000
}

/// Scan a contiguous RGBA byte slice and return `(min, max)` luma values
/// of all opaque pixels (alpha ≥ `ALPHA_THRESHOLD`).
///
/// Returns `(u32::MAX, u32::MIN)` if no opaque pixel is found.
#[must_use]
pub fn find_luma_range_rgba_bytes(bytes: &[u8]) -> (u32, u32) {
    #[cfg(feature = "simd")]
    return find_luma_range_simd(bytes);

    #[cfg(not(feature = "simd"))]
    find_luma_range_scalar(bytes)
}

/// Scalar fallback.
#[allow(dead_code)]
#[cfg(any(not(feature = "simd"), test))]
fn find_luma_range_scalar(bytes: &[u8]) -> (u32, u32) {
    let mut min = u32::MAX;
    let mut max = u32::MIN;
    for pixel in bytes.chunks_exact(4) {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        if a >= ALPHA_THRESHOLD {
            let luma = luma_scalar(r, g, b);
            min = min.min(luma);
            max = max.max(luma);
        }
    }
    (min, max)
}

/// SIMD path using the `wide` crate — processes 8 pixels at a time.
#[cfg(feature = "simd")]
fn find_luma_range_simd(bytes: &[u8]) -> (u32, u32) {
    use wide::u32x8;

    let w_r = u32x8::splat(2126);
    let w_g = u32x8::splat(7152);
    let w_b = u32x8::splat(722);
    let thresh = u32x8::splat(u32::from(ALPHA_THRESHOLD));

    let mut min = u32::MAX;
    let mut max = u32::MIN;

    // Process 8 pixels (32 bytes) at a time
    let chunks = bytes.chunks_exact(32);
    let remainder = chunks.remainder();

    for chunk in chunks {
        // Gather each channel from interleaved RGBA into separate u32x8 lanes
        let r = u32x8::new([
            u32::from(chunk[0]),
            u32::from(chunk[4]),
            u32::from(chunk[8]),
            u32::from(chunk[12]),
            u32::from(chunk[16]),
            u32::from(chunk[20]),
            u32::from(chunk[24]),
            u32::from(chunk[28]),
        ]);
        let g = u32x8::new([
            u32::from(chunk[1]),
            u32::from(chunk[5]),
            u32::from(chunk[9]),
            u32::from(chunk[13]),
            u32::from(chunk[17]),
            u32::from(chunk[21]),
            u32::from(chunk[25]),
            u32::from(chunk[29]),
        ]);
        let b = u32x8::new([
            u32::from(chunk[2]),
            u32::from(chunk[6]),
            u32::from(chunk[10]),
            u32::from(chunk[14]),
            u32::from(chunk[18]),
            u32::from(chunk[22]),
            u32::from(chunk[26]),
            u32::from(chunk[30]),
        ]);
        let a = u32x8::new([
            u32::from(chunk[3]),
            u32::from(chunk[7]),
            u32::from(chunk[11]),
            u32::from(chunk[15]),
            u32::from(chunk[19]),
            u32::from(chunk[23]),
            u32::from(chunk[27]),
            u32::from(chunk[31]),
        ]);

        // luma = (2126*R + 7152*G + 722*B) / 10000
        let luma_unscaled = r * w_r + g * w_g + b * w_b;

        // Extract and process only opaque pixels
        let lumas: [u32; 8] = luma_unscaled.into();
        let alphas: [u32; 8] = a.into();

        // a >= thresh  is equivalent to  a > thresh - 1
        let opaque_mask = a.simd_gt(thresh - u32x8::splat(1));
        if opaque_mask.to_array() == [0u32; 8] {
            continue;
        }

        for i in 0..8 {
            if alphas[i] >= u32::from(ALPHA_THRESHOLD) {
                let luma = lumas[i] / 10000;
                min = min.min(luma);
                max = max.max(luma);
            }
        }
    }

    // Scalar remainder
    for pixel in remainder.chunks_exact(4) {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        if a >= ALPHA_THRESHOLD {
            let luma = luma_scalar(r, g, b);
            min = min.min(luma);
            max = max.max(luma);
        }
    }

    (min, max)
}

/// For 8 RGBA pixels, compute the charset index for each opaque pixel.
///
/// Returns an array of `(luma_index, is_opaque)` pairs — `is_opaque` is `true`
/// when the pixel's alpha ≥ `ALPHA_THRESHOLD`.
///
/// `luma_min`, `luma_range`, `num_chars_minus_1` are pre-computed from Pass 1.
#[cfg(feature = "simd")]
#[must_use]
pub fn compute_charset_indices(
    chunk: &[u8; 32],
    luma_min: u32,
    luma_range: u32,
    num_chars_minus_1: u32,
) -> [(u32, bool); 8] {
    use wide::u32x8;

    let r = u32x8::new([
        u32::from(chunk[0]),
        u32::from(chunk[4]),
        u32::from(chunk[8]),
        u32::from(chunk[12]),
        u32::from(chunk[16]),
        u32::from(chunk[20]),
        u32::from(chunk[24]),
        u32::from(chunk[28]),
    ]);
    let g = u32x8::new([
        u32::from(chunk[1]),
        u32::from(chunk[5]),
        u32::from(chunk[9]),
        u32::from(chunk[13]),
        u32::from(chunk[17]),
        u32::from(chunk[21]),
        u32::from(chunk[25]),
        u32::from(chunk[29]),
    ]);
    let b = u32x8::new([
        u32::from(chunk[2]),
        u32::from(chunk[6]),
        u32::from(chunk[10]),
        u32::from(chunk[14]),
        u32::from(chunk[18]),
        u32::from(chunk[22]),
        u32::from(chunk[26]),
        u32::from(chunk[30]),
    ]);
    let a: [u32; 8] = [
        u32::from(chunk[3]),
        u32::from(chunk[7]),
        u32::from(chunk[11]),
        u32::from(chunk[15]),
        u32::from(chunk[19]),
        u32::from(chunk[23]),
        u32::from(chunk[27]),
        u32::from(chunk[31]),
    ];

    let luma_raw: [u32; 8] =
        (r * u32x8::splat(2126) + g * u32x8::splat(7152) + b * u32x8::splat(722)).into();

    let thresh = u32::from(ALPHA_THRESHOLD);
    let mut out = [(0u32, false); 8];

    for i in 0..8 {
        let luma = luma_raw[i] / 10000;
        let norm = ((luma.saturating_sub(luma_min)) * 255) / luma_range;
        let idx = (norm * num_chars_minus_1 / 255).min(num_chars_minus_1);
        out[i] = (idx, a[i] >= thresh);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_luma_white() {
        assert_eq!(luma_scalar(255, 255, 255), 255);
    }

    #[test]
    fn scalar_luma_black() {
        assert_eq!(luma_scalar(0, 0, 0), 0);
    }

    #[test]
    fn find_range_single_opaque_pixel() {
        let bytes = [255, 0, 0, 255];
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        let expected = luma_scalar(255, 0, 0);
        assert_eq!(min, expected);
        assert_eq!(max, expected);
    }

    #[test]
    fn find_range_transparent_pixel_ignored() {
        let bytes = [255, 255, 255, 0];
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, u32::MAX);
        assert_eq!(max, u32::MIN);
    }

    #[test]
    fn find_range_mixed() {
        let bytes = [0, 0, 0, 255, 255, 255, 255, 255];
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, 0);
        assert_eq!(max, 255);
    }

    #[test]
    fn find_range_skips_below_alpha_threshold() {
        let bytes = [
            128, 128, 128, 29, // alpha 29 < 30 — ignored
            100, 100, 100, 255,
        ];
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        let expected = luma_scalar(100, 100, 100);
        assert_eq!(min, expected);
        assert_eq!(max, expected);
    }

    #[test]
    fn find_range_many_pixels_exercises_simd_path() {
        let mut bytes = Vec::with_capacity(64);
        for i in 0u8..16 {
            let v = i * 16;
            bytes.extend_from_slice(&[v, v, v, 255]);
        }
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, luma_scalar(0, 0, 0));
        assert_eq!(max, luma_scalar(240, 240, 240));
    }

    #[test]
    fn find_range_all_transparent_many_pixels() {
        let pixel = [100u8, 200, 50, 0];
        let bytes: Vec<u8> = pixel.iter().copied().cycle().take(4 * 32).collect();
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, u32::MAX);
        assert_eq!(max, u32::MIN);
    }

    #[test]
    fn scalar_matches_inline_formula() {
        for r in (0..=255u8).step_by(17) {
            for g in (0..=255u8).step_by(17) {
                for b in (0..=255u8).step_by(17) {
                    let expected =
                        (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b)) / 10000;
                    assert_eq!(luma_scalar(r, g, b), expected);
                }
            }
        }
    }
    #[test]
    #[cfg(feature = "simd")]
    fn find_range_simd_vs_scalar_fuzz() {
        // Test various buffer lengths to check SIMD chunks + remainder logic
        for len in [4, 12, 32, 36, 64, 128, 132] {
            let mut bytes = Vec::with_capacity(len);
            for i in 0..(len / 4) {
                // let v = (i % 255) as u8;
                //
                // Masking ensures the value is 0-255, then we cast.
                // Clippy usually ignores this cast because it's a common pattern.
                let v = u8::try_from(i % 255)
                    .unwrap_or_else(|_| unreachable!("i % 255 must fit in u8"));
                // Alternate alpha to test the SIMD mask logic
                let alpha = if i % 3 == 0 { 0 } else { 255 };
                bytes.extend_from_slice(&[v, v, v, alpha]);
            }

            // 1. Calculate using your SIMD function
            let (simd_min, simd_max) = find_luma_range_simd(&bytes);

            // 2. Calculate using a simple manual scalar loop
            let mut scalar_min = u32::MAX;
            let mut scalar_max = u32::MIN;
            for pixel in bytes.chunks_exact(4) {
                if pixel[3] >= ALPHA_THRESHOLD {
                    let l = luma_scalar(pixel[0], pixel[1], pixel[2]);
                    scalar_min = scalar_min.min(l);
                    scalar_max = scalar_max.max(l);
                }
            }

            assert_eq!(simd_min, scalar_min, "Min mismatch at len {len}");
            assert_eq!(simd_max, scalar_max, "Max mismatch at len {len}");
        }
    }
}

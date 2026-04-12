//! SIMD-accelerated pixel processing utilities.
//!
//! Falls back to scalar implementations when the `simd` feature is disabled
//! or the target architecture doesn't support the required instructions.

/// Alpha threshold below which a pixel is considered transparent.
const ALPHA_THRESHOLD: u8 = 30;

/// Compute Rec.709 luma for a single pixel (scalar).
#[inline]
#[must_use]
pub fn luma_scalar(r: u8, g: u8, b: u8) -> u32 {
    (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b)) / 10000
}

/// Scan a contiguous RGBA byte slice and return the `(min, max)` luma values
/// of all opaque pixels (alpha ≥ 30).
///
/// The input is raw RGBA bytes — i.e. `&[u8]` where every consecutive group
/// of 4 bytes is `[R, G, B, A]`.  This is exactly the layout returned by
/// `image::RgbaImage::as_raw()`.
///
/// If no opaque pixel is found the sentinel pair `(u32::MAX, u32::MIN)` is
/// returned so callers can detect fully-transparent images the same way the
/// existing code does.
///
/// When the `simd` feature is enabled *and* the CPU supports the right
/// instruction set, this dispatches to a SIMD fast-path.  Otherwise it falls
/// back to a portable scalar loop.
#[inline]
#[must_use]
pub fn find_luma_range_rgba_bytes(bytes: &[u8]) -> (u32, u32) {
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: we just confirmed AVX2 support at runtime.
            return unsafe { find_luma_range_avx2(bytes) };
        }
        // SAFETY: SSE2 is always available on x86_64.
        return unsafe { find_luma_range_sse2(bytes) };
    }

    #[allow(unreachable_code)]
    find_luma_range_scalar(bytes)
}

// ── Scalar fallback ──────────────────────────────────────────────────────────

/// Pure-scalar scan — no feature gates required.
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

// ── SSE2 fast-path ───────────────────────────────────────────────────────────

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn find_luma_range_sse2(bytes: &[u8]) -> (u32, u32) {
    use std::arch::x86_64::{
        __m128i, _mm_cmpgt_epi32, _mm_cvtsi128_si32, _mm_loadu_si128, _mm_movemask_epi8,
        _mm_set1_epi32, _mm_setzero_si128, _mm_shuffle_epi32, _mm_sub_epi32, _mm_unpackhi_epi8,
        _mm_unpackhi_epi16, _mm_unpacklo_epi8, _mm_unpacklo_epi16,
    };

    let mut min = u32::MAX;
    let mut max = u32::MIN;

    // Process 4 pixels (16 bytes) at a time
    let chunks = bytes.chunks_exact(16);
    let remainder = chunks.remainder();

    let zero = { _mm_setzero_si128() };
    let alpha_thresh_i32 = { _mm_set1_epi32(i32::from(ALPHA_THRESHOLD)) };

    for chunk in chunks {
        let raw = unsafe { _mm_loadu_si128(chunk.as_ptr().cast()) };

        // Unpack interleaved RGBA to per-channel 32-bit lanes.
        // SSE2 lacks _mm_shuffle_epi8, so we widen in two stages:
        //   bytes → 16-bit → 32-bit

        // Stage 1: bytes → 16-bit
        let lo16 = { _mm_unpacklo_epi8(raw, zero) }; // pixels 0,1 as u16
        let hi16 = { _mm_unpackhi_epi8(raw, zero) }; // pixels 2,3 as u16

        // Stage 2: 16-bit → 32-bit (4 values each)
        let px01_lo = { _mm_unpacklo_epi16(lo16, zero) }; // pixel 0: [R0 G0 B0 A0] as u32
        let px01_hi = { _mm_unpackhi_epi16(lo16, zero) }; // pixel 1
        let px23_lo = { _mm_unpacklo_epi16(hi16, zero) }; // pixel 2
        let px23_hi = { _mm_unpackhi_epi16(hi16, zero) }; // pixel 3

        // Each register holds [R, G, B, A] as four i32 lanes.
        let pixels: [__m128i; 4] = [px01_lo, px01_hi, px23_lo, px23_hi];

        for px in &pixels {
            // Lane 3 = A
            let a_lane = { _mm_shuffle_epi32::<0xFF>(*px) };
            let opaque =
                { _mm_cmpgt_epi32(a_lane, _mm_sub_epi32(alpha_thresh_i32, _mm_set1_epi32(1))) };

            if { _mm_movemask_epi8(opaque) } == 0 {
                continue;
            }

            let r_val = { _mm_cvtsi128_si32(*px) }.cast_unsigned();
            let g_val = { _mm_cvtsi128_si32(_mm_shuffle_epi32::<0x55>(*px)) }.cast_unsigned();
            let b_val = { _mm_cvtsi128_si32(_mm_shuffle_epi32::<0xAA>(*px)) }.cast_unsigned();

            let luma = (2126 * r_val + 7152 * g_val + 722 * b_val) / 10000;
            min = min.min(luma);
            max = max.max(luma);
        }
    }

    // Handle remaining pixels
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

// ── AVX2 fast-path ───────────────────────────────────────────────────────────

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn find_luma_range_avx2(bytes: &[u8]) -> (u32, u32) {
    use std::arch::x86_64::{
        _mm256_add_epi32, _mm256_cmpgt_epi32, _mm256_loadu_si256, _mm256_movemask_epi8,
        _mm256_mullo_epi32, _mm256_set_epi8, _mm256_set1_epi32, _mm256_shuffle_epi8,
        _mm256_storeu_si256,
    };

    let mut vmin = { _mm256_set1_epi32(i32::MAX) };
    let mut vmax = { _mm256_set1_epi32(0) };

    // Process 8 pixels (32 bytes) at a time
    let chunks = bytes.chunks_exact(32);
    let remainder = chunks.remainder();

    // Shuffle masks: gather each channel byte into the low byte of its 32-bit lane.
    // AVX2 _mm256_shuffle_epi8 operates independently on each 128-bit lane.
    //
    // Within a 128-bit lane, pixel layout is:
    //   byte 0  1  2  3 | 4  5  6  7 | 8  9 10 11 | 12 13 14 15
    //        R0 G0 B0 A0  R1 G1 B1 A1  R2 G2 B2 A2   R3 G3 B3 A3

    let shuf_r = {
        _mm256_set_epi8(
            -1, -1, -1, 12, -1, -1, -1, 8, -1, -1, -1, 4, -1, -1, -1, 0, -1, -1, -1, 12, -1, -1,
            -1, 8, -1, -1, -1, 4, -1, -1, -1, 0,
        )
    };
    let shuf_g = {
        _mm256_set_epi8(
            -1, -1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1, -1, -1, 13, -1, -1,
            -1, 9, -1, -1, -1, 5, -1, -1, -1, 1,
        )
    };
    let shuf_b = {
        _mm256_set_epi8(
            -1, -1, -1, 14, -1, -1, -1, 10, -1, -1, -1, 6, -1, -1, -1, 2, -1, -1, -1, 14, -1, -1,
            -1, 10, -1, -1, -1, 6, -1, -1, -1, 2,
        )
    };
    let shuf_a = {
        _mm256_set_epi8(
            -1, -1, -1, 15, -1, -1, -1, 11, -1, -1, -1, 7, -1, -1, -1, 3, -1, -1, -1, 15, -1, -1,
            -1, 11, -1, -1, -1, 7, -1, -1, -1, 3,
        )
    };

    let w_r = { _mm256_set1_epi32(2126) };
    let w_g = { _mm256_set1_epi32(7152) };
    let w_b = { _mm256_set1_epi32(722) };
    let thresh = { _mm256_set1_epi32(i32::from(ALPHA_THRESHOLD) - 1) };

    for chunk in chunks {
        let raw = unsafe { _mm256_loadu_si256(chunk.as_ptr().cast()) };

        // Gather per-channel values as u32 lanes
        let r = { _mm256_shuffle_epi8(raw, shuf_r) };
        let g = { _mm256_shuffle_epi8(raw, shuf_g) };
        let b = { _mm256_shuffle_epi8(raw, shuf_b) };
        let a = { _mm256_shuffle_epi8(raw, shuf_a) };

        // luma_unscaled = 2126*R + 7152*G + 722*B
        let luma_unscaled = {
            _mm256_add_epi32(
                _mm256_add_epi32(_mm256_mullo_epi32(r, w_r), _mm256_mullo_epi32(g, w_g)),
                _mm256_mullo_epi32(b, w_b),
            )
        };

        // Alpha mask: which pixels are opaque? a > (threshold - 1)
        let opaque_mask = { _mm256_cmpgt_epi32(a, thresh) };
        let mask_bits = { _mm256_movemask_epi8(opaque_mask) };

        if mask_bits == 0 {
            // All 8 pixels are transparent — skip entirely
            continue;
        }

        // AVX2 has no integer divide, so extract and divide scalarly.
        let mut luma_arr = [0i32; 8];
        unsafe { _mm256_storeu_si256(luma_arr.as_mut_ptr().cast(), luma_unscaled) };

        let mut alpha_arr = [0i32; 8];
        unsafe { _mm256_storeu_si256(alpha_arr.as_mut_ptr().cast(), a) };

        for i in 0..8 {
            if alpha_arr[i] >= i32::from(ALPHA_THRESHOLD) {
                let luma = (luma_arr[i].cast_unsigned()) / 10000;
                unsafe { min_max_update_scalar(&mut vmin, &mut vmax, luma, i) };
            }
        }
    }

    // Horizontal reduce vmin / vmax across 8 lanes
    let mut final_min = u32::MAX;
    let mut final_max = u32::MIN;
    let mut min_arr = [0i32; 8];
    let mut max_arr = [0i32; 8];
    unsafe { _mm256_storeu_si256(min_arr.as_mut_ptr().cast(), vmin) };
    unsafe { _mm256_storeu_si256(max_arr.as_mut_ptr().cast(), vmax) };
    for i in 0..8 {
        final_min = final_min.min(min_arr[i].cast_unsigned());
        final_max = final_max.max(max_arr[i].cast_unsigned());
    }

    // Handle remaining pixels
    for pixel in remainder.chunks_exact(4) {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        if a >= ALPHA_THRESHOLD {
            let luma = luma_scalar(r, g, b);
            final_min = final_min.min(luma);
            final_max = final_max.max(luma);
        }
    }

    (final_min, final_max)
}

/// Update the per-lane SIMD min/max accumulators for a single lane.
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[inline]
unsafe fn min_max_update_scalar(
    vmin: &mut std::arch::x86_64::__m256i,
    vmax: &mut std::arch::x86_64::__m256i,
    luma: u32,
    lane: usize,
) {
    use std::arch::x86_64::{_mm256_loadu_si256, _mm256_storeu_si256};

    let mut min_arr = [0i32; 8];
    let mut max_arr = [0i32; 8];
    unsafe { _mm256_storeu_si256(min_arr.as_mut_ptr().cast(), *vmin) };
    unsafe { _mm256_storeu_si256(max_arr.as_mut_ptr().cast(), *vmax) };

    let luma_i32 = luma.cast_signed();
    if luma_i32 < min_arr[lane] {
        min_arr[lane] = luma_i32;
        *vmin = unsafe { _mm256_loadu_si256(min_arr.as_ptr().cast()) };
    }
    if luma_i32 > max_arr[lane] {
        max_arr[lane] = luma_i32;
        *vmax = unsafe { _mm256_loadu_si256(max_arr.as_ptr().cast()) };
    }
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
        let bytes = [
            0, 0, 0, 255, // black
            255, 255, 255, 255, // white
        ];
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, 0);
        assert_eq!(max, 255);
    }

    #[test]
    fn find_range_skips_below_alpha_threshold() {
        let bytes = [
            128, 128, 128, 29, // alpha 29 < 30 — ignored
            100, 100, 100, 255, // opaque
        ];
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        let expected = luma_scalar(100, 100, 100);
        assert_eq!(min, expected);
        assert_eq!(max, expected);
    }

    #[test]
    fn find_range_many_pixels_exercises_simd_path() {
        // 16 pixels = 64 bytes — enough to hit the 32-byte AVX2 chunks
        // and the 16-byte SSE2 chunks with remainder.
        let mut bytes = Vec::with_capacity(64);
        for i in 0u8..16 {
            let v = i * 16; // 0, 16, 32, ... 240
            bytes.extend_from_slice(&[v, v, v, 255]);
        }
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, luma_scalar(0, 0, 0));
        assert_eq!(max, luma_scalar(240, 240, 240));
    }

    #[test]
    fn find_range_all_transparent_many_pixels() {
        // 32 transparent pixels: repeat [100, 200, 50, 0] × 32
        let pixel = [100u8, 200, 50, 0];
        let bytes: Vec<u8> = pixel.iter().copied().cycle().take(4 * 32).collect();
        let (min, max) = find_luma_range_rgba_bytes(&bytes);
        assert_eq!(min, u32::MAX);
        assert_eq!(max, u32::MIN);
    }

    #[test]
    fn scalar_matches_inline_formula() {
        // Verify our scalar function matches the formula used throughout render/mod.rs
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let expected =
                        (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b)) / 10000;
                    assert_eq!(
                        luma_scalar(r, g, b),
                        expected,
                        "mismatch at ({r}, {g}, {b})"
                    );
                }
            }
        }
    }
}

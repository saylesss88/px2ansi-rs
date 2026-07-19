//! Pixel processing — auto-vectorized via LLVM, no explicit SIMD needed.

/// Alpha threshold below which a pixel is considered transparent.
pub const ALPHA_THRESHOLD: u8 = 30;

#[inline]
#[must_use]
/// Compute Rec.709 luma for a single pixel (scalar)
pub fn luma_scalar(r: u8, g: u8, b: u8) -> u32 {
    (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b)) / 10000
}

/// Returns `true` if every pixel in a 32-byte (8-pixel) RGBA chunk is transparent.
/// LLVM will typically vectorize this alpha scan into a single wide comparison + OR reduction.
#[inline]
fn chunk_all_transparent(chunk: &[u8; 32]) -> bool {
    chunk
        .as_chunks::<4>()
        .0
        .iter()
        .all(|&[_, _, _, a]| a < ALPHA_THRESHOLD)
}

/// Scan RGBA bytes and return `(min, max)` luma of opaque pixels.
/// Returns `(u32::MAX, u32::MIN)` if no opaque pixel found.
#[must_use]
pub fn find_luma_range_rgba_bytes(bytes: &[u8]) -> Option<(u32, u32)> {
    let (chunks, remainder) = bytes.as_chunks::<32>();

    let mut min = u32::MAX;
    let mut max = u32::MIN;
    let mut found = false;

    // Process main blocks with your shortcut
    for chunk in chunks {
        if chunk_all_transparent(chunk) {
            continue;
        }
        process_pixels(chunk, &mut min, &mut max, &mut found);
    }
    // Process remainder
    process_pixels(remainder, &mut min, &mut max, &mut found);

    if found { Some((min, max)) } else { None }
}

// Helper keeps the core logic in one place for the auto-vectorizer
#[allow(clippy::inline_always)]
#[inline(always)]
fn process_pixels(data: &[u8], min: &mut u32, max: &mut u32, found: &mut bool) {
    for &[r, g, b, a] in data.as_chunks::<4>().0 {
        if a >= ALPHA_THRESHOLD {
            let luma = luma_scalar(r, g, b);
            *min = (*min).min(luma);
            *max = (*max).max(luma);
            *found = true;
        }
    }
}

/// Compute charset index for each pixel in a 32-byte (8 pixel) RGBA chunk.
/// Returns `(luma_index, is_opaque)` per pixel.
#[must_use]
pub fn compute_charset_indices(
    chunk: &[u8; 32],
    luma_min: u32,
    luma_range: u32,
    num_chars_minus_1: u32,
) -> [(u32, bool); 8] {
    let mut out = [(0u32, false); 8];

    for (i, item) in out.iter_mut().enumerate() {
        let base = i * 4;
        let [r, g, b, a] = [
            chunk[base],
            chunk[base + 1],
            chunk[base + 2],
            chunk[base + 3],
        ];

        let luma = luma_scalar(r, g, b);
        let norm = (luma.saturating_sub(luma_min) * 255) / luma_range;
        let idx = (norm * num_chars_minus_1 / 255).min(num_chars_minus_1);

        *item = (idx, a >= ALPHA_THRESHOLD);
    }
    out
}

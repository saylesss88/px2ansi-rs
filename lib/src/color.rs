//! Color utilities including Oklab-based 256-color quantization.
//!
//! Oklab is a perceptually uniform color space that produces better
//! palette matching than naive RGB distance.

/// Convert sRGB u8 to linear float.
#[inline]
fn srgb_to_linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB to Oklab [L, a, b].
#[inline]
pub fn rgb_to_oklab(red: u8, green: u8, blue: u8) -> [f32; 3] {
    let red = srgb_to_linear(red);
    let green = srgb_to_linear(green);
    let blue = srgb_to_linear(blue);

    let l = 0.412_221_46_f32.mul_add(red, 0.536_332_55_f32.mul_add(green, 0.051_445_995 * blue));
    let m = 0.211_903_5_f32.mul_add(red, 0.680_699_5_f32.mul_add(green, 0.107_396_96 * blue));
    let s = 0.088_302_46_f32.mul_add(red, 0.281_718_85_f32.mul_add(green, 0.629_978_7 * blue));

    let l = l.cbrt();
    let m = m.cbrt();
    let s = s.cbrt();

    [
        0.210_454_26_f32.mul_add(l, 0.793_617_8_f32.mul_add(m, -0.004_072_047 * s)),
        1.977_998_5_f32.mul_add(l, -2.428_592_2_f32.mul_add(m, 0.450_593_7 * s)),
        0.025_904_037_f32.mul_add(l, 0.782_771_77_f32.mul_add(m, -0.808_675_77 * s)),
    ]
}

/// Perceptual distance between two colors in Oklab space.
#[inline]
pub fn oklab_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dl = a[0] - b[0];
    let da = a[1] - b[1];
    let db = a[2] - b[2];
    dl.mul_add(dl, da.mul_add(da, db * db))
}

/// The standard xterm 256-color palette as RGB triples.
/// Colors 0-15 are the system colors, 16-231 are the color cube,
/// 232-255 are the grayscale ramp.
pub static XTERM_256: [[u8; 3]; 256] = generate_xterm_256();

const fn generate_xterm_256() -> [[u8; 3]; 256] {
    let mut palette = [[0u8; 3]; 256];

    // System colors 0-15 (standard terminal colors)
    let system: [[u8; 3]; 16] = [
        [0, 0, 0],       // 0  Black
        [128, 0, 0],     // 1  Maroon
        [0, 128, 0],     // 2  Green
        [128, 128, 0],   // 3  Olive
        [0, 0, 128],     // 4  Navy
        [128, 0, 128],   // 5  Purple
        [0, 128, 128],   // 6  Teal
        [192, 192, 192], // 7  Silver
        [128, 128, 128], // 8  Grey
        [255, 0, 0],     // 9  Red
        [0, 255, 0],     // 10 Lime
        [255, 255, 0],   // 11 Yellow
        [0, 0, 255],     // 12 Blue
        [255, 0, 255],   // 13 Fuchsia
        [0, 255, 255],   // 14 Aqua
        [255, 255, 255], // 15 White
    ];

    let mut i = 0;
    while i < 16 {
        palette[i] = system[i];
        i += 1;
    }

    // 6x6x6 color cube: indices 16-231
    let mut r: u8 = 0;
    while r < 6 {
        let mut g: u8 = 0;
        while g < 6 {
            let mut b: u8 = 0;
            while b < 6 {
                let idx = 16 + 36 * (r as usize) + 6 * (g as usize) + (b as usize);
                palette[idx] = [
                    if r == 0 { 0 } else { 55 + 40 * r },
                    if g == 0 { 0 } else { 55 + 40 * g },
                    if b == 0 { 0 } else { 55 + 40 * b },
                ];
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }

    // Grayscale ramp: indices 232-255
    let mut i: u8 = 0;
    while i < 24 {
        let v = 8 + 10 * i;
        palette[232 + (i as usize)] = [v, v, v];
        i += 1;
    }
    palette
}

/// Find the closest xterm-256 color index for an RGB value using
/// perceptual Oklab distance.
///
/// This is O(256) but fast in practice since it's just float arithmetic.
#[must_use]
pub fn rgb_to_xterm256(r: u8, g: u8, b: u8) -> u8 {
    let target = rgb_to_oklab(r, g, b);
    let mut best_idx = 0u8;
    let mut best_dist = f32::MAX;

    for (i, &[pr, pg, pb]) in XTERM_256.iter().enumerate() {
        let candidate = rgb_to_oklab(pr, pg, pb);
        let dist = oklab_distance(target, candidate);
        if dist < best_dist {
            best_dist = dist;
            best_idx = u8::try_from(i).expect("palette index fits in u8");
        }
    }

    best_idx
}
/// Detects whether the terminal supports 24-bit truecolor.
///
/// Checks `COLORTERM` env var first (most reliable), then falls back
/// to checking `TERM` for known truecolor terminals.
pub fn terminal_supports_truecolor() -> bool {
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        let ct = colorterm.to_lowercase();
        if ct == "truecolor" || ct == "24bit" {
            return true;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        let t = term.to_lowercase();
        if t.contains("256color") || t.contains("truecolor") {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn is_neutral([r, g, b]: [u8; 3]) -> bool {
        let rg = (r as i16 - g as i16).abs();
        let gb = (g as i16 - b as i16).abs();
        let rb = (r as i16 - b as i16).abs();
        rg <= 20 && gb <= 20 && rb <= 20
    }

    fn is_reddish([r, g, b]: [u8; 3]) -> bool {
        r > g.saturating_add(20) && r > b.saturating_add(20)
    }

    fn is_greenish([r, g, b]: [u8; 3]) -> bool {
        g > r.saturating_add(20) && g > b.saturating_add(20)
    }

    fn is_bluish([r, g, b]: [u8; 3]) -> bool {
        b > r.saturating_add(20) && b > g.saturating_add(20)
    }

    fn is_yellowish([r, g, b]: [u8; 3]) -> bool {
        r > 180 && g > 180 && b < 120
    }

    fn is_cyanish([r, g, b]: [u8; 3]) -> bool {
        g > 180 && b > 180 && r < 120
    }

    fn is_magentish([r, g, b]: [u8; 3]) -> bool {
        r > 180 && b > 180 && g < 120
    }

    #[test]
    fn maps_black_to_a_dark_entry() {
        let idx = rgb_to_xterm256(0, 0, 0);
        let [r, g, b] = XTERM_256[idx as usize];
        assert!(r < 40 && g < 40 && b < 40);
    }

    #[test]
    fn maps_white_to_a_light_entry() {
        let idx = rgb_to_xterm256(255, 255, 255);
        let [r, g, b] = XTERM_256[idx as usize];
        assert!(r > 200 && g > 200 && b > 200);
    }

    #[test]
    fn maps_gray_to_a_neutral_entry() {
        let idx = rgb_to_xterm256(128, 128, 128);
        let rgb = XTERM_256[idx as usize];
        assert!(is_neutral(rgb));
    }

    #[test]
    fn maps_red_to_a_reddish_entry() {
        let idx = rgb_to_xterm256(255, 0, 0);
        let rgb = XTERM_256[idx as usize];
        assert!(is_reddish(rgb));
    }

    #[test]
    fn maps_green_to_a_greenish_entry() {
        let idx = rgb_to_xterm256(0, 255, 0);
        let rgb = XTERM_256[idx as usize];
        assert!(is_greenish(rgb));
    }

    #[test]
    fn maps_blue_to_a_bluish_entry() {
        let idx = rgb_to_xterm256(0, 0, 255);
        let rgb = XTERM_256[idx as usize];
        assert!(is_bluish(rgb));
    }

    #[test]
    fn maps_yellow_to_a_yellowish_entry() {
        let idx = rgb_to_xterm256(255, 255, 0);
        let rgb = XTERM_256[idx as usize];
        assert!(is_yellowish(rgb));
    }

    #[test]
    fn maps_cyan_to_a_cyanish_entry() {
        let idx = rgb_to_xterm256(0, 255, 255);
        let rgb = XTERM_256[idx as usize];
        assert!(is_cyanish(rgb));
    }

    #[test]
    fn maps_magenta_to_a_magentish_entry() {
        let idx = rgb_to_xterm256(255, 0, 255);
        let rgb = XTERM_256[idx as usize];
        assert!(is_magentish(rgb));
    }
}

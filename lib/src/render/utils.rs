use super::options::RenderOptions;
use crate::render::CharsetMode;

use image::imageops::FilterType;
use terminal_size::{Height, Width, terminal_size};

impl RenderOptions {
    /// Calculates the optimal target dimensions for the terminal.
    ///
    /// This is the most complex part of the renderer, as it has to account for:
    /// 1. Terminal width/height (auto-detected).
    /// 2. Different character aspect ratios (Braille vs. Half-blocks).
    /// 3. User-defined width overrides.
    /// 4. Nearest-neighbor scaling for pixel art preservation.
    #[must_use]
    pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
        const MAX_SAFE: u32 = 16384;
        let (term_w, term_h) = get_terminal_size();
        let (max_w, max_h) = if term_w > 0 && term_h > 0 {
            match self.charset() {
                CharsetMode::Braille => (term_w * 2, term_h * 4),
                CharsetMode::Unicode if self.style().full => (term_w / 2, term_h),
                CharsetMode::Sixel => (term_w * 8, term_h * 16),
                CharsetMode::Ascii | CharsetMode::Fade => {
                    // Derive height from width using aspect ratio + cell correction
                    // Terminal cells are ~2:1 tall:wide so divide height by 2
                    let w = term_w.saturating_sub(2);
                    let aspect = f64::from(orig_h) / f64::from(orig_w);
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let h = (f64::from(w) * aspect / 2.0).ceil() as u32;
                    (w, h.min(term_h))
                }
                _ => (term_w.saturating_sub(2), term_h * 2 / 3),
            }
        } else {
            (80, 40)
        };

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (render_w, render_h) = self.width().map_or_else(
            || {
                // ASCII/Fade: derive from terminal width, not min-scale
                if matches!(self.charset(), CharsetMode::Ascii | CharsetMode::Fade) {
                    let w = max_w;
                    let aspect = f64::from(orig_h) / f64::from(orig_w);
                    let h = (f64::from(w) * aspect / 2.0).ceil() as u32;
                    return (w, h.min(max_h));
                }
                if self.filter() == FilterType::Nearest && orig_w < 120 {
                    let scale_w = (f64::from(max_w) / f64::from(orig_w)).floor();
                    let scale_h = (f64::from(max_h) / f64::from(orig_h)).floor();
                    let scale = scale_w.min(scale_h).max(1.0);
                    (
                        (f64::from(orig_w) * scale) as u32,
                        (f64::from(orig_h) * scale) as u32,
                    )
                } else {
                    let scale = (f64::from(max_w) / f64::from(orig_w))
                        .min(f64::from(max_h) / f64::from(orig_h));
                    (
                        (f64::from(orig_w) * scale).round() as u32,
                        (f64::from(orig_h) * scale).round() as u32,
                    )
                }
            },
            |tw| {
                let aspect = f64::from(orig_h) / f64::from(orig_w);
                let h = (f64::from(tw) * aspect / 2.0).ceil() as u32;
                (tw, h)
            },
        );
        let result = (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE));

        eprintln!(
            "DEBUG charset={:?} term={}x{} max={}x{} orig={}x{} render={}x{}",
            self.charset(),
            term_w,
            term_h,
            max_w,
            max_h,
            orig_w,
            orig_h,
            result.0,
            result.1
        );

        result
    }
}
/// Use Env vars to get the terminal size
#[must_use]
pub fn get_terminal_size() -> (u32, u32) {
    let ts = terminal_size();
    let env_cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());
    let env_rows = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());

    if let Some((Width(w), Height(h))) = ts {
        return (u32::from(w), u32::from(h));
    }
    if let (Some(c), Some(r)) = (env_cols, env_rows) {
        return (c, r);
    }
    (80, 24)
}

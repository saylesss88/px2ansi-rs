use super::options::RenderOptions;
use crate::render::CharsetMode;

use terminal_size::{Height, Width, terminal_size};

impl RenderOptions {
    /// Calculates the optimal target dimensions for the terminal.
    #[must_use]
    pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
        const MAX_SAFE: u32 = 16_384;

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Scaling positive terminal dimensions by aspect ratios produces positive values within u32 range; MAX_SAFE further bounds results"
        )]
        fn fit_preserving_aspect(
            orig_w: u32,
            orig_h: u32,
            max_w: u32,
            max_h: u32,
            cell_w: f64,
            cell_h: f64,
        ) -> (u32, u32) {
            let img_aspect = f64::from(orig_h) / f64::from(orig_w);
            let cell_aspect = cell_h / cell_w;

            let h_from_w = (f64::from(max_w) * img_aspect * cell_aspect).ceil() as u32;
            if h_from_w <= max_h {
                (max_w.max(1), h_from_w.max(1))
            } else {
                let w_from_h = (f64::from(max_h) / (img_aspect * cell_aspect)).floor() as u32;
                (w_from_h.max(1), max_h.max(1))
            }
        }

        let (term_w, term_h) = get_terminal_size();

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Terminal scaling math uses bounded conversions"
        )]
        let (render_w, render_h) = self.width().map_or_else(
            || match self.charset() {
                CharsetMode::Ansi => fit_preserving_aspect(
                    orig_w,
                    orig_h,
                    term_w.saturating_sub(2),
                    term_h * 2,
                    1.0,
                    1.0,
                ),

                CharsetMode::Unicode => {
                    if self.style().full {
                        fit_preserving_aspect(orig_w, orig_h, term_w / 2, term_h, 1.0, 1.0)
                    } else {
                        fit_preserving_aspect(
                            orig_w,
                            orig_h,
                            term_w.saturating_sub(2),
                            term_h * 2,
                            1.0,
                            1.0,
                        )
                    }
                }
                CharsetMode::Braille => {
                    fit_preserving_aspect(orig_w, orig_h, term_w * 2, term_h * 4, 1.0, 1.0)
                }
                CharsetMode::Sixel => (orig_w, orig_h),
                CharsetMode::Ascii
                | CharsetMode::Fade
                | CharsetMode::Kanji
                | CharsetMode::Chinese => {
                    let w = term_w.saturating_sub(2);
                    let aspect = f64::from(orig_h) / f64::from(orig_w);
                    let h_from_w = (f64::from(w) * aspect / 2.0).ceil() as u32;
                    if h_from_w <= term_h {
                        (w, h_from_w.max(1))
                    } else {
                        let w_from_h = ((f64::from(term_h) * 2.0) / aspect).floor() as u32;
                        (w_from_h.max(1), term_h)
                    }
                }
            },
            |tw| {
                let h = fit_preserving_aspect(orig_w, orig_h, tw, u32::MAX, 1.0, 2.0).1;
                (tw.max(1), h.max(1))
            },
        );
        (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE))
        // let result = (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE));

        // eprintln!(
        //     "DEBUG charset={:?} term={}x{} orig={}x{} render={}x{}",
        //     self.charset(),
        //     term_w,
        //     term_h,
        //     orig_w,
        //     orig_h,
        //     result.0,
        //     result.1
        // );

        // result
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

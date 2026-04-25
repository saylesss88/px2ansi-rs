use super::options::RenderOptions;
use crate::render::CharsetMode;

use image::imageops::FilterType;
use terminal_size::{terminal_size, Height, Width};

impl RenderOptions {
    /// Calculates the optimal target dimensions for the terminal.
    #[must_use]
    pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
        const MAX_SAFE: u32 = 16_384;

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

        fn mode_cell_dims(mode: CharsetMode, unicode_full: bool, wide: bool) -> (f64, f64) {
            match mode {
                CharsetMode::Braille => (2.0, 4.0),
                CharsetMode::Unicode if unicode_full => (2.0, 1.0),
                CharsetMode::Sixel => (8.0, 16.0),
                _ if wide => (2.0, 1.0),
                CharsetMode::Ansi
                | CharsetMode::Fade
                | CharsetMode::Kanji
                | CharsetMode::Chinese
                | CharsetMode::Ascii => (1.0, 1.0),
                _ => (1.0, 1.0),
            }
        }

        let (term_w, term_h) = get_terminal_size();

        let (max_w, max_h) = if term_w > 0 && term_h > 0 {
            match self.charset() {
                CharsetMode::Braille => (term_w * 2, term_h * 4),
                CharsetMode::Unicode if self.style().full => (term_w / 2, term_h),
                CharsetMode::Sixel => (term_w * 8, term_h * 16),
                CharsetMode::Ansi
                | CharsetMode::Kanji
                | CharsetMode::Chinese
                | CharsetMode::Fade => (term_w.saturating_sub(2), term_h * 2),
                _ => (term_w.saturating_sub(2), term_h),
            }
        } else {
            (80, 40)
        };
        let (cell_w, cell_h) = mode_cell_dims(self.charset(), self.style().full, self.style().wide);

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Terminal scaling math uses bounded conversions"
        )]
        let (render_w, render_h) = self.width().map_or_else(
            || match self.charset() {
                CharsetMode::Braille => {
                    fit_preserving_aspect(orig_w, orig_h, term_w * 2, term_h * 4, 1.0, 1.0)
                }
                CharsetMode::Unicode if self.style().full => {
                    fit_preserving_aspect(orig_w, orig_h, term_w / 2, term_h, 1.0, 1.0)
                }
                CharsetMode::Sixel => {
                    fit_preserving_aspect(orig_w, orig_h, term_w * 8, term_h * 16, 1.0, 1.0)
                }
                CharsetMode::Ascii
                | CharsetMode::Fade
                | CharsetMode::Kanji
                | CharsetMode::Chinese
                | CharsetMode::Ansi => {
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
                _ => fit_preserving_aspect(
                    orig_w,
                    orig_h,
                    term_w.saturating_sub(2),
                    term_h * 2 / 3,
                    1.0,
                    1.0,
                ),
            },
            |tw| {
                let aspect = f64::from(orig_h) / f64::from(orig_w);
                let h = (f64::from(tw) * aspect / 2.0).ceil() as u32;
                (tw.max(1), h.max(1))
            },
        );
        // (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE))
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
    // pub fn calculate_dimensions(&self, orig_w: u32, orig_h: u32) -> (u32, u32) {
    //     const MAX_SAFE: u32 = 16384;
    //     let (term_w, term_h) = get_terminal_size();
    //     let (max_w, max_h) = if term_w > 0 && term_h > 0 {
    //         match self.charset() {
    //             CharsetMode::Braille => (term_w * 2, term_h * 4),
    //             CharsetMode::Unicode if self.style().full => (term_w / 2, term_h),
    //             CharsetMode::Sixel => (term_w * 8, term_h * 16),

    //             CharsetMode::Kanji
    //             | CharsetMode::Chinese
    //             | CharsetMode::Ascii
    //             | CharsetMode::Fade => {
    //                 #[expect(
    //                     clippy::cast_possible_truncation,
    //                     clippy::cast_sign_loss,
    //                     reason = "Terminal dimension math involves safe float-to-u32 conversions"
    //                 )]
    //                 {
    //                     let w = term_w.saturating_sub(2);
    //                     let aspect = f64::from(orig_h) / f64::from(orig_w);

    //                     let h_from_w = (f64::from(w) * aspect / 2.0).ceil() as u32;

    //                     if h_from_w <= term_h {
    //                         (w, h_from_w)
    //                     } else {
    //                         let w_from_h = ((f64::from(term_h) * 2.0) / aspect).floor() as u32;
    //                         let chosen_w = w_from_h.min(w);
    //                         (chosen_w, term_h)
    //                     }
    //                 }
    //             }
    //             _ => (term_w.saturating_sub(2), term_h * 2 / 3),
    //         }
    //     } else {
    //         (80, 40)
    //     };

    //     #[expect(
    //         clippy::cast_possible_truncation,
    //         clippy::cast_sign_loss,
    //         reason = "Terminal scaling math involves intentional truncation of pixel coordinates"
    //     )]
    //     let (render_w, render_h) = self.width().map_or_else(
    //         || {
    //             if matches!(
    //                 self.charset(),
    //                 CharsetMode::Kanji
    //                     | CharsetMode::Chinese
    //                     | CharsetMode::Ascii
    //                     | CharsetMode::Fade
    //             ) {
    //                 let w = max_w;
    //                 let aspect = f64::from(orig_h) / f64::from(orig_w);
    //                 let h_from_w = (f64::from(w) * aspect / 2.0).ceil() as u32;
    //                 if h_from_w <= max_h {
    //                     (w, h_from_w.min(max_h))
    //                 } else {
    //                     let w_from_h = ((f64::from(max_h) * 2.0) / aspect).floor() as u32;
    //                     let chosen_w = w_from_h.min(w);
    //                     (chosen_w, max_h)
    //                 }
    //             } else if self.filter() == FilterType::Nearest && orig_w < 120 {
    //                 let scale_w = (f64::from(max_w) / f64::from(orig_w)).floor();
    //                 let scale_h = (f64::from(max_h) / f64::from(orig_h)).floor();
    //                 let scale = scale_w.min(scale_h).max(1.0);
    //                 (
    //                     (f64::from(orig_w) * scale) as u32,
    //                     (f64::from(orig_h) * scale) as u32,
    //                 )
    //             } else {
    //                 let scale = (f64::from(max_w) / f64::from(orig_w))
    //                     .min(f64::from(max_h) / f64::from(orig_h));
    //                 (
    //                     (f64::from(orig_w) * scale).round() as u32,
    //                     (f64::from(orig_h) * scale).round() as u32,
    //                 )
    //             }
    //         },
    //         |tw| {
    //             // User provided width 'tw' derive height from it.
    //             let aspect = f64::from(orig_h) / f64::from(orig_w);
    //             let h = (f64::from(tw) * aspect / 2.0).ceil() as u32;
    //             (tw, h)
    //         },
    //     );

    //     (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE))
    //     // let result = (render_w.clamp(1, MAX_SAFE), render_h.clamp(1, MAX_SAFE));

    //     // eprintln!(
    //     //     "DEBUG charset={:?} term={}x{} max={}x{} orig={}x{} render={}x{}",
    //     //     self.charset(),
    //     //     term_w,
    //     //     term_h,
    //     //     max_w,
    //     //     max_h,
    //     //     orig_w,
    //     //     orig_h,
    //     //     result.0,
    //     //     result.1
    //     // );

    //     // result
    // }
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

//! Image rotation helpers for the `--rotate`, `--axis`, and `--unidirectional` flags.
//!
//! Three axes are supported:
//! - [`RotateAxis::Z`]: Flat canvas spin (0° → 90° → 180° → 270°). Default.
//! - [`RotateAxis::Y`]: Coin-flip illusion on the vertical axis (squish + h-flip).
//! - [`RotateAxis::X`]: Cartwheel illusion on the horizontal axis (squish + v-flip).
//!
//! Two spin modes:
//! - Default (ping-pong): front → back → front, reversing each revolution.
//! - `--unidirectional`: always spins the same way using all 4 phases.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use anyhow::Result;
use image::{DynamicImage, imageops};
use px2ansi::RenderOptions;
use std::{io::Write, thread, time::Duration};

/// How the image is oriented during a `--rotate` spin animation.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum RotateAxis {
    /// Spin flat on the canvas plane — 0° → 90° → 180° → 270°. (default)
    Z,
    /// Coin-flip illusion on the vertical axis: the image squishes and
    /// reveals its horizontal mirror as the "back face".
    Y,
    /// Cartwheel illusion on the horizontal axis: the image squishes and
    /// reveals its vertical mirror as the "back face".
    X,
}

/// How the `--rotate` flag should behave.
#[derive(Debug, Clone)]
pub enum RotateMode {
    /// Animate a continuous spin at the given frames-per-second.
    Spin {
        fps: u8,
        axis: RotateAxis,
        /// If true, always spins the same direction (4-phase unidirectional).
        /// If false (default), ping-pongs: front → back → front.
        unidirectional: bool,
    },
    /// Rotate once by the given angle (90, 180, or 270 degrees).
    Static(u16),
}

/// Parses `--rotate` + `--fps` + `--axis` + `--unidirectional` into a [`RotateMode`].
///
/// * `angle = None`    → no rotation
/// * `angle = Some(0)` → spin mode (sentinel set by `default_missing_value`)
/// * `angle = Some(n)` → static rotation by `n` degrees
///
/// # Errors
///
/// Returns an error if `angle` is not one of `0`, `90`, `180`, or `270`.
pub fn parse_rotate(
    angle: Option<u16>,
    fps: u8,
    axis: RotateAxis,
    unidirectional: bool,
) -> Result<Option<RotateMode>> {
    match angle {
        None => Ok(None),
        Some(0) => Ok(Some(RotateMode::Spin {
            fps,
            axis,
            unidirectional,
        })),
        Some(n @ (90 | 180 | 270)) => Ok(Some(RotateMode::Static(n))),

        Some(other) => anyhow::bail!(
            "Invalid --rotate value: {other}. Valid values are 90, 180, or 270. \
             Omit a value entirely to animate a full 360° spin."
        ),
    }
}

/// Applies a static rotation to `img`.
#[must_use]
pub fn apply_static(img: DynamicImage, degrees: u16) -> DynamicImage {
    match degrees {
        90 => DynamicImage::ImageRgba8(imageops::rotate90(&img.to_rgba8())),
        180 => DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8())),
        270 => DynamicImage::ImageRgba8(imageops::rotate270(&img.to_rgba8())),
        _ => img,
    }
}

// ── Frame generators ──────────────────────────────────────────────────────────

/// Number of interpolation steps per quarter-turn.
/// 8 steps × 4 quarters = 32 frames per full revolution.
const STEPS_PER_QUARTER: u32 = 8;

/// Generates frames for a Z-axis canvas spin.
/// Each of the 4 poses is held for `STEPS_PER_QUARTER` ticks so total frame
/// count matches the illusion axes (32), keeping `--fps` consistent.
fn generate_zaxis_frames(img: &DynamicImage) -> Vec<DynamicImage> {
    let r0 = img.clone();
    let r90 = DynamicImage::ImageRgba8(imageops::rotate90(&img.to_rgba8()));
    let r180 = DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8()));
    let r270 = DynamicImage::ImageRgba8(imageops::rotate270(&img.to_rgba8()));

    let mut frames = Vec::with_capacity(4 * STEPS_PER_QUARTER as usize);
    for pose in [r0, r90, r180, r270] {
        for _ in 0..STEPS_PER_QUARTER {
            frames.push(pose.clone());
        }
    }
    frames
}

/// Generates frames for a ping-pong Y or X axis spin.
///
/// Goes front → back → front, reversing each revolution. 2 phases, ~16 frames.
fn generate_pingpong_frames(img: &DynamicImage, axis: RotateAxis) -> Vec<DynamicImage> {
    let (w, h) = (img.width(), img.height());
    let back = match axis {
        RotateAxis::Y => DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8())),
        RotateAxis::X => DynamicImage::ImageRgba8(imageops::flip_vertical(&img.to_rgba8())),
        RotateAxis::Z => return generate_zaxis_frames(img),
    };

    let mut frames = Vec::with_capacity((2 * STEPS_PER_QUARTER) as usize);

    // Phase 1 — front squishes to edge
    for i in (1..=STEPS_PER_QUARTER).rev() {
        frames.push(apply_squish(img, i, axis, w, h));
    }
    // Phase 2 — back expands from edge to full
    for i in 1..=STEPS_PER_QUARTER {
        frames.push(apply_squish(&back, i, axis, w, h));
    }
    // Reverse: back squishes, front returns (ping-pong return trip)
    for i in (1..=STEPS_PER_QUARTER).rev() {
        frames.push(apply_squish(&back, i, axis, w, h));
    }
    for i in 1..=STEPS_PER_QUARTER {
        frames.push(apply_squish(img, i, axis, w, h));
    }

    frames
}

/// Generates frames for a unidirectional Y or X axis spin.
///
/// Always rotates the same way — 4 phases, 32 frames per revolution:
///
/// | Phase | Face  | Direction     |
/// |-------|-------|---------------|
/// | 1     | Front | full → edge   |
/// | 2     | Back  | edge → full   |
/// | 3     | Back  | full → edge   |
/// | 4     | Front | edge → full   |
///
/// The loop from phase 4 back to phase 1 is seamless — front is always
/// at the same width at the boundary, so there is no visible jump.
fn generate_unidirectional_frames(img: &DynamicImage, axis: RotateAxis) -> Vec<DynamicImage> {
    let (w, h) = (img.width(), img.height());

    let front = img.clone();
    let back = match axis {
        RotateAxis::Y => DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8())),
        RotateAxis::X => DynamicImage::ImageRgba8(imageops::flip_vertical(&img.to_rgba8())),
        RotateAxis::Z => return generate_zaxis_frames(img),
    };

    let mut frames = Vec::with_capacity((4 * STEPS_PER_QUARTER) as usize);

    for quarter in 0..4 {
        let is_front = quarter % 2 == 0;
        let source = if is_front { &front } else { &back };

        for step in (1..=STEPS_PER_QUARTER).rev() {
            frames.push(apply_squish(source, step, axis, w, h));
        }
        for step in 1..=STEPS_PER_QUARTER {
            frames.push(apply_squish(source, step, axis, w, h));
        }
    }

    frames
}
// fn generate_unidirectional_frames(img: &DynamicImage, axis: RotateAxis) -> Vec<DynamicImage> {
//     let (w, h) = (img.width(), img.height());
//     let back = match axis {
//         RotateAxis::Y => DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8())),
//         RotateAxis::X => DynamicImage::ImageRgba8(imageops::flip_vertical(&img.to_rgba8())),
//         RotateAxis::Z => return generate_zaxis_frames(img),
//     };

//     let mut frames = Vec::with_capacity((4 * STEPS_PER_QUARTER) as usize);

//     // Phase 1: Front face squishes to edge (full → 0)
//     for i in (1..=STEPS_PER_QUARTER).rev() {
//         frames.push(apply_squish(img, i, axis, w, h));
//     }
//     // Phase 2: Back face expands from edge (0 → full)
//     for i in 1..=STEPS_PER_QUARTER {
//         frames.push(apply_squish(&back, i, axis, w, h));
//     }
//     // Phase 3: Back face squishes to edge (full → 0)
//     for i in (1..=STEPS_PER_QUARTER).rev() {
//         frames.push(apply_squish(&back, i, axis, w, h));
//     }
//     // Phase 4: Front face expands from edge (0 → full)
//     for i in 1..=STEPS_PER_QUARTER {
//         frames.push(apply_squish(img, i, axis, w, h));
//     }

//     frames
// }

// ── Squish + padding helpers ──────────────────────────────────────────────────

/// Resizes `img` for one step of a squish animation along `axis`,
/// then pads it back to the original bounding box so it stays centered.
///
/// `step` ranges from `1..=STEPS_PER_QUARTER`:
/// - `STEPS_PER_QUARTER` → full size
/// - `1`                 → almost gone (1 px thin)
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "fairly safe"
)]
fn apply_squish(img: &DynamicImage, step: u32, axis: RotateAxis, w: u32, h: u32) -> DynamicImage {
    match axis {
        RotateAxis::Y => {
            let ratio = f64::from(step) / f64::from(STEPS_PER_QUARTER);
            let new_w = (f64::from(w) * ratio).round() as u32;
            let new_w = new_w.max(1);

            pad_to_width(img.resize_exact(new_w, h, imageops::FilterType::Nearest), w)
        }
        RotateAxis::X => {
            let ratio = f64::from(step) / f64::from(STEPS_PER_QUARTER);
            let new_h = (f64::from(h) * ratio).round() as u32;
            let new_h = new_h.max(1);

            pad_to_height(img.resize_exact(w, new_h, imageops::FilterType::Nearest), h)
        }
        RotateAxis::Z => img.clone(),
    }
}

/// Centers `img` horizontally inside a transparent canvas of `target_w` width.
fn pad_to_width(img: DynamicImage, target_w: u32) -> DynamicImage {
    let h = img.height();
    let current_w = img.width();
    if current_w >= target_w {
        return img;
    }
    let mut canvas = image::RgbaImage::new(target_w, h);
    let offset_x = i64::from((target_w - current_w) / 2);
    imageops::overlay(&mut canvas, &img.to_rgba8(), offset_x, 0);
    DynamicImage::ImageRgba8(canvas)
}

/// Centers `img` vertically inside a transparent canvas of `target_h` height.
fn pad_to_height(img: DynamicImage, target_h: u32) -> DynamicImage {
    let w = img.width();
    let current_h = img.height();
    if current_h >= target_h {
        return img;
    }
    let mut canvas = image::RgbaImage::new(w, target_h);
    let offset_y = i64::from((target_h - current_h) / 2);
    imageops::overlay(&mut canvas, &img.to_rgba8(), 0, offset_y);
    DynamicImage::ImageRgba8(canvas)
}

// ── Spin loop ─────────────────────────────────────────────────────────────────

/// Renders a continuous spin loop to `writer`.
///
/// Pre-renders all frames once to ANSI byte buffers, then replays them in a
/// loop until interrupted (Ctrl-C). Each frame is preceded by an ANSI
/// clear-screen + cursor-home sequence so the image appears to rotate in place.
///
/// # Errors
///
/// Returns an error if any write or render call fails.
pub fn run_spin_loop<W: Write>(
    img: &DynamicImage,
    render: &RenderOptions,
    fps: u8,
    axis: RotateAxis,
    unidirectional: bool,
    writer: &mut W,
) -> Result<()> {
    const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
    // const SHOW_CURSOR: &[u8] = b"\x1b[?25h";
    const GOTO_HOME: &[u8] = b"\x1b[H";
    const CLEAR_SCREEN: &[u8] = b"\x1b[2J";
    // const CLEAR: &[u8] = b"\x1b[2J\x1b[H";
    let delay = Duration::from_millis(1000 / u64::from(fps.max(1)));

    let frames = match axis {
        RotateAxis::Z => generate_zaxis_frames(img),
        axis if unidirectional => generate_unidirectional_frames(img, axis),
        axis => generate_pingpong_frames(img, axis),
    };

    // Pre-render all frames to ANSI byte buffers.
    // With the `parallel` feature, all frames are rendered concurrently
    // on a typical machine this cuts the startup delay from ~32x single-frame
    // time down to ~(32 / num_cpus)x.
    let buffers: Vec<Vec<u8>> = {
        #[cfg(feature = "parallel")]
        {
            frames
                .par_iter()
                .map(|frame| {
                    let mut buf =
                        Vec::with_capacity(frame.width() as usize * frame.height() as usize * 2);
                    // render_centered takes &mut W — we buffer into a Vec per frame
                    // then collect in order, so output is deterministic.
                    render.render_centered(frame, &mut buf).map(|()| buf)
                })
                .collect::<std::result::Result<Vec<_>, _>>()?
        }
        #[cfg(not(feature = "parallel"))]
        {
            let mut bufs = Vec::with_capacity(frames.len());
            for frame in &frames {
                let mut buf =
                    Vec::with_capacity(frame.width() as usize * frame.height() as usize * 2);
                render.render_centered(frame, &mut buf)?;
                bufs.push(buf);
            }
            bufs
        }
    };

    // 2. Initial Clear
    writer.write_all(CLEAR_SCREEN)?;
    writer.write_all(HIDE_CURSOR)?;

    loop {
        for buf in &buffers {
            // 3. Move cursor to top
            writer.write_all(GOTO_HOME)?;
            writer.write_all(buf)?;

            // 4. Force everything to the terminal
            writer.flush()?;
            thread::sleep(delay);
        }
    }
    // writer.write_all(SHOW_CURSOR)?;
}

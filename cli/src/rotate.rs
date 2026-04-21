//! Image rotation helpers for the `--rotate` and `--axis` flags.
//!
//! Three axes are supported:
//! - [`RotateAxis::Z`]: Flat canvas spin (0° → 90° → 180° → 270°). Default.
//! - [`RotateAxis::Y`]: Coin-flip illusion on the vertical axis (squish + h-flip).
//! - [`RotateAxis::X`]: Cartwheel illusion on the horizontal axis (squish + v-flip).

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
    /// Animate a continuous 360° spin at the given frames-per-second.
    Spin { fps: u8, axis: RotateAxis },
    /// Rotate once by the given angle (90, 180, or 270 degrees).
    Static(u16),
}

/// Parses `--rotate` + `--fps` + `--axis` into a [`RotateMode`], if rotation was requested.
///
/// * `angle = None`    → no rotation at all
/// * `angle = Some(0)` → spin mode (sentinel set by `default_missing_value`)
/// * `angle = Some(n)` → static rotation by `n` degrees
///
/// # Errors
///
/// Returns an error if `angle` is not one of `0`, `90`, `180`, or `270`.
pub fn parse_rotate(angle: Option<u16>, fps: u8, axis: RotateAxis) -> Result<Option<RotateMode>> {
    match angle {
        None => Ok(None),
        Some(0) => Ok(Some(RotateMode::Spin { fps, axis })),
        Some(90) | Some(180) | Some(270) => Ok(Some(RotateMode::Static(angle.unwrap()))),
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

// ── Frame generators ─────────────────────────────────────────────────────────

/// Number of interpolation steps per quarter-turn for the illusion axes.
/// 8 steps × 4 quarters = 32 frames per full revolution.
const STEPS_PER_QUARTER: u32 = 8;

/// Generates 32 frames for a Z-axis canvas spin (four 90° hard steps, each
/// held for `STEPS_PER_QUARTER` ticks so timing matches the illusion axes).
fn generate_zaxis_frames(img: &DynamicImage) -> Vec<DynamicImage> {
    let r0 = img.clone();
    let r90 = DynamicImage::ImageRgba8(imageops::rotate90(&img.to_rgba8()));
    let r180 = DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8()));
    let r270 = DynamicImage::ImageRgba8(imageops::rotate270(&img.to_rgba8()));

    // Repeat each pose for STEPS_PER_QUARTER frames so the total frame count
    // equals the illusion axes (32), giving consistent --fps behaviour.
    let mut frames = Vec::with_capacity(4 * STEPS_PER_QUARTER as usize);
    for pose in [r0, r90, r180, r270] {
        for _ in 0..STEPS_PER_QUARTER {
            frames.push(pose.clone());
        }
    }
    frames
}

/// Generates 32 frames for a Y-axis coin-flip illusion.
///
/// The image squishes horizontally to zero (front → edge), then expands as its
/// horizontal mirror (edge → back), and then reverses back to front.
fn generate_yaxis_frames(img: &DynamicImage) -> Vec<DynamicImage> {
    let w = img.width();
    let h = img.height();
    let back = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img.to_rgba8()));

    let mut frames = Vec::with_capacity((4 * STEPS_PER_QUARTER) as usize);

    // Phase 1 — front squishes away (full → 0)
    for i in (0..=STEPS_PER_QUARTER).rev() {
        let new_w = (w as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_w == 0 {
            continue;
        }
        let resized = img.resize_exact(new_w, h, imageops::FilterType::Nearest);
        frames.push(pad_to_width(resized, w));
    }

    // Phase 2 — back expands into view (0 → full)
    for i in 1..=STEPS_PER_QUARTER {
        let new_w = (w as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_w == 0 {
            continue;
        }
        let resized = back.resize_exact(new_w, h, imageops::FilterType::Nearest);
        frames.push(pad_to_width(resized, w));
    }

    // Phase 3 — back squishes away (full → 0)
    for i in (0..=STEPS_PER_QUARTER).rev() {
        let new_w = (w as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_w == 0 {
            continue;
        }
        let resized = back.resize_exact(new_w, h, imageops::FilterType::Nearest);
        frames.push(pad_to_width(resized, w));
    }

    // Phase 4 — front expands back into view (0 → full)
    for i in 1..=STEPS_PER_QUARTER {
        let new_w = (w as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_w == 0 {
            continue;
        }
        let resized = img.resize_exact(new_w, h, imageops::FilterType::Nearest);
        frames.push(pad_to_width(resized, w));
    }

    frames
}

/// Generates 32 frames for an X-axis cartwheel illusion.
///
/// Same as Y-axis but squishes vertically and flips the image top-to-bottom.
fn generate_xaxis_frames(img: &DynamicImage) -> Vec<DynamicImage> {
    let w = img.width();
    let h = img.height();
    let back = DynamicImage::ImageRgba8(imageops::flip_vertical(&img.to_rgba8()));

    let mut frames = Vec::with_capacity((4 * STEPS_PER_QUARTER) as usize);

    // Phase 1 — front squishes away (full → 0)
    for i in (0..=STEPS_PER_QUARTER).rev() {
        let new_h = (h as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_h == 0 {
            continue;
        }
        let resized = img.resize_exact(w, new_h, imageops::FilterType::Nearest);
        frames.push(pad_to_height(resized, h));
    }

    // Phase 2 — back expands into view (0 → full)
    for i in 1..=STEPS_PER_QUARTER {
        let new_h = (h as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_h == 0 {
            continue;
        }
        let resized = back.resize_exact(w, new_h, imageops::FilterType::Nearest);
        frames.push(pad_to_height(resized, h));
    }

    // Phase 3 — back squishes away (full → 0)
    for i in (0..=STEPS_PER_QUARTER).rev() {
        let new_h = (h as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_h == 0 {
            continue;
        }
        let resized = back.resize_exact(w, new_h, imageops::FilterType::Nearest);
        frames.push(pad_to_height(resized, h));
    }

    // Phase 4 — front expands back (0 → full)
    for i in 1..=STEPS_PER_QUARTER {
        let new_h = (h as f32 * i as f32 / STEPS_PER_QUARTER as f32) as u32;
        if new_h == 0 {
            continue;
        }
        let resized = img.resize_exact(w, new_h, imageops::FilterType::Nearest);
        frames.push(pad_to_height(resized, h));
    }

    frames
}

// ── Padding helpers ───────────────────────────────────────────────────────────

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

/// Renders a continuous 360° spin loop to `writer`, using the given axis.
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
    writer: &mut W,
) -> Result<()> {
    let delay = Duration::from_millis(1000 / u64::from(fps.max(1)));

    // Build the frame list for the chosen axis.
    let frames = match axis {
        RotateAxis::Z => generate_zaxis_frames(img),
        RotateAxis::Y => generate_yaxis_frames(img),
        RotateAxis::X => generate_xaxis_frames(img),
    };

    // Pre-render every frame to an ANSI byte buffer once.
    let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(frames.len());
    for frame in &frames {
        let mut buf = Vec::with_capacity(frame.width() as usize * frame.height() as usize * 2);
        render.render_centered(frame, &mut buf)?;
        buffers.push(buf);
    }

    const CLEAR: &[u8] = b"\x1b[2J\x1b[H";

    loop {
        for buf in &buffers {
            writer.write_all(CLEAR)?;
            writer.write_all(buf)?;
            writer.flush()?;
            thread::sleep(delay);
        }
    }
}

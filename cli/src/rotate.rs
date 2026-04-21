//! Image rotation helpers for the `--rotate` flag.
//!
//! Provides two modes:
//! - [`RotateMode::Spin`]: Animates a continuous 360° loop.
//! - [`RotateMode::Static`]: Rotates once by 90, 180, or 270 degrees.

use std::{io::Write, thread, time::Duration};

use anyhow::Result;
use image::{DynamicImage, imageops};
use px2ansi::RenderOptions;

/// How the `--rotate` flag should behave.
#[derive(Debug, Clone)]
pub enum RotateMode {
    /// Animate a continuous 360° spin at the given frames-per-second.
    Spin { fps: u8 },
    /// Rotate once by the given angle (90, 180, or 270 degrees).
    Static(u16),
}

/// Parses `--rotate` + `--fps` into a [`RotateMode`], if rotation was requested.
///
/// * `angle = None`    → no rotation at all
/// * `angle = Some(0)` → spin mode (sentinel set by `default_missing_value`)
/// * `angle = Some(n)` → static rotation by `n` degrees
///
/// # Errors
///
/// Returns an error if `angle` is not one of `0`, `90`, `180`, or `270`.
pub fn parse_rotate(angle: Option<u16>, fps: u8) -> Result<Option<RotateMode>> {
    match angle {
        None => Ok(None),
        Some(0) => Ok(Some(RotateMode::Spin { fps })),
        Some(90 | 180 | 270) => Ok(Some(RotateMode::Static(angle.unwrap()))),
        Some(other) => anyhow::bail!(
            "Invalid --rotate value: {other}. Valid values are 90, 180, or 270. \
             Omit a value entirely to animate a full 360° spin."
        ),
    }
}

/// Applies a static rotation to `img`.
///
/// # Panics (never)
///
/// The `degrees` value is already validated by [`parse_rotate`], so only
/// 90 / 180 / 270 reach this function.
#[must_use]
pub fn apply_static(img: DynamicImage, degrees: u16) -> DynamicImage {
    match degrees {
        90 => DynamicImage::ImageRgba8(imageops::rotate90(&img.to_rgba8())),
        180 => DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8())),
        270 => DynamicImage::ImageRgba8(imageops::rotate270(&img.to_rgba8())),
        _ => img, // validated upstream, unreachable in practice
    }
}

/// Renders a continuous 360° spin loop to `writer`.
///
/// Cycles through 0° → 90° → 180° → 270° → 0° … until the process is
/// interrupted (Ctrl-C). Each frame is preceded by an ANSI clear-screen +
/// cursor-home sequence so the image appears to rotate in place.
///
/// # Errors
///
/// Returns an error if any write or render call fails.
pub fn run_spin_loop<W: Write>(
    img: &DynamicImage,
    render: &RenderOptions,
    fps: u8,
    writer: &mut W,
) -> Result<()> {
    // Clear screen + cursor home (ANSI)
    const CLEAR: &[u8] = b"\x1b[2J\x1b[H";
    let delay = Duration::from_millis(1000 / u64::from(fps.max(1)));

    // Pre-build the four rotated frames as DynamicImages.
    let frames: [DynamicImage; 4] = [
        img.clone(),
        DynamicImage::ImageRgba8(imageops::rotate90(&img.to_rgba8())),
        DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8())),
        DynamicImage::ImageRgba8(imageops::rotate270(&img.to_rgba8())),
    ];

    // Pre-render all four frames to ANSI byte buffers once, then just replay them.
    let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(4);
    for frame in &frames {
        let mut buf = Vec::with_capacity(frame.width() as usize * frame.height() as usize * 2);
        render.render_centered(frame, &mut buf)?;
        buffers.push(buf);
    }

    loop {
        for buf in &buffers {
            writer.write_all(CLEAR)?;
            writer.write_all(buf)?;
            writer.flush()?;
            thread::sleep(delay);
        }
    }
}

//! # Main Entry Point
//!
//! The primary way to use this module is through [`write_ansi_art`], which
//! handles the internal rendering state and dispatches the image data
//! to the appropriate strategy based on the provided [`RenderOptions`].

/// ANSI color state tracking and glyph-writing helpers for both
/// serial (deduplicating) and parallel (stateless) render paths.
pub mod color;

/// [`RenderOptions`] and [`RenderOptionsBuilder`]: the primary configuration
/// types for controlling charset, resize filter, color mode, and output width.
pub mod options;

#[cfg(feature = "parallel")]
mod parallel;
mod pixel;
mod renderer;
mod serial;

/// Core rendering types: [`CharsetMode`], [`ColorMode`], [`Density`],
/// [`RenderStyle`], and [`RenderError`].
pub mod types;

/// Terminal size detection and dimension calculation for fitting images
/// to the current terminal viewport.
pub mod utils;

pub use options::*;
pub use renderer::write_ansi_art;
#[cfg(feature = "sixel")]
pub use renderer::write_sixel;
pub use types::*;
pub use utils::*;

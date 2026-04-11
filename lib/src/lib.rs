//! # px2ansi
//!
//! A high-fidelity terminal art engine for rendering images as ANSI terminal art.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use px2ansi::{RenderOptions, RenderStylePreset};
//! use image::DynamicImage;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let img: DynamicImage = unimplemented!();
//! // Build options with a preset, then override specific fields
//! let mut builder = RenderOptions::builder();
//! builder.preset(RenderStylePreset::Braille);
//! builder.width(120);
//! builder.color(true);
//! let opts = builder.build();
//!
//! let mut stdout = std::io::stdout();
//! opts.render_centered(&img, &mut stdout)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Rasterization (requires `rasterize` feature)
//!
//! ```rust,no_run
//! # #[cfg(feature = "rasterize")]
//! # {
//! use px2ansi::{RasterTheme, rasterize_ansi_with_theme};
//!
//! # let ansi_bytes: &[u8] = b"";
//! let img = rasterize_ansi_with_theme(ansi_bytes, RasterTheme::Dracula)?;
//! img.save("output.png")?;
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod cli_enums;
pub mod indexer;
pub mod render;

pub(crate) mod color;
#[cfg(feature = "rasterize")]
pub(crate) mod rasterize;
#[cfg(feature = "rasterize")]
pub(crate) mod themes;

// ── Core re-exports (always available) ──────────────────────────────────────
pub use crate::{
    cli_enums::{RenderStylePreset, ResizeFilter},
    indexer::{ImageEntry, build_index},
    render::{
        CharsetMode, Density, RenderOptions, RenderOptionsBuilder, RenderStyle, write_ansi_art,
    },
};

// ── Rasterization re-exports (feature = "rasterize") ───────���────────────────
#[cfg(feature = "rasterize")]
pub use crate::{
    rasterize::{rasterize_ansi, rasterize_ansi_with_theme},
    themes::RasterTheme,
};

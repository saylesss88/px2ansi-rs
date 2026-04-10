//! # px2ansi
//!
//! A high-fidelity terminal art engine.
//!
//! ## Library Usage
//!
//! The library provides a flexible [`RenderOptionsBuilder`] to configure the output.
//! You can start from a [`RenderStylePreset`] and override specific fields like
//! width or color.
//! ```rust,no_run
//! use px2ansi::{RenderOptions, RenderStylePreset};
//! # use image::DynamicImage;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let img: DynamicImage = unimplemented!();
//!
//! // Option A: The "One-Liner" (Ensure setters return `Self`)
//! let opts = RenderOptions::builder()
//!     .preset(RenderStylePreset::Braille)
//!     .width(120)
//!     .color(true)
//!     .build();
//!
//! // Option B: If your builder uses &mut self, do this:
//! let mut builder = RenderOptions::builder();
//! builder.preset(RenderStylePreset::Braille);
//! builder.width(120);
//! let opts = builder.build();
//!
//! let mut stdout = std::io::stdout();
//! opts.render_centered(&img, &mut stdout)?;
//! # Ok(())
//! # }
//! ```

pub mod cli_enums;
pub mod indexer;
#[cfg(feature = "rasterize")]
pub mod rasterize;
pub mod render;

// re-exports
pub use crate::{
    cli_enums::{RenderStylePreset, ResizeFilter},
    indexer::{ImageEntry, build_index},
    render::{
        CharsetMode, Density, RenderOptions, RenderOptionsBuilder, RenderStyle, write_ansi_art,
    },
};

#[cfg(feature = "rasterize")]
pub use crate::rasterize::rasterize_ansi;

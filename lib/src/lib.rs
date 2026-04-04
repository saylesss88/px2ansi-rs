//! # px2ansi
//!
//! A high-fidelity terminal art engine.
//!
//! ## Library Usage
//!
//! The library provides a flexible [`RenderOptionsBuilder`] to configure the output.
//! You can start from a [`RenderStylePreset`] and override specific fields like
//! width or color.
//!
//! ```rust,no_run
//! use px2ansi::{RenderOptions, RenderStylePreset, write_ansi_art};
//! use image::open;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let img = open("photo.png")?;
//!
//! // Create options starting from a preset
//! let opts = RenderOptions::builder()
//!     .preset(RenderStylePreset::Braille)
//!     .width(120)
//!     .color(true)
//!     .build();
//!
//! let mut stdout = std::io::stdout();
//! opts.render_centered(&img, &mut stdout)?;
//! # Ok(())
//! # }
//! ```

pub mod cli_enums;
pub mod indexer;
pub mod rasterize;
pub mod render;

// re-exports
pub use crate::cli_enums::{RenderStylePreset, ResizeFilter};
pub use crate::rasterize::rasterize_ansi;
pub use crate::render::{
    CharsetMode, Density, RenderOptions, RenderOptionsBuilder, RenderStyle, write_ansi_art,
};

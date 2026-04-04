//! # px2ansi
//!
//! A high-fidelity terminal art engine. Transforms images into terminal-native
//! art using 7 rendering styles: ANSI blocks, Braille, Kanji, Unicode, Fade, ASCII, Chinese.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use px2ansi::{RenderOptions, RenderStylePreset, write_ansi_art};
//! use image::open;
//!
//! let img = open("photo.png").unwrap();
//! let opts = RenderOptions::builder()
//!     .style(Some(RenderStylePreset::Braille))
//!     .width(Some(120))
//!     .build();
//!
//! let prepared = opts.prepare_image(&img);
//! write_ansi_art(&prepared, &mut std::io::stdout(), opts).unwrap();
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

//! # px2ansi-rs
//!
//! A high-fidelity terminal art engine and asset manager designed for
//! performance and variety. `px2ansi-rs` transforms images into terminal-native
//! art using 7 distinct rendering styles, including high-density Braille,
//! Kanji, and traditional ANSI blocks.
//!
//! ## Core Features
//! * **High Performance**: Optimized for a "Build Once, Show Many" workflow via indexing.
//! * **Multiple Styles**: Supports `ansi`, `unicode`, `fade`, `ascii`, `braille`, `full-block`, and `kanji`.
//! * **Rasterization**: Includes a built-in rasterizer to convert ANSI escape sequences
//!   back into PNG images using an embedded Iosevka font.

#![allow(clippy::multiple_crate_versions)]

pub mod cli;
pub mod cli_enums;
pub mod indexer;
pub mod options;
pub mod rasterize;
mod render;

pub use crate::cli::{Cli, Commands};
/// CLI-specific enums for style presets and image resizing filters.
pub use cli_enums::{RenderStylePreset, ResizeFilter};

/// Configuration types for fine-tuning the output, including
/// density, charset modes, and overall render styles.
pub use options::{CharsetMode, Density, RenderOptions, RenderStyle};

/// The primary entry point for turning ANSI byte streams into PNG buffers.
pub use rasterize::rasterize_ansi;

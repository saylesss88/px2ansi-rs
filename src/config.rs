use crate::cli::ResizeFilter;
use serde::{Deserialize, Serialize};

/// Global configuration for the px2ansi engine.
///
/// This struct handles settings persisted in the user's config file (e.g., `default-config.toml`).
/// It follows the "Hierarchy of Truth": CLI flags override these settings, which in turn
/// override the hardcoded defaults.
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct AppConfig {
    /// The rendering mode to use.
    /// Supported values: "ansi" (standard packing) or "unicode" (high-definition symbols).
    pub mode: String,

    /// Whether to display execution timing and performance metadata after rendering.
    pub latency: bool,

    /// The resampling filter used when scaling images to fit the terminal.
    /// `Nearest` is recommended for pixel art, while `Lanczos3` is best for photos.
    pub filter: ResizeFilter,

    /// If true, renders pixels as square blocks (██) instead of vertically packed half-blocks.
    /// This is ideal for a "retro" 1:1 pixel aspect ratio.
    pub full: bool,

    /// The path to the JSON index file containing the image library.
    ///
    /// **Note:** Using an absolute path is recommended so the tool works
    /// regardless of your current working directory.
    pub index: String,
}

impl Default for AppConfig {
    /// Provides the fallback defaults used when no config file is found.
    fn default() -> Self {
        Self {
            mode: "ansi".into(),
            latency: false,
            filter: ResizeFilter::Lanczos3,
            full: false,
            index: "index.json".into(),
        }
    }
}

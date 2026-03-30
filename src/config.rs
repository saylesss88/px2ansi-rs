use serde::{Deserialize, Serialize};

use px2ansi_rs::{RenderStylePreset, ResizeFilter};

/// Global configuration for the px2ansi engine.
///
/// This struct handles settings persisted in the user's config file (e.g., `default-config.toml`).
/// It follows the "Hierarchy of Truth": CLI flags override these settings, which in turn
/// override the hardcoded defaults.
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    /// Whether to display execution timing and performance metadata after rendering.
    pub latency: bool,

    /// The resampling filter used when scaling images to fit the terminal.
    /// `Nearest` is recommended for pixel art, while `Lanczos3` is best for photos.
    pub filter: ResizeFilter,

    /// The path to the JSON index file containing the image library.
    ///
    /// **Note:** Using an absolute path is recommended so the tool works
    /// regardless of your current working directory.
    pub index: String,

    pub style: RenderStylePreset,

    pub output_image: Option<String>,
}

impl Default for Config {
    /// Provides the fallback defaults used when no config file is found.
    fn default() -> Self {
        Self {
            latency: false,
            filter: ResizeFilter::Lanczos3,
            index: "index.json".into(),
            style: RenderStylePreset::Ansi,
            output_image: None,
        }
    }
}

use crate::RenderStylePreset;
use px2ansi::{ColorMode, Density, RenderOptions, ResizeFilter};

/// Constructs a [`RenderOptions`] instance from a set of optional configuration parameters.
///
/// This helper function simplifies the initialization of a renderer by mapping optional
/// CLI-style arguments into a structured builder. If a parameter is `None`, the
/// [`RenderOptions`] defaults (defined in the builder) will be preserved.
///
/// # Arguments
///
/// * `style` - An optional preset that defines the overall aesthetic (e.g., ASCII vs Unicode).
/// * `density` - Character sets used to represent different brightness levels.
/// * `width` - The target width in characters for the rendered output.
/// * `filter` - The resampling algorithm used if the image needs to be resized.
/// * `no_color` - If `true`, explicitly disables ANSI color output in the resulting options.
///
/// # Examples
///
/// ```rust
/// use px2ansi_rs::build_render_options;
/// use px2ansi::RenderStylePreset;
///
/// // Width is passed through
/// let opts = build_render_options(None, None, Some(80), None, None, false);
/// assert_eq!(opts.width(), Some(80));
///
/// // No-op when all None/false
/// let opts = build_render_options(None, None, None, None, None, false);
/// assert_eq!(opts.width(), None);
/// assert!(opts.color_mode(), ColorMode::None); // color is on by default
///
/// // no_color disables color
/// let opts = build_render_options(None, None, None, None, None, true);
/// assert!(!opts.color_mode(), ColorMode::None);
/// ```
#[must_use]
pub fn build_render_options(
    style: Option<RenderStylePreset>,
    density: Option<Density>,
    width: Option<u32>,
    filter: Option<ResizeFilter>,
    color_mode: Option<ColorMode>,
    dither: bool,
) -> RenderOptions {
    let mut builder = RenderOptions::builder();

    if let Some(s) = style {
        builder = builder.preset(s);
    }
    if let Some(d) = density {
        builder = builder.density(d);
    }
    if let Some(w) = width {
        builder = builder.width(w);
    }
    if let Some(f) = filter {
        builder = builder.filter(f);
    }

    // Move this OUT of the filter else-block so it always runs
    if let Some(mode) = color_mode {
        builder = builder.color_mode(mode);
    }

    // Always apply the dither flag and then build
    builder.dither(dither).build()
}

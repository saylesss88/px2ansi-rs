use crate::RenderStylePreset;
use px2ansi::{Density, RenderOptions, ResizeFilter};

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
/// ```
/// let options = build_render_options(None, None, Some(80), None, false);
/// assert_eq!(options.width, Some(80));
/// ```
#[must_use]
pub fn build_render_options(
    style: Option<RenderStylePreset>,
    density: Option<Density>,
    width: Option<u32>,
    filter: Option<ResizeFilter>,
    no_color: bool,
) -> RenderOptions {
    let mut builder = RenderOptions::builder();

    if let Some(s) = style {
        builder.preset(s);
    }
    if let Some(d) = density {
        builder.density(d);
    }
    if let Some(w) = width {
        builder.width(w);
    }
    if let Some(f) = filter {
        builder.filter(f);
    }
    if no_color {
        builder.color(false);
    }

    builder.build()
}

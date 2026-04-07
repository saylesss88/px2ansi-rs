use crate::RenderStylePreset;
use px2ansi::{Density, RenderOptions, ResizeFilter};

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

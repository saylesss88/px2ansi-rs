# px2ansi

`px2ansi` is a high-fidelity terminal art engine for Rust. It transforms images
into terminal-native art using multiple rendering styles, including ANSI blocks,
Unicode half-blocks, Braille, Fade, ASCII, Chinese, and Kanji.

It is designed as the reusable core behind `px2ansi-rs`, but it can also be used
directly as a library in your own projects.

**Features**

- Multiple rendering styles: `ansi`, `unicode`, `braille`, `fade`, `ascii`,
  `chinese`, `kanji`.

- Configurable resize filters.

- Automatic terminal-friendly dimension calculation.

- ANSI art rendering to any `Write` target.

- Rasterization support for converting ANSI output back into PNG.

**Installation**

Add `px2ansi` to your `Cargo.toml`:

```toml
[dependencies]
px2ansi = "0.3.18"
image = "0.25"
```

If you only want the core engine and already have `image` in your project, just
depend on `px2ansi` and reuse your existing image setup.

**Quick Start**

```rs
use image::open;
use px2ansi::{RenderOptions, RenderStylePreset, write_ansi_art};

fn main() -> anyhow::Result<()> {
    let img = open("photo.png")?;

    let opts = RenderOptions::builder()
        .style(Some(RenderStylePreset::Braille))
        .width(Some(120))
        .build();

    let prepared = opts.prepare_image(&img);

    let mut out = std::io::stdout();
    write_ansi_art(&prepared, &mut out, opts)?;

    Ok(())
}
```

**Core Types**

`RenderOptions`

The main configuration object for rendering. It controls:

- target width,

- resize filter,

- charset mode,

- density,

- color output.

---

`RenderStylePreset`

A convenience enum for quickly choosing a style preset such as:

- `Ansi`

- `Unicode`

- `Braille`

- `Fade`

- `Ascii`

- `Kanji`

- `Chinese`

---

`CharsetMode`

Defines the character family used for rendering.

---

`ResizeFilter`

Controls image resampling quality.

---

`RenderStyle`

Controls layout-related styling such as full-block mode and density.

---

**Example: Custom Width and Style**

```rs
use image::open;
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};

fn main() -> anyhow::Result<()> {
    let img = open("sprite.png")?;

    let opts = RenderOptions::builder()
        .style(Some(RenderStylePreset::FullBlock))
        .width(80)
        .filter(Some(ResizeFilter::Nearest))
        .build();

    let prepared = opts.prepare_image(&img);
    prepared.save("preview.png")?;

    Ok(())
}
```

---

**Example: Rendering to a Buffer**

```rs
use image::open;
use px2ansi::{RenderOptions, write_ansi_art};

fn main() -> anyhow::Result<()> {
    let img = open("photo.png")?;
    let opts = RenderOptions::default();
    let prepared = opts.prepare_image(&img);

    let mut buf = Vec::new();
    write_ansi_art(&prepared, &mut buf, opts)?;

    let ansi = String::from_utf8(buf)?;
    println!("{ansi}");

    Ok(())
}
```

---

**Rasterization**

If you render ANSI art into a byte stream, you can turn it back into a PNG with
rasterize_ansi.

```rs
use px2ansi::rasterize_ansi;

fn main() -> anyhow::Result<()> {
    let input = b"\x1b[31m██\x1b[0m\n";
    let png = rasterize_ansi(input)?;
    std::fs::write("out.png", png)?;
    Ok(())
}
```

---

**Re-exports**

The crate root re-exports the most common types so users do not need to dig
through internal modules:

```rust
use px2ansi::{
    CharsetMode,
    Density,
    RenderOptions,
    RenderOptionsBuilder,
    RenderStyle,
    RenderStylePreset,
    ResizeFilter,
    rasterize_ansi,
    write_ansi_art,
};
```

---

**Library vs CLI**

`px2ansi` is the reusable rendering library.

If you want the command-line interface, install `px2ansi-rs` instead.

## License

GPL-3.0

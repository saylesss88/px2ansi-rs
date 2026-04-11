# px2ansi library

[![Crates.io](https://img.shields.io/crates/v/px2ansi.svg)](https://crates.io/crates/px2ansi)
[![Documentation](https://docs.rs/px2ansi/badge.svg)](https://docs.rs/px2ansi)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

If you want the command-line interface, check out [px2ansi-rs](../cli).


`px2ansi` converts images into terminal art by resizing them to terminal cell
proportions, mapping pixels to several character sets, and writing ANSI-colored
output to any `Write` target.

It is the rendering core behind `px2ansi-rs`, but it can also be used directly
in other Rust projects.

## Features

- Multiple rendering styles: `ansi`, `unicode`, `braille`, `fade`, `ascii`,
  `chinese`, `kanji`, `sixel`.

- Configurable resize filters.

- Automatic terminal-friendly dimension calculation.

- Write ANSI art to any `std::io::Write` target.

- Optionally rasterize ANSI output back into PNG

- Optional Sixel output for terminals that support it

## Installation

Add `px2ansi` to your `Cargo.toml`:

```toml
[dependencies]
px2ansi = "0.1.2"
image = "0.25.10"
```

If you only want the core engine and already have `image` in your project, just
depend on `px2ansi` and reuse your existing image setup.

**Quick Start**

```rust
use image::open;
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter, write_ansi_art};

fn main() -> anyhow::Result<()> {
    let img = open("photo.png")?;

    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Braille)
        .width(120)
        .filter(ResizeFilter::Nearest)
        .build();

    let prepared = opts.prepare_image(&img);

    let mut out = std::io::stdout();
    write_ansi_art(&prepared, &mut out, opts)?;

    Ok(())
}
```

Alternative style:

```rust
let mut builder = RenderOptions::builder();
builder.preset(RenderStylePreset::FullBlock);
builder.width(80);

if some_condition {
    builder.color(false);
}

let opts = builder.build();
```

**Automatic Centering and Resizing**

The library can automatically detect terminal size and center the output for
you:

```rust
let mut stdout = std::io::stdout();
opts.render_centered(&img, &mut stdout)?;
```

**Core Types**

| **Type**            | **Purpose**                               |
| ------------------- | ----------------------------------------- |
| `RenderOptions`     | Main render settings.                     |
| `RenderStylePreset` | Ready-made presets for common styles.     |
| `CharsetMode`       | The character set used to render pixels.  |
| `Density`           | Output density for ASCII-style rendering. |
| `RenderStyle`       | Low-level style tweaks.                   |

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

- `Sixel`

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

## Examples

### Custom Width and Style

```rust
use image::open;
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};

fn main() -> anyhow::Result<()> {
    let img = open("sprite.png")?;

    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::FullBlock)
        .width(80)
        .filter(ResizeFilter::Nearest)
        .build();

    let prepared = opts.prepare_image(&img);
    prepared.save("preview.png")?;

    Ok(())
}
```

---

### Example: Rendering to a Buffer

```rust
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

### Rasterization

If you render ANSI art into a byte stream, you can turn it back into a PNG with
`rasterize_ansi`.

```rust
use px2ansi::rasterize_ansi;

fn main() -> anyhow::Result<()> {
    let input = b"\x1b[31m██\x1b[0m\n";
    let png = rasterize_ansi(input)?;
    std::fs::write("out.png", png)?;
    Ok(())
}
```

---

### Inspecting Options

```rust
// Once you have an options object, you can inspect its state:
let opts = RenderOptions::builder()
    .preset(RenderStylePreset::FullBlock)
    .build();

if opts.style().is_full() {
    println!("Rendering in double-width mode!");
}

println!("Current density: {:?}", opts.style().density());
```

---

### Reusing the Builder

```rust
// New capability: Reusing a builder
let mut builder = RenderOptions::builder();
builder.width(100).filter(ResizeFilter::Triangle);

let low_res = builder.build();

// Change one thing and build again
let high_res = builder.width(200).build();
```

## Re-exports

The crate root re-exports the most common types so users do not need to dig
through internal modules:

```rust
use px2ansi::{
    cli_enums::{RenderStylePreset, ResizeFilter},
    indexer::{ImageEntry, build_index},
    rasterize::rasterize_ansi,
    render::{
        CharsetMode, Density, RenderOptions, RenderOptionsBuilder, RenderStyle, write_ansi_art,
    },
};
```

---

### Using the Indexer as a Library

The indexer is part of the public `px2ansi` library API and can be used
independently in your own Rust projects:

```rust
use px2ansi::indexer::{build_index, ImageEntry};
use std::path::Path;

// Build the index
build_index(
    Path::new("/home/user/sprites"),
    Path::new("/home/user/sprites/index.json"),
)?;

// Load and use it
let json = std::fs::read_to_string("index.json")?;
let entries: Vec<ImageEntry> = serde_json::from_str(&json)?;

for entry in &entries {
    println!("{}: {}x{}px at {}",
        entry.name,
        entry.dimensions.0,
        entry.dimensions.1,
        entry.path
    );
}
```

### Index Format

The index is a plain JSON file. Easy to inspect, version control, or process
with other tools:

```json
[
  {
    "name": "pikachu",
    "path": "/home/user/sprites/pikachu.png",
    "dimensions": [96, 96]
  },
  {
    "name": "charizard",
    "path": "/home/user/sprites/charizard.png",
    "dimensions": [128, 128]
  }
]
```

---

### Advanced Usage: Manual Rendering

For most cases, render_centered is the easiest way to go. However, if you need
full control over the image scaling or want to skip the terminal size detection,
you can use the low-level render method.

This is useful for TUI applications or when rendering to non-terminal targets
like files or network streams.

```rust
use px2ansi::RenderOptions;

fn custom_pipeline(img: &image::DynamicImage) -> anyhow::Result<()> {
    let opts = RenderOptions::default();

    // 1. Manually prepare the image (resizing happens here)
    // You can also use your own resizing logic before passing it to render!
    let prepared = opts.prepare_image(img);

    // 2. Render directly to a writer (no automatic centering)
    let mut stdout = std::io::stdout();
    opts.render(&prepared, &mut stdout)?;

    Ok(())
}
```

---

## Optional Features

Both features are **enabled by default**. Disable them individually or together
for minimal builds.

| Feature     | Dependency | What it does                                                         |
| ----------- | ---------- | -------------------------------------------------------------------- |
| `rasterize` | `fontdue`  | Renders ANSI art to a PNG image using an embedded monospace font     |
| `sixel`     | `viuer`    | Streams pixel-accurate images directly to Sixel-compatible terminals |

### Controlling features

```bash
# Minimal - pure ANSI text output only
cargo add px2ansi --no-default-features

# Sixel terminal output, no PNG rasterization
cargo add px2ansi --no-default-features --features sixel

# PNG rasterization, no Sixel output
cargo add px2ansi --no-default-features --features rasterize

# Everything (Full feature set)
cargo add px2ansi --features full
```

In `Cargo.toml`:

```toml
# Default (both features on)
px2ansi = "0.1.2"

# Minimal
px2ansi = { version = "0.1.2", default-features = false }

# Pick what you need
px2ansi = { version = "0.1.2", default-features = false, features = ["rasterize"] }
```

---

### Sixel

Renders pixel-accurate images inline in the terminal using the
[Sixel graphics protocol](https://en.wikipedia.org/wiki/Sixel).

**Compatible terminals:** foot, WezTerm, iTerm2, mlterm, xterm (with `-ti 340`)

```rust
use px2ansi::{RenderOptions, RenderStylePreset};
use std::io::stdout;

let mut builder = RenderOptions::builder();
builder.preset(RenderStylePreset::Sixel);
let opts = builder.build();

opts.render_centered(&img, &mut stdout())?;
```

---

### Rasterize

Converts ANSI art to a PNG image using an embedded
[Iosevka Charon Mono](https://github.com/nicowillis/iosevka-charon) font. Useful
for saving previews or sharing output as an image.

Use the default TokyoNight Theme:

If you render ANSI art into a byte stream, you can turn it back into a PNG with
`rasterize_ansi`.

```rust
use px2ansi::rasterize_ansi;

fn main() -> anyhow::Result<()> {
    let input = b"\x1b[31m██\x1b[0m\n";
    let png = rasterize_ansi(input)?;
    std::fs::write("out.png", png)?;
    Ok(())
}
```


```rust
use px2ansi::{RasterTheme, rasterize_ansi_with_theme};

// First render to an ANSI buffer
let mut buf = Vec::new();
opts.render_centered(&img, &mut buf)?;

// Then rasterize to PNG with a theme background
let png = rasterize_ansi_with_theme(&buf, RasterTheme::Dracula)?;
png.save("output.png")?;
```

Available themes: `TokyoNight` (default), `Dracula`, `Nord`, `GruvboxDark`,
`OneDark`, `SolarizedDark`, `Black`, `White`

---

### Error Handling

Unlike the CLI which uses anyhow for simplicity, the `px2ansi` library provides
a structured RenderError enum. This allows you to programmatically react to
specific failure states using `thiserror.`

```rust
use px2ansi::{RenderOptions, RenderError, CharsetMode};
use std::str::FromStr;

fn main() {
    let result = CharsetMode::from_str("invalid_mode");

    match result {
        Err(RenderError::InvalidCharset(name)) => {
            eprintln!("User tried to use an unsupported charset: {name}");
        }
        Err(RenderError::Io(e)) => {
            eprintln!("A writing error occurred: {e}");
        }
        _ => { /* ... */ }
    }
}
```

**Common Error Variants**

| **Variant**              | **Description**                                                               |
| ------------------------ | ----------------------------------------------------------------------------- |
| `InvalidCharset(String)` | Triggered when a string cannot be parsed into a valid `CharsetMode`.          |
| `InvalidDensity(String)` | Triggered when a string cannot be parsed into a valid `Density`. (ASCII Only) |
| `Io(std::io::Error)`     | Wrapped standard I/O errors (e.g., pipe broken, disk full).                   |
| `Image(String)`          | Errors occurred during image manipulation or resizing.                        |

---

## Library vs CLI

`px2ansi` is the reusable rendering library.

If you want the command-line interface, install `px2ansi-rs` instead.

## License

GPL-3.0

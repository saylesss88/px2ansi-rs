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

> [!NOTE]
> This is a new project, the public API is subject to change

## Features

- Multiple rendering styles: `Ansi`, `Unicode`, `Braille`, `Fade`, `Ascii`,
  `Chinese`, `Kanji`, `FullBlock`, `Dense`, `Sixel`.

- Configurable resize filters.

- Automatic terminal-friendly dimension calculation.

- Write ANSI art to any `std::io::Write` target.

- Optionally rasterize ANSI output back into PNG (with selectable themes).

- Optional Sixel output for terminals that support it.

## Installation

Add `px2ansi` to your `Cargo.toml`:

```toml
[dependencies]
px2ansi = "0.1.1"
image = "0.25"
```

If you only want the core engine and already have `image` in your project, just
depend on `px2ansi` and reuse your existing image setup.

## Quick Start

```rust
use image::open;
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};

fn main() -> anyhow::Result<()> {
    let img = open("photo.png")?;

    let opts = RenderOptions::builder()
        .preset(RenderStylePreset::Braille)
        .width(120)
        .filter(ResizeFilter::Nearest)
        .build();

    let mut out = std::io::stdout();
    opts.render(&img, &mut out)?;

    Ok(())
}
```

### Automatic Centering

The library can automatically detect terminal size, center the output, and
handle resizing for you:

```rust
let mut stdout = std::io::stdout();
opts.render_centered(&img, &mut stdout)?;
```

### Rendering to a Buffer

You can render to any `std::io::Write` target, including an in-memory buffer:

```rust
use image::open;
use px2ansi::RenderOptions;

fn main() -> anyhow::Result<()> {
    let img = open("photo.png")?;
    let opts = RenderOptions::default();

    let mut buf = Vec::new();
    opts.render(&img, &mut buf)?;

    let ansi = String::from_utf8(buf)?;
    println!("{ansi}");

    Ok(())
}
```

`render` also works with a `std::io::Cursor`:

```rust
use std::io::Cursor;
use px2ansi::RenderOptions;

let opts = RenderOptions::default();
let mut cursor = Cursor::new(Vec::new());
opts.render(&img, &mut cursor)?;
```

### Manual Image Preparation

If you need control over the image scaling step, use `prepare_image` separately.
This is useful for TUI applications or when rendering to non-terminal targets
like files or network streams:

```rust
use px2ansi::RenderOptions;

fn custom_pipeline(img: &image::DynamicImage) -> anyhow::Result<()> {
    let opts = RenderOptions::builder().width(40).build();

    // 1. Manually prepare the image (resizing happens here)
    let prepared = opts.prepare_image(img);
    assert_eq!(prepared.width(), 40);

    // 2. Render directly to a writer (no automatic centering)
    let mut stdout = std::io::stdout();
    opts.render(&prepared, &mut stdout)?;

    Ok(())
}
```

---

## Core Types

| **Type**              | **Purpose**                                                  |
| --------------------- | ------------------------------------------------------------ |
| `RenderOptions`       | Main render settings (width, filter, charset, color, etc.).  |
| `RenderOptionsBuilder`| Builder for constructing `RenderOptions` step-by-step.       |
| `RenderStylePreset`   | Ready-made presets for common styles.                        |
| `CharsetMode`         | The character set used to render pixels.                     |
| `Density`             | Output density for ASCII-style rendering (`Light`, `Medium`, `Heavy`). |
| `RenderStyle`         | Low-level style tweaks (`is_full()`, `density()`).           |
| `ResizeFilter`        | Controls image resampling quality.                           |
| `ColorMode`           | Color output mode: `TrueColor`, `Ansi256`, or `None`.       |
| `RenderError`         | Structured error type for rendering failures.                |

### `RenderOptions`

The main configuration object for rendering. Controls target width, resize
filter, charset mode, density, and color output.

Default configuration:
- Charset: `Ansi`
- Color: `true`
- Width: `None` (auto-detect from terminal)

```rust
let opts = RenderOptions::default();
assert_eq!(opts.charset(), CharsetMode::Ansi);
assert!(opts.color());
assert_eq!(opts.width(), None);
```

### `RenderStylePreset`

A convenience enum for quickly choosing a style:

| Preset      | Charset     | Notes                              |
| ----------- | ----------- | ---------------------------------- |
| `Ansi`      | `Ansi`      | Half-block characters (▀/▄)        |
| `Unicode`   | `Unicode`   | Half or full blocks                |
| `Braille`   | `Braille`   | 2×4 Braille dot patterns           |
| `Fade`      | `Fade`      | Block-shade ramp (░▒▓█)            |
| `Ascii`     | `Ascii`     | 92-character density ramp          |
| `Kanji`     | `Kanji`     | Double-width Japanese characters   |
| `Chinese`   | `Chinese`   | Double-width Chinese characters    |
| `FullBlock` | `Unicode`   | Forces double-width full blocks (██), sets `is_full() = true` |
| `Dense`     | `Ascii`     | ASCII with `Density::Heavy`        |
| `Sixel`     | `Sixel`     | Pixel-accurate Sixel output        |

### `Density`

Controls the complexity of the ASCII character ramp. Only affects `Ascii` mode:

- `Light` — Sparse, minimal characters
- `Medium` — Default 92-character ramp
- `Heavy` — Dense ramp including block elements

### `ResizeFilter`

Controls image resampling quality:

| Filter       | Description                          |
| ------------ | ------------------------------------ |
| `Nearest`    | Best for pixel art                   |
| `Triangle`   | Linear interpolation                 |
| `CatmullRom` | Sharp cubic filter                   |
| `Gaussian`   | Blurry cubic filter                  |
| `Lanczos3`   | High-quality resampling (slowest)    |

---

## Builder API

The builder supports chaining:

```rust
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter, Density};

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Ascii)
    .density(Density::Light)
    .width(120)
    .filter(ResizeFilter::Nearest)
    .color(false)
    .build();
```

Or a mutable style:

```rust
let mut builder = RenderOptions::builder();
builder.preset(RenderStylePreset::FullBlock);
builder.width(80);

if some_condition {
    builder.color(false);
}

let opts = builder.build();
```

### Inspecting Options

```rust
use px2ansi::{RenderOptions, RenderStylePreset};

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::FullBlock)
    .build();

if opts.style().is_full() {
    println!("Rendering in double-width mode!");
}

println!("Current density: {:?}", opts.style().density());
```

---

## Indexer

The indexer scans a directory for image files and produces a JSON index. It is
part of the public `px2ansi` library API:

```rust
use px2ansi::indexer::{build_index, ImageEntry};
use std::path::Path;

// Build the index — scans subdirectories, ignores non-image files
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

**Indexer behavior:**

- Recursively scans subdirectories for images.
- Ignores non-image files (`.txt`, `.json`, etc.).
- Entries are sorted alphabetically by name.
- An empty directory produces an empty JSON array (`[]`).
- Image names are derived from the file stem (without extension).

### Index Format

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

## Optional Features

All features are **enabled by default**. Disable them individually or together
for minimal builds.

| Feature     | Dependency | What it does                                                         |
| ----------- | ---------- | -------------------------------------------------------------------- |
| `rasterize` | `fontdue`  | Renders ANSI art to a PNG image using an embedded monospace font      |
| `sixel`     | `viuer`    | Streams pixel-accurate images directly to Sixel-compatible terminals |
| `parallel`  | `rayon`    | Enables parallel processing for performance                          |

### Controlling Features

```bash
# Minimal — pure ANSI text output only
cargo add px2ansi --no-default-features

# Sixel terminal output, no PNG rasterization
cargo add px2ansi --no-default-features --features sixel

# PNG rasterization, no Sixel output
cargo add px2ansi --no-default-features --features rasterize

# Everything (full feature set)
cargo add px2ansi --features full
```

In `Cargo.toml`:

```toml
# Default (rasterize + sixel + parallel)
px2ansi = "0.2.1"

# Minimal
px2ansi = { version = "0.2.1", default-features = false }

# Pick what you need
px2ansi = { version = "0.2.1", default-features = false, features = ["rasterize"] }
```

---

### Sixel

Renders pixel-accurate images inline in the terminal using the
[Sixel graphics protocol](https://en.wikipedia.org/wiki/Sixel).

**Compatible terminals:** foot, WezTerm, iTerm2, mlterm, xterm (with `-ti 340`)

```rust
use px2ansi::{RenderOptions, RenderStylePreset};
use std::io::stdout;

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Sixel)
    .build();

opts.render_centered(&img, &mut stdout())?;
```

---

### Rasterize

Converts ANSI art to a PNG image using an embedded
[Iosevka Charon Mono](https://github.com/nicowillis/iosevka-charon) font. Useful
for saving previews or sharing output as an image.

**With the default TokyoNight theme:**

```rust
use px2ansi::{RenderOptions, rasterize_ansi_with_theme, RasterTheme};

let img = image::open("photo.png")?;
let opts = RenderOptions::default();

// Render to an ANSI buffer
let mut buf = Vec::new();
opts.render(&img, &mut buf)?;

// Rasterize to a PNG image
let png = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight)?;
png.save("output.png")?;
```

**With a different theme:**

```rust
use px2ansi::{RasterTheme, rasterize_ansi_with_theme};

let png = rasterize_ansi_with_theme(&buf, RasterTheme::Dracula)?;
png.save("output.png")?;
```

**Available themes:** `TokyoNight` (default), `Dracula`, `Nord`, `GruvboxDark`,
`OneDark`, `SolarizedDark`, `Black`, `White`

> Different themes produce different background colors. For example, `TokyoNight`
> and `White` will render visibly different backgrounds.

---

## Re-exports

The crate root re-exports the most common types so users do not need to dig
through internal modules:

```rust
use px2ansi::{
    // Core rendering
    RenderOptions, RenderOptionsBuilder, RenderStyle,
    CharsetMode, Density,
    write_ansi_art,

    // Presets and filters
    RenderStylePreset, ResizeFilter,

    // Indexer
    indexer::{ImageEntry, build_index},

    // Rasterization (requires "rasterize" feature)
    // rasterize_ansi, rasterize_ansi_with_theme, RasterTheme,
};
```

---

## Error Handling

Unlike the CLI which uses `anyhow` for simplicity, the `px2ansi` library provides
a structured `RenderError` enum. This allows you to programmatically react to
specific failure states.

```rust
use px2ansi::{CharsetMode, RenderError};
use std::str::FromStr;

fn main() {
    let result = CharsetMode::from_str("invalid_mode");

    match result {
        Err(RenderError::InvalidCharset(name)) => {
            eprintln!("Unsupported charset: {name}");
        }
        Err(RenderError::Io(e)) => {
            eprintln!("A writing error occurred: {e}");
        }
        _ => { /* ... */ }
    }
}
```

**Error Variants**

| **Variant**              | **Description**                                                               |
| ------------------------ | ----------------------------------------------------------------------------- |
| `InvalidCharset(String)` | Triggered when a string cannot be parsed into a valid `CharsetMode`.          |
| `InvalidDensity(String)` | Triggered when a string cannot be parsed into a valid `Density`.              |
| `Io(std::io::Error)`     | Wrapped standard I/O errors (e.g., pipe broken, disk full).                   |
| `Image(String)`          | Errors during image manipulation or resizing.                                 |

---

## Library vs CLI

`px2ansi` is the reusable rendering library.

If you want the command-line interface, install `px2ansi-rs` instead.

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)


# px2ansi

[![Crates.io](https://img.shields.io/crates/v/px2ansi.svg)](https://crates.io/crates/px2ansi)
[![Documentation](https://docs.rs/px2ansi/badge.svg)](https://docs.rs/px2ansi)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`px2ansi` converts images into terminal art by resizing them to terminal cell
proportions, mapping pixels to several character sets, and writing ANSI-colored
output to any `Write` target.

It is the rendering core behind [`px2ansi-rs`](../cli) (the CLI), but can be
used directly in other Rust projects.

> [!IMPORTANT]
> This is a new project, the public API is subject to change.

If you want the command-line interface, see [px2ansi-rs](../cli).

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Types](#core-types)
- [Builder API](#builder-api)
- [Indexer](#indexer)
- [Optional Features](#optional-features)
- [Performance](#performance)
- [Re-exports](#re-exports)
- [Error Handling](#error-handling)
- [License](#license)

---

## Installation

```toml
[dependencies]
px2ansi = "0.3.14"
image = "0.25"
```

---

## Quick Start

```rust
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};

let img = image::open("image.png").unwrap();

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Braille)
    .width(120)
    .filter(ResizeFilter::Nearest)
    .build();

opts.render(&img, &mut std::io::stdout()).unwrap();
```

### Render Centered (auto terminal sizing)

```rust
let opts = RenderOptions::default();
opts.render_centered(&img, &mut std::io::stdout()).unwrap();
```

### Render to a Buffer

```rust
let opts = px2ansi::RenderOptions::default();
let mut buf = Vec::new();
opts.render(&img, &mut buf).unwrap();
let ansi = String::from_utf8(buf).unwrap();
```

### Manual Image Preparation

Useful when you need control over the resize step (e.g. TUI apps):

```rust
let opts = RenderOptions::builder().width(40).build();
let prepared = opts.prepare_image(&img); // resize happens here
opts.render(&prepared, &mut std::io::sink()).unwrap();
```

---

## Core Types

| Type                   | Purpose                                                                |
| ---------------------- | ---------------------------------------------------------------------- |
| `RenderOptions`        | Main render settings (width, filter, charset, color, etc.)             |
| `RenderOptionsBuilder` | Builder for constructing `RenderOptions` step-by-step                  |
| `RenderStylePreset`    | Ready-made presets for common styles                                   |
| `CharsetMode`          | The character set used to render pixels                                |
| `Density`              | Output density for ASCII-style rendering (`Light`, `Medium`, `Heavy`)  |
| `RenderStyle`          | Low-level style tweaks (`is_full()`, `density()`)                      |
| `ResizeFilter`         | Controls image resampling quality                                      |
| `ColorMode`            | Color output mode: `TrueColor`, `Ansi256`, or `None`                   |
| `RenderError`          | Structured error type for rendering failures                           |

### `RenderOptions` defaults

- Charset: `Ansi`
- Color mode: `TrueColor`
- Width: `None` (auto-detect from terminal)

### `RenderStylePreset`

| Preset      | Notes                                              |
| ----------- | -------------------------------------------------- |
| `Ansi`      | Half-block characters (▀/▄)                        |
| `Unicode`   | Half or full blocks                                |
| `Braille`   | 2×4 Braille dot patterns                           |
| `Fade`      | Block-shade ramp (░▒▓█)                            |
| `Ascii`     | 92-character density ramp                          |
| `Kanji`     | Double-width Japanese characters                   |
| `Chinese`   | Double-width Chinese characters                    |
| `FullBlock` | Double-width solid blocks (██), `is_full() = true` |
| `Dense`     | ASCII with `Density::Heavy`                        |
| `Sixel`     | Pixel-accurate Sixel output (requires feature)     |

### `ResizeFilter`

| Filter       | Description                       |
| ------------ | --------------------------------- |
| `Nearest`    | Best for pixel art                |
| `Triangle`   | Linear interpolation              |
| `CatmullRom` | Sharp cubic filter                |
| `Gaussian`   | Blurry cubic filter               |
| `Lanczos3`   | High-quality resampling (slowest) |

---

## Builder API

```rust
use px2ansi::{ColorMode, RenderOptions, RenderStylePreset};

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::FullBlock)
    .width(80)
    .color_mode(ColorMode::None)
    .build();

if opts.style().is_full() {
    println!("double-width mode");
}
```

---

## Indexer

Scans a directory for images and produces a JSON index:

```rust
use px2ansi::indexer::{build_index, ImageEntry};
use std::path::Path;

build_index(
    Path::new("/home/user/sprites"),
    Path::new("/home/user/sprites/index.json"),
).unwrap();

let json = std::fs::read_to_string("index.json").unwrap();
let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();

for entry in &entries {
    println!("{}: {}x{}px", entry.name, entry.dimensions.0, entry.dimensions.1);
}
```

**Index format:**

```json
[
  { "name": "pikachu",   "path": "/sprites/pikachu.png",   "dimensions": [96, 96]   },
  { "name": "charizard", "path": "/sprites/charizard.png", "dimensions": [128, 128] }
]
```

- Recursively scans subdirectories; ignores non-image files.
- Entries sorted alphabetically by name.
- Empty directory → empty JSON array `[]`.

---

## Optional Features

All features are **disabled by default**.

| Feature     | Dependency  | What it does                                            |
| ----------- | ----------- | ------------------------------------------------------- |
| `sixel`     | `icy_sixel` | Pixel-accurate Sixel protocol output                    |
| `rasterize` | `fontdue`   | Converts ANSI output back to PNG with selectable themes |
| `parallel`  | `rayon`     | Multi-threaded rendering for large images (>120,000 px) |

```toml
# Pick what you need
px2ansi = { version = "0.3.12", features = ["parallel", "sixel"] }

# Everything
px2ansi = { version = "0.3.12", features = ["full"] }
```

### Sixel

Compatible terminals: `foot`, `WezTerm`, `iTerm2`, `ghostty`, `xterm -ti 340`

```rust
let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Sixel)
    .max_colors(256)  // default: 64, max: 256
    .diffusion(0.8)   // 0.0 = none, 1.0 = full Floyd-Steinberg
    .build();

opts.render_centered(&img, &mut std::io::stdout()).unwrap();
```

### Dithering

Floyd-Steinberg error diffusion to reduce banding in gradients:

```rust
let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Ascii)
    .dither(true)
    .build();
```

### Rasterize

Converts ANSI art to a PNG using an embedded Iosevka Charon Mono font:

```rust
#[cfg(feature = "rasterize")]
{
    use px2ansi::{RenderOptions, RasterTheme, rasterize_ansi_with_theme};

    let mut buf = Vec::new();
    RenderOptions::default().render(&img, &mut buf).unwrap();

    let png = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight).unwrap();
    png.save("output.png").unwrap();
}
```

**Themes:** `TokyoNight` (default), `Dracula`, `Nord`, `GruvboxDark`, `OneDark`,
`SolarizedDark`, `Black`, `White`

---

## Performance

The pixel pipeline uses **LLVM auto-vectorization**, no unsafe intrinsics, no
platform-specific code. On capable hardware, LLVM emits AVX2/NEON instructions
automatically.

To unlock wider SIMD on your own machine:

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

The `parallel` feature activates Rayon-based multi-threading automatically when
the pixel count exceeds **120,000 pixels**, avoiding overhead on typical terminal
output.

Run the bundled benchmarks:

```bash
cargo bench --bench pixels      # pixel pipeline only
cargo bench --features sixel    # include Sixel groups
cargo bench --features full     # everything
```

---

## Re-exports

```rust
// Core (always available)
use px2ansi::{
    RenderOptions, RenderOptionsBuilder, RenderStyle,
    CharsetMode, ColorMode, Density,
    write_ansi_art, get_terminal_size,
    RenderStylePreset, ResizeFilter,
    ImageEntry, build_index,
};

// Rasterize (feature = "rasterize")
#[cfg(feature = "rasterize")]
use px2ansi::{ rasterize_ansi, rasterize_ansi_with_theme, RasterTheme };
```

---

## Error Handling

Structured `RenderError` enum (unlike the CLI which uses `anyhow`):

```rust
use px2ansi::{CharsetMode, RenderError};
use std::str::FromStr;

match CharsetMode::from_str("invalid_mode") {
    Err(RenderError::InvalidCharset(name)) => eprintln!("Unknown charset: {name}"),
    Err(e) => eprintln!("Error: {e}"),
    Ok(_) => {}
}
```

| Variant                   | Description                                            |
| ------------------------- | ------------------------------------------------------ |
| `InvalidCharset(String)`  | String cannot be parsed into a valid `CharsetMode`     |
| `InvalidDensity(String)`  | String cannot be parsed into a valid `Density`         |
| `Io(std::io::Error)`      | Standard I/O errors (pipe broken, disk full, etc.)     |
| `Image(String)`           | Errors during image manipulation or resizing           |
| `Font(String)`            | Errors during font loading or glyph rasterization      |
| `EmptyCells`              | ANSI input parsed to zero cells                        |
| `Json(serde_json::Error)` | JSON serialization errors for the image index          |

---

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

# px2ansi library

[![Crates.io](https://img.shields.io/crates/v/px2ansi.svg)](https://crates.io/crates/px2ansi)
[![Documentation](https://docs.rs/px2ansi/badge.svg)](https://docs.rs/px2ansi)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

If you want the command-line interface, check out [px2ansi-rs](../cli).

## Table of Contents

<details>
<summary> Table of Contents </summary>

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
  - [Automatic Centering](#automatic-centering)
  - [Rendering to a Buffer](#rendering-to-a-buffer)
  - [Manual Image Preparation](#manual-image-preparation)
- [Core Types](#core-types)
  - [RenderOptions](#renderoptions-1)
  - [RenderStylePreset](#renderstylepreset-1)
  - [Density](#density-1)
  - [ResizeFilter](#resizefilter-1)
- [Builder API](#builder-api)
  - [Inspecting Options](#inspecting-options)
- [Indexer](#indexer)
  - [Index Format](#index-format)
- [Optional Features](#optional-features)
  - [Controlling Features](#controlling-features)
  - [Sixel](#sixel)
  - [Dithering](#dithering)
  - [Rasterize](#rasterize)
- [⚡ Performance](#-performance)
  - [SIMD Acceleration](#simd-acceleration-simd-feature)
    - [Benchmark](#benchmark)
  - [Parallel Rendering](#parallel-rendering-parallel-feature)
- [Re-exports](#re-exports)
- [Error Handling](#error-handling)
- [Library vs CLI](#library-vs-cli)
- [License](#license)

## </details>

`px2ansi` converts images into terminal art by resizing them to terminal cell
proportions, mapping pixels to several character sets, and writing ANSI-colored
output to any `Write` target.

It is the rendering core behind `px2ansi-rs`, but it can also be used directly
in other Rust projects.

> [!IMPORTANT]
> This is a new project, the public API is subject to change

## Features

- Multiple rendering styles: `Ansi`, `Unicode`, `Braille`, `Fade`, `Ascii`,
  `Chinese`, `Kanji`, `FullBlock`, `Dense`, `Sixel`.

- Configurable resize filters.

- Automatic terminal-friendly dimension calculation.

- Write ANSI art to any `std::io::Write` target.

- Optionally rasterize ANSI output back into PNG (with selectable themes).

- Optional Sixel output for terminals that support it.

- Optional SIMD (`wide`) and optional parallel execution (`rayon`)

## Installation

Add `px2ansi` to your `Cargo.toml`:

```toml
[dependencies]
px2ansi = "0.2.3"
image = "0.25"
```

If you only want the core engine and already have `image` in your project, just
depend on `px2ansi` and reuse your existing image setup.

## Quick Start

```rust
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};
# use image::{DynamicImage, Rgba};
# let img = DynamicImage::ImageRgba8(image::ImageBuffer::new(1, 1));
#
// Build options: use a preset and override specific fields
let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Braille)
    .width(120)
    .filter(ResizeFilter::Nearest)
    .build();
// Render directly to stdout
let mut out = std::io::sink();
opts.render(&img, &mut out).unwrap();
```

### Automatic Centering

The library can automatically detect terminal size, center the output, and
handle resizing for you:

```rust
use px2ansi::RenderOptions;
use std::io;
# use image::{DynamicImage, ImageBuffer};
# let opts = RenderOptions::default();
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(1, 1));
let mut stdout = io::sink(); // Using sink prevents flooding test output
opts.render_centered(&img, &mut stdout).unwrap();
```

### Rendering to a Buffer

You can render to any `std::io::Write` target, including an in-memory buffer:

```rust
# use image::{DynamicImage, ImageBuffer};
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(1, 1));
let opts = px2ansi::RenderOptions::default();
let mut buf = Vec::new();
opts.render(&img, &mut buf).unwrap();
let ansi = String::from_utf8(buf).unwrap();
// In a real app, you'd println!("{ansi}");
```

`render` also works with a `std::io::Cursor`:

```rust
use px2ansi::RenderOptions;
use std::io::Cursor;
# use image::{DynamicImage, ImageBuffer};
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(1, 1));

let opts = RenderOptions::default();
let mut cursor = Cursor::new(Vec::new());

opts.render(&img, &mut cursor).unwrap();

// Access the data from the cursor
let _result = cursor.into_inner();
```

### Manual Image Preparation

If you need control over the image scaling step, use `prepare_image` separately.
This is useful for TUI applications or when rendering to non-terminal targets
like files or network streams:

```rust
use px2ansi::RenderOptions;
# use image::{DynamicImage, ImageBuffer, Rgba};
# // Create a 100x100 dummy image to test resizing logic
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(100, 100));

let opts = RenderOptions::builder().width(40).build();

// 1. Manually prepare the image (resizing happens here)
let prepared = opts.prepare_image(&img);
assert_eq!(prepared.width(), 40);

// 2. Render directly to a writer
let mut sink = std::io::sink();
opts.render(&prepared, &mut sink).unwrap();
```

---

## Core Types

| **Type**               | **Purpose**                                                            |
| ---------------------- | ---------------------------------------------------------------------- |
| `RenderOptions`        | Main render settings (width, filter, charset, color, etc.).            |
| `RenderOptionsBuilder` | Builder for constructing `RenderOptions` step-by-step.                 |
| `RenderStylePreset`    | Ready-made presets for common styles.                                  |
| `CharsetMode`          | The character set used to render pixels.                               |
| `Density`              | Output density for ASCII-style rendering (`Light`, `Medium`, `Heavy`). |
| `RenderStyle`          | Low-level style tweaks (`is_full()`, `density()`).                     |
| `ResizeFilter`         | Controls image resampling quality.                                     |
| `ColorMode`            | Color output mode: `TrueColor`, `Ansi256`, or `None`.                  |
| `RenderError`          | Structured error type for rendering failures.                          |

### `RenderOptions`

The main configuration object for rendering. Controls target width, resize
filter, charset mode, density, and color output.

Default configuration:

- Charset: `Ansi`
- `Color_Mode`: `truecolor`
- Width: `Non` (auto-detect from terminal)

```rust
use px2ansi::{CharsetMode, ColorMode, RenderOptions};

let opts = RenderOptions::default();

assert_eq!(opts.charset(), CharsetMode::Ansi);
assert_eq!(opts.color_mode(), ColorMode::TrueColor);
assert_eq!(opts.width(), None);
```

### `RenderStylePreset`

A convenience enum for quickly choosing a style:

| Preset      | Charset   | Notes                                                         |
| ----------- | --------- | ------------------------------------------------------------- |
| `Ansi`      | `Ansi`    | Half-block characters (▀/▄)                                   |
| `Unicode`   | `Unicode` | Half or full blocks                                           |
| `Braille`   | `Braille` | 2×4 Braille dot patterns                                      |
| `Fade`      | `Fade`    | Block-shade ramp (░▒▓█)                                       |
| `Ascii`     | `Ascii`   | 92-character density ramp                                     |
| `Kanji`     | `Kanji`   | Double-width Japanese characters                              |
| `Chinese`   | `Chinese` | Double-width Chinese characters                               |
| `FullBlock` | `Unicode` | Forces double-width full blocks (██), sets `is_full() = true` |
| `Dense`     | `Ascii`   | ASCII with `Density::Heavy`                                   |
| `Sixel`     | `Sixel`   | Pixel-accurate Sixel output                                   |

### `Density`

Controls the complexity of the ASCII character ramp. Only affects `Ascii` mode:

- `Light` — Sparse, minimal characters
- `Medium` — Default 92-character ramp
- `Heavy` — Dense ramp including block elements

### `ResizeFilter`

Controls image resampling quality:

| Filter       | Description                       |
| ------------ | --------------------------------- |
| `Nearest`    | Best for pixel art                |
| `Triangle`   | Linear interpolation              |
| `CatmullRom` | Sharp cubic filter                |
| `Gaussian`   | Blurry cubic filter               |
| `Lanczos3`   | High-quality resampling (slowest) |

---

## Builder API

The builder supports chaining:

```rust
use px2ansi::{ColorMode, RenderOptions, RenderStylePreset};
# let monochrome = true;

let mut builder = RenderOptions::builder()
    .preset(RenderStylePreset::FullBlock)
    .width(80);
  
builder = if monochrome {
    builder.color_mode(ColorMode::None)
} else {
    builder
};

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

assert!(opts.style().is_full());
println!("Current density: {:?}", opts.style().density());
```

---

## Indexer

The indexer scans a directory for image files and produces a JSON index. It is
part of the public `px2ansi` library API:

```rust,no_run
use px2ansi::indexer::{build_index, ImageEntry};
use std::path::Path;

fn build_and_read_index() {
    // Build the index, scans subdirectories, ignores non-image files
    build_index(
        Path::new("/home/user/sprites"),
        Path::new("/home/user/sprites/index.json"),
    ).unwrap();

    // Load and use it
    let json = std::fs::read_to_string("index.json").unwrap();
    let entries: Vec<ImageEntry> = serde_json::from_str(&json).unwrap();

    for entry in &entries {
        println!("{}: {}x{}px at {}",
            entry.name,
            entry.dimensions.0,
            entry.dimensions.1,
            entry.path
        );
    }
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

All features are **disabled by default**. Enable them individually or together
for minimal builds.

| Feature     | Dependency | What it does                                                                                   |
| ----------- | ---------- | ---------------------------------------------------------------------------------------------- |
| `rasterize` | `fontdue`  | Renders ANSI art to a PNG image using an embedded monospace font                               |
| `sixel`     | `viuer`    | Streams pixel-accurate images directly to Sixel-compatible terminals                           |
| `parallel`  | `rayon`    | Enables parallel processing for performance                                                    |
| `simd`      | `wide`     | Accelerates color matching and pixel processing by using CPU vector instructions (AVX2, NEON). |

### Controlling Features

```bash
# Minimal — pure ANSI text output only
cargo add px2ansi

# Sixel terminal output, no PNG rasterization
cargo add px2ansi --features sixel

# PNG rasterization, no Sixel output
cargo add px2ansi --features rasterize

# Enable all optimization features
cargo add px2ansi --features simd parallel

# Everything (full feature set)
cargo add px2ansi --features full
```

In `Cargo.toml`:

```toml
# Default (Minimal, no features enabled)
px2ansi = "0.2.3"

# Pick what you need
px2ansi = { version = "0.2.3",  features = ["parallel", "simd"] }

# Include all features ("parallel", "simd", "rasterize", "sixel")
px2ansi = { version = "0.2.3",  features = ["full"] }
```

---

### Sixel

Renders pixel-accurate images inline in the terminal using the
[Sixel graphics protocol](https://en.wikipedia.org/wiki/Sixel).

**Compatible terminals:** `foot`, `WezTerm`, `iTerm2`, `ghosTTY`, `xterm` (with
`-ti 340`)

```rust,no_run
use px2ansi::{RenderOptions, RenderStylePreset};
# use image::{DynamicImage, ImageBuffer};
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(1, 1));

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Sixel)
    .build();

let mut out = std::io::stdout();
opts.render_centered(&img, &mut out).unwrap();
```

---

### Dithering

The `px2ansi` library implements a specialized Luminance-Preserving Error
Diffusion algorithm. This is designed to solve the "banding" problem common in
terminal art, where a limited character set or color palette creates harsh
transitions in gradients.

```rust,no_run
use px2ansi::{RenderOptions, RenderStylePreset};
# use image::{DynamicImage, ImageBuffer};
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(10, 10));

let opts = RenderOptions::builder()
    .preset(RenderStylePreset::Ascii)
    .dither(true) // Enables error-diffusion dither
    .build();

let mut buf = Vec::new();
opts.render(&img, &mut buf).unwrap();
```

### Rasterize

Converts ANSI art to a PNG image using an embedded
[Iosevka Charon Mono](https://github.com/nicowillis/iosevka-charon) font. Useful
for saving previews or sharing output as an image.

**With the default `TokyoNight` theme:**

It’s a very common instinct to wrap examples in functions to "protect" them from
the global scope, but in Rust doc tests, that actually creates a "dead code"
zone where the logic is never verified.

For the Rasterize feature, we have a new hurdle: it's likely gated behind a
cargo feature. If a user tries to run your docs without that feature enabled,
the test will fail. We can handle this gracefully using the # trick combined
with cfg attributes.

1. The Default Rasterize Example We'll strip the function, mock the image, and
   handle the feature gate so the test only runs if the rasterize feature is
   active.

Markdown

```rust
# #[cfg(feature = "rasterize")]
# {
use px2ansi::{RenderOptions, RasterTheme, rasterize_ansi_with_theme};
# use image::{DynamicImage, ImageBuffer};
# let img = DynamicImage::ImageRgba8(ImageBuffer::new(10, 10));

let opts = RenderOptions::default();

// Render to an ANSI buffer
let mut buf = Vec::new();
opts.render(&img, &mut buf).unwrap();

// Rasterize to a PNG image
let png = rasterize_ansi_with_theme(&buf, RasterTheme::TokyoNight).unwrap();
# // We use a sink/memory in tests instead of writing to disk
# let mut output_buf = std::io::Cursor::new(Vec::new());
# png.write_to(&mut output_buf, image::ImageFormat::Png).unwrap();
# }
```

**With a different theme:**

```rust
# #[cfg(feature = "rasterize")]
# {
use px2ansi::{RasterTheme, rasterize_ansi_with_theme};
# let buf = b"\x1b[31mHello\x1b[0m".to_vec(); // Mock ANSI data

let png = rasterize_ansi_with_theme(&buf, RasterTheme::Dracula).unwrap();
// Save to disk
// png.save("output.png").unwrap();
# }
```

**Available themes:** `TokyoNight` (default), `Dracula`, `Nord`, `GruvboxDark`,
`OneDark`, `SolarizedDark`, `Black`, `White`

> Different themes produce different background colors. For example,
> `TokyoNight` and `White` will render visibly different backgrounds.

---

## ⚡ Performance

### SIMD Acceleration (`simd` feature)

The `simd` feature enables hardware-accelerated pixel processing via the
[wide](https://crates.io/crates/wide) crate. This implementation is portable,
100% safe Rust, and requires no architecture-specific intrinsics or nightly
compiler. LLVM automatically lowers the wide vector types to AVX2, SSE4.1, or
NEON instructions depending on your target hardware.

### Performance Impact

`px2ansi` uses SIMD to accelerate the two most critical stages of rendering:

1. **Luma Range Scanning (Pass 1)**: Scans the entire image to find the
   perceptual brightness floor and ceiling (L*min* and L*max*). This allows the
   library to maximize contrast by stretching the character ramp to fit the
   image's actual dynamic range.

2. **Character Indexing (Pass 2)**: Maps every pixel to a character in your
   charset. By processing 8 pixels at a time in a single `u32x8` vector lane, we
   significantly reduce the CPU cycles required for the normalization math.

**How it works**

With the `simd` feature enabled, the renderer processes chunks of 8 pixels in a
single instruction rather than one at a time:

```text
// (explanatory snippet about SIMD — do not compile it in doctests)
// 8 Rec.709 luma values computed simultaneously:
// let luma_raw = r * u32x8::splat(2126) + g * u32x8::splat(7152) + b * u32x8::splat(722);
```

- **Branchless Transparency**: Transparent pixels (alpha < 30) are handled using
  bitmasks. This avoids per-pixel branching, keeping the CPU pipeline full and
  predictable.

- **Exact Fallback**: Any remaining pixels that don't fill a full 8-pixel chunk
  are handled by an identical scalar fallback, so output is bit-for-bit
  identical regardless of hardware support.

#### Rec.709 luma formula

Both the SIMD and scalar paths use the same perceptually-weighted formula:

```latex
$$Y = \frac{2126 \cdot R + 7152 \cdot G + 722 \cdot B}{10000}$$
```

This matches the [ITU-R BT.709](https://www.itu.int/rec/R-REC-BT.709) standard
used in HDTV and sRGB color spaces, giving accurate brightness perception across
all charset modes.

#### Enabling SIMD

```toml
[dependencies]
px2ansi = { version = "0.2.3", features = ["simd"] }
```

Or to enable everything:

```toml
[dependencies]
px2ansi = { version = "0.2.3", features = ["full"] }
```

If the `simd` feature is **not** enabled, the library automatically falls back
to a scalar implementation with identical output — no code changes required.

#### Benchmark

The following benchmarks demonstrate the impact of the `simd` feature when
processing high-resolution images. These tests were conducted using the
`px2ansi-rs` CLI on an

| Mode             | Wall time | User CPU | Test Image                    |
| ---------------- | --------- | -------- | ----------------------------- |
| `px2ansi` (SIMD) | ~7.6 ms   | ~2.2 ms  | `nixos.png` (1.2M px, sparse) |
| `px2ansi` (SIMD) | ~9.1 ms   | ~4.6 ms  | `scream.png` (0.6M px, dense) |
| `rascii --color` | ~10.0 ms  | ~4.5 ms  | `nixos.png`                   |
| `rascii --color` | ~10.2 ms  | ~5.8 ms  | `scream.png`                  |

The SIMD luma scan alone roughly **halves CPU time** for the render pass.

> [!NOTE]
> `px2ansi` scales better with resolution. Even though the NixOS image
> has double the pixels of the Scream image, it actually completes the task
> faster.

---

### Parallel Rendering (`parallel` feature)

The `parallel` feature enables [Rayon](https://crates.io/crates/rayon)-based
multi-threaded rendering via `into_par_iter()` for Pass 2 (glyph mapping and
colorization).

> [!IMPORTANT]
> Rayon has a fixed thread-pool startup cost of ~1–2 ms. For typical
> terminal-sized output (~200×100 = 20,000 pixels) this overhead
> **exceeds** the render time itself. The library therefore only activates
> parallel rendering dynamically when the pixel count exceeds **120,000
> pixels**,falling back to the fast serial + SIMD path otherwise:

```rust
# let (width, height) = (100, 100);
let use_parallel = cfg!(feature = "parallel") && (width * height > 120_000);
```

This means standard terminal rendering always hits the fast path, while large
off-screen or file renders automatically scale across all cores.

## Re-exports

The crate root re-exports the most common types so users do not need to dig
through internal modules:

```rust,no_run
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

Unlike the CLI which uses `anyhow` for simplicity, the `px2ansi` library
provides a structured `RenderError` enum. This allows you to programmatically
react to specific failure states.

```rust
use px2ansi::{CharsetMode, RenderError};
use std::str::FromStr;

let result = CharsetMode::from_str("invalid_mode");

match result {
    Err(RenderError::InvalidCharset(name)) => {
        eprintln!("Unsupported charset: {name}");
        assert_eq!(name, "invalid_mode");
    }
    #
    # Err(RenderError::Io(e)) => {
    #     eprintln!("A writing error occurred: {e}");
    # }
    _ => { /* handle success or other errors */ }
}
```

**Error Variants**

| **Variant**               | **Description**                                                             |
| ------------------------- | --------------------------------------------------------------------------- |
| `InvalidCharset(String)`  | Triggered when a string cannot be parsed into a valid `CharsetMode`.        |
| `InvalidDensity(String)`  | Triggered when a string cannot be parsed into a valid `Density`.            |
| `Io(std::io::Error)`      | Wrapped standard I/O errors (e.g., pipe broken, disk full).                 |
| `Image(String)`           | Errors during image manipulation or resizing.                               |
| `Font(String)`            | Errors during font loading or glyph rasterization via fontdue.              |
| `EmptyCells`              | Returned when ANSI input parses to zero cells, producing nothing to render. |
| `Json(serde_json::Error)` | Errors during JSON serialization of the image index.                        |

---

## Library vs CLI

`px2ansi` is the reusable rendering library.

If you want the command-line interface, install `px2ansi-rs` instead.

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/px2ansi-rs-png.png" alt="px2ansi-rs logo">
</p>

# px2ansi-rs

[![Crates.io](https://img.shields.io/crates/v/px2ansi-rs.svg)](https://crates.io/crates/px2ansi-rs)
[![Documentation](https://docs.rs/px2ansi-rs/badge.svg)](https://docs.rs/px2ansi-rs)
[![Nix Flake](https://img.shields.io/badge/Nix_Flake-Geared-dddd00?logo=nixos&logoColor=white)](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)
[![Nix](https://img.shields.io/badge/Nix-5277C3?style=flat&logo=nixos&logoColor=white)](https://nixos.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`px2ansi-rs` is a high-fidelity terminal art engine and asset manager. It
transforms images into terminal-native art using 10 rendering styles, from
classic ANSI blocks to high-density Braille and Kanji. With built-in indexing
and manifest support, it is designed to manage and display entire sprite
libraries with the same ease as `pokemon-colorscripts`.

Inspired by the original [px2ansi](https://github.com/Nellousan/px2ansi)
project, this is a complete reimplementation with indexing, fuzzy search, TUI
browsing, and advanced filters.

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/output.gif" width="600" alt="px2ansi-rs demo">
</p>

<details>
<summary>Original NixOS image used for conversions</summary>
<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-original.png" width="400" alt="Original NixOS Logo">
</p>
</details>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-braille.png" width="400" alt="Braille rendering example">
</p>

<details>
<summary>NixOS Kanji</summary>
<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-kanji.png" width="400" alt="NixOS Kanji">
</p>
</details>

<details>
<summary>NixOS Chinese</summary>
<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-chinese.png" width="400" alt="NixOS Chinese">
</p>
</details>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/fetcher.gif" width="600" alt="px2ansi-rs fetch demo">
</p>

<a id="top"></a>

## Table of Contents

<details>
<summary>Expand</summary>

- [Features](#features)
  - [Optional Features](#optional-features)
- [Installation](#installation)
  - [Quick Reference](#quick-reference)
- [Usage](#usage)
  - [Convert an Image](#convert-an-image)
  - [Color Modes](#color-modes)
  - [Image Rotation](#image-rotation)
  - [Create an Index](#create-an-index)
  - [Show by Name](#show-by-name)
  - [Fetch Mode](#fetch-mode)
  - [List Assets](#list-assets)
- [Configuration](#configuration)
- [Shell Completions](#shell-completions)
- [Rendering Styles](#rendering-styles)
- [Performance](#performance)
  - [Auto-Vectorization](#auto-vectorization)
  - [Benchmarks](#benchmarks)
  - [Fetch Performance](#fetch-performance)
  - [Latency Metrics](#latency-metrics)
- [Rasterize Output to PNG](#rasterize-output-to-png)
- [Using px2ansi as a Library](#-using-px2ansi-as-a-library)
- [Project Builds](#project-builds)
- [Troubleshooting](#troubleshooting--errors)
- [Similar Crates](#similar-crates)
- [Changelog](#changelog)
- [License](#license)

</details>

---

## Features

- **10 rendering styles**: `ansi`, `unicode`, `fade`, `ascii`, `braille`,
  `full-block`, `dense`, `chinese`, `kanji`, `sixel`
- **Fuzzy search**: `show pika` matches Pikachu
- **Interactive TUI**: `show -i` to browse sprites visually
- **Truecolor + transparency**: 24-bit RGB with alpha support via Oklab color
  space
- **Smart resize**: auto-fits terminal width; `--width` for manual control
- **5 resize filters**: `nearest` through `lanczos3`
- **ASCII density control**: `--density light|medium|heavy`
- **Monochrome output**: `--color-mode none` disables ANSI escapes
- **Dithering**: Floyd-Steinberg error diffusion for gradients and grayscale
- **Image rotation**: spin, flip, or mirror on x/y/z axes
- **System fetch**: display system info alongside static or rotating images
- **Sixel output**: pixel-accurate rendering for supported terminals, with OSC
  11 background detection
- **PNG rasterization**: convert ANSI output back to PNG with selectable themes
- **High-performance backend**: SIMD-accelerated pixel processing via
  auto-vectorization; optional multi-core via `rayon`

Built on top of [`px2ansi`](https://crates.io/crates/px2ansi), a standalone Rust
library exposing the full rendering engine as a public API.

### Optional Features

```bash
# Minimal — no sixel, no rasterization, no rayon
cargo install px2ansi-rs --no-default-features

# Specific features
cargo install px2ansi-rs --no-default-features --features sixel
cargo install px2ansi-rs --no-default-features --features parallel
cargo install px2ansi-rs --no-default-features --features rasterize

# Everything
cargo install px2ansi-rs --features full
```

[Back to TOC](#top)

---

## Installation

```bash
# From crates.io
cargo install px2ansi-rs

# From source
git clone https://github.com/saylesss88/px2ansi-rs
cd px2ansi-rs
cargo install --path cli
```

### Quick Reference

```text
Usage: px2ansi-rs [OPTIONS] <COMMAND>

Commands:
  convert      Convert a single image to ANSI/Unicode/Fade/Braille/Kanji/Full-block/Ascii
  index        Create a JSON index of a directory
  show         Display a sprite from the index
  list         List entries in the index
  completions  Generate shell completions and add to your shell config.
               Example: `px2ansi-rs completions bash >> ~/.bashrc`
  help         Print this message or the help of the given subcommand(s)

Options:
  -l, --latency          Show timing and execution metadata
  -I, --index <INDEX>    Path to the JSON index file (overrides config file setting)
  -h, --help             Print help (see a summary with '-h')
  -V, --version          Print version
```

Most subcommands have their own help: `px2ansi-rs convert --help`

[Back to TOC](#top)

---

## Usage

> [!NOTE]
> `px2ansi-rs` uses a subcommand-based interface: `convert`, `index`, `show`,
> and `list`. Most subcommands have their own help menus:
> `px2ansi-rs convert --help` / `px2ansi-rs show --help`

### Convert an Image

Basic conversion to stdout with automatic terminal sizing:

```bash
# Basic conversion (auto-sized to terminal width)
px2ansi-rs convert image.png
px2ansi-rs convert image.png --style unicode

# Save ANSI output to a file
px2ansi-rs convert image.png --style braille --output out.txt

# Force width and filter
px2ansi-rs convert sprite.png --width 50 --filter nearest
px2ansi-rs convert photo.png --filter lanczos3

# Show latency
px2ansi-rs convert sprite.png --width 50 --filter nearest -l

# Full-block mode (pokemon-colorscripts look)
px2ansi-rs convert image.png --style full-block --filter nearest

# ASCII with density control
px2ansi-rs convert image.png --style ascii --density light
px2ansi-rs convert image.png --style ascii --density heavy  # same as --style dense

# Monochrome and dithering
px2ansi-rs convert image.png --style ascii --color-mode none
px2ansi-rs convert image.png --style ascii --color-mode 256 --dither
```

### Color Modes

`px2ansi-rs` goes beyond simple ANSI escapes by prioritizing perceptual
accuracy. When rendering in 256-color mode it uses the **Oklab color space** for
quantization — a perceptually uniform space where equal numerical distances
correspond to equal perceived color differences. Raw pixels are linearized via a
fast lookup table to account for gamma correction before matching, so teals stay
teal and NixOS blues don't drift toward purple.

| Mode        | Description                                                     |
| ----------- | --------------------------------------------------------------- |
| `truecolor` | (Default) 24-bit ANSI sequences. Best for modern terminals.     |
| `ansi256`   | Quantizes to xterm-256 palette using Oklab perceptual matching. |
| `none`      | Disables all color escapes. Useful for piping or monochrome.    |

**Auto-detection order**

1. Checks `COLORTERM` for `truecolor` or `24bit`.
2. Inspects `TERM` for `256color` compatibility.
3. Respects the `NO_COLOR` standard — if set, all color output is disabled.

```bash
# Force 256-color mode even if truecolor is supported
px2ansi-rs convert <image> --color-mode 256

# Disable color for a monochrome ASCII look
px2ansi-rs convert <image> --color-mode none
```

### Image Rotation

```bash
# z-axis canvas spin (default)
px2ansi-rs convert skull.png --rotate

# Coin-flip on vertical axis
px2ansi-rs convert skull.png --rotate --axis y

# Cartwheel on horizontal axis
px2ansi-rs convert skull.png --rotate --axis x

# Slower spin
px2ansi-rs show skull --rotate --axis y --fps 4

# Static one-shot rotation
px2ansi-rs convert skull.png --rotate 90
px2ansi-rs convert skull.png --rotate 180

# Unidirectional (always flips the same way)
px2ansi-rs convert skull.png --rotate --axis y --unidirectional
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/newest-rotate.gif" width="600" alt="px2ansi-rs rotate demo">
</p>

### Create an Index

```bash
px2ansi-rs index ./assets/sprites --output index.json
```

If `--output` is omitted the index path falls back to the configured default (or
`index.json`).

### Show by Name

Once indexed, display an image by name without the full path:

```bash
px2ansi-rs show pikachu --style ansi
px2ansi-rs show random
px2ansi-rs show random --style unicode
px2ansi-rs show random --style ansi --filter nearest

# Equivalent to: px2ansi-rs show random
px2ansi-rs show

# Fuzzy matching — may open bulbasaur
px2ansi-rs show bul

# Interactive TUI browse
px2ansi-rs show -i
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/bul.png" width="400" alt="Bulbasaur search example">
</p>

### Fetch Mode

```bash
# Static image with system info
px2ansi-rs convert nixos.png --style ascii --fetch

# Rotating image with system info
px2ansi-rs convert skull.png --style ascii --rotate --axis y --unidirectional --fetch

# Random sprite fetch (great for shell startup)
px2ansi-rs show --fetch
px2ansi-rs show random --fetch
```

Fetch mode is terminal-width aware — the image is automatically scaled to fit
alongside the info block. On narrow terminals (e.g. a tiling WM with a
half-width pane) it falls back to a stacked layout.

**`~/fetch.conf`** — customize your fetch display:

```conf
show_hostname  = false
show_arch      = true
show_cpu       = true
show_cpu_usage = true
show_disk      = true
show_local_ip  = true
show_shell     = true

label_os       = System
label_cpu      = Processor
label_memory   = RAM
label_disk     = Storage

# Width of the left-hand label column (default 12)
key_width      = 8
```

> [!TIP]
> Layout is handled automatically. If text wraps on an unusually small pane,
> lowering `key_width` reduces the width of the info block.

**`PokéSprite` fetch setup**

```bash
git clone https://github.com/msikma/pokesprite.git
px2ansi-rs index /home/your-user/pokesprite/pokemon-gen8/shiny -o index.json
```

Add to `~/.config/px2ansi-rs/default-config.toml`:

```toml
filter = "nearest"
index  = "/home/your-user/pokesprite/pokemon-gen8/shiny/index.json"
```

Add to `.zshrc` / `.bashrc`:

```bash
px2ansi-rs show --fetch
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/fetch.png" width="400" alt="Fetch example">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/scream-fetch.png" width="400" alt="Sixel Scream Fetch example">
</p>
### List Assets

```bash
px2ansi-rs list
px2ansi-rs list --count 10

# Point at a specific index
px2ansi-rs -I /path/to/custom.json list
```

[Back to TOC](#top)

---

## Configuration

### File location

| OS      | Path                                                           |
| ------- | -------------------------------------------------------------- |
| Linux   | `~/.config/px2ansi-rs/default-config.toml`                     |
| macOS   | `~/Library/Application Support/px2ansi-rs/default-config.toml` |
| Windows | `%AppData%\px2ansi-rs\config\default-config.toml`              |

### Example `default-config.toml`

```toml
style        = "ansi"
latency      = true
filter       = "lanczos3"
index        = "/home/your-user/pokesprite/pokemon-gen8/shiny/index.json"
raster_theme = "tokyo-night"
```

### Priority

1. **CLI flags** always win.
2. **Config file** is used if no flag is provided.
3. **Built-in defaults** apply if the config file is missing.

| Setting        | Default       |
| -------------- | ------------- |
| `style`        | `ansi`        |
| `filter`       | `nearest`     |
| `latency`      | `false`       |
| `index`        | `index.json`  |
| `raster_theme` | `tokyo-night` |
| `output_image` | none          |

### NixOS configuration

```nix
home.file = {
  ".config/px2ansi-rs/default-config.toml".text = ''
    filter  = "nearest"
    latency = true
    index   = "/home/jr/pokesprite/pokemon-gen8/shiny/index.json"
  '';
};
```

[Back to TOC](#top)

---

## Shell Completions

```bash
# Zsh
source <(px2ansi-rs completions zsh)

# Bash
source <(px2ansi-rs completions bash)

# Fish
px2ansi-rs completions fish | source
```

**NixOS**

```nix
programs.zsh.initContent = ''
  if command -v px2ansi-rs >/dev/null; then
    source <(px2ansi-rs completions zsh)
  fi
'';
```

[Back to TOC](#top)

---

## Rendering Styles

| Mode       | Flag                 | Description                                | Best for                   |
| ---------- | -------------------- | ------------------------------------------ | -------------------------- |
| ANSI       | `--style ansi`       | Half-blocks (`▀▄`) — 2 pixels per row      | Compatibility and speed    |
| HD Unicode | `--style unicode`    | High-definition Unicode half-blocks        | High-fidelity assets       |
| Full Block | `--style full-block` | Solid `██` squares (double-width)          | 8-bit and 16-bit pixel art |
| Braille    | `--style braille`    | 2×4 dot patterns                           | Fine detail and line art   |
| Fade       | `--style fade`       | Block shading (`░▒▓█`)                     | High-contrast logos        |
| ASCII      | `--style ascii`      | Character-density ramp (92 chars)          | Photos and classic art     |
| Dense      | `--style dense`      | ASCII with heavy density (shorthand)       | Bold, block-heavy output   |
| Kanji      | `--style kanji`      | Japanese kanji density ramp (double-width) | Stylized output            |
| Chinese    | `--style chinese`    | Chinese density ramp (double-width)        | Stylized output            |
| Sixel      | `--style sixel`      | Pixel-accurate Sixel protocol output       | Supported terminals only   |

> [!NOTE]
> `--style ascii` supports `--density light|medium|heavy`.
> `--style dense` is shorthand for `--style ascii --density heavy`.

[Back to TOC](#top)

---

## Performance

### Auto-Vectorization

`px2ansi-rs` recently moved from hand-written explicit SIMD intrinsics to
**auto-vectorization** — and the gains were larger than expected.

**What is auto-vectorization?**

Modern CPUs can process multiple data values in a single instruction using SIMD
registers (`SSE4.2`, `AVX2` on `x86_64`; `NEON` on `ARM`). Normally you either
write intrinsics by hand (tedious, brittle, architecture-specific) or let the
compiler figure it out automatically. (that second approach is
auto-vectorization)

When code is written in a way the compiler can reason about (tight loops over
contiguous slices, no pointer aliasing, predictable access patterns) `rustc` and
LLVM will emit vectorized machine code automatically, processing 8, 16, or 32
bytes per cycle instead of one.

The pixel-processing hot path in `px2ansi-rs` (`find_luma_range_rgba_bytes` and
related routines) is exactly this shape: scanning raw `&[u8]` slices with simple
arithmetic. Restructuring those loops to be alias-free and slice-bounded gave
the compiler everything it needed to generate AVX2 or SSE4.2 code without a
single `std::arch` intrinsic.

**What does this mean for you?**

Nothing to install and nothing to configure. `cargo install px2ansi-rs` gives
you a binary already compiled for your architecture's baseline (`SSE2` on
`x86_64`). If you build from source you can tell the compiler to target your
specific CPU and unlock wider vector units:

```bash
# Target your exact CPU (enables AVX2, AVX-512, etc. if available)
RUSTFLAGS="-C target-cpu=native" cargo install --path cli

# Verify what was enabled (look for avx2, sse4.2, neon, etc.)
rustc --print cfg | grep target_feature
```

> [!NOTE]
> `target-cpu=native` produces a binary that may not run on older machines.
> Distribute without this flag; use it for personal builds or
> benchmarking.

**Pixel Processing Throughput**

The following benchmarks measure `find_luma_range_rgba_bytes`, the core
brightness-normalization scan run on every frame.

| Buffer Size | Throughput (GiB/s) | Improvement vs. explicit SIMD |
| ----------- | ------------------ | ----------------------------- |
| 256 B       | 3.44 GiB/s         | +71.7%                        |
| 1 KiB       | 3.40 GiB/s         | +67.1%                        |
| 4 KiB       | 3.50 GiB/s         | +79.0%                        |
| 64 KiB      | 3.51 GiB/s         | +84.7%                        |

**End-to-End Rendering**

| Config                   | Improvement  |
| ------------------------ | ------------ |
| Fastest / Nearest filter | ~7.8% faster |
| High-quality / Lanczos3  | ~6.4% faster |

> [!NOTE]
> Benchmarks run on `v0.3.11` with `criterion`. Results vary by CPU architecture
> and available SIMD width (SSE4.2, AVX2, NEON, etc.).

### Benchmarks

This crate uses Criterion for rigorous performance tracking. We separate our
benchmarks into two categories:

- Pixel Processing (`benches/pixels.rs`): Micro-benchmarks for the SIMD luma
  range and charset index calculation.

- End-to-End Rendering (`benches/rendering.rs`): Measures the full pipeline from
  raw bytes to ANSI/Sixel output.

**More benches with hyperfine**

Benchmarked against [`viu`](https://github.com/atanunq/viu) with
`hyperfine --warmup 3` on NixOS. Images: `nixos.png` (1183×1024) and
`scream.png` (700×909).

**Half-block rendering (`--style ansi` vs `viu --blocks`)**

| Image        | `px2ansi-rs`        | `viu -b`         | Improvement |
| ------------ | ------------------- | ---------------- | ----------- |
| `nixos.png`  | **4.4 ms** ± 0.7 ms | 13.9 ms ± 0.7 ms | 3× faster   |
| `scream.png` | **6.2 ms**          | 11.9 ms          | 1.9× faster |

User CPU time is 2.2 ms vs 10.6 ms — a ~4.8× reduction in actual compute; the
remainder is process startup and I/O.

**Sixel rendering (`--style sixel` vs `viu --static`)**

| Image        | `px2ansi-rs`(`icy_sixel`) | `viu` (`viuer`) | Difference |
| ------------ | ------------------------- | --------------- | --------------- |
| `nixos.png`  | 18.6 ms                   | 14.2 ms         | viu 4.4 ms faster         |
| `scream.png` | 24.9 ms                     | 11.9            | viu +13.0 ms faster        |

`px2ansi-rs` is slower here, and that is worth being honest about. Sixel
encoding is CPU-intensive, it involves palette quantization and bit-packing that
`viu` offloads to `libsixel`, a heavily optimized C library with over a years of
micro-optimization work behind it.

`px2ansi-rs` uses `icy_sixel`, a pure-Rust encoder. The trade-off is deliberate:

- **No FFI**, no unsafe boundary: libsixel is linked via FFI, which means a C
  toolchain dependency at build time, potential undefined behavior at the FFI
  boundary, and unsafe code that rustc cannot reason about.

- **Cross-platform without extra steps**: `icy_sixel` compiles anywhere Rust
  does. No `pkg-config`, no system `libsixel`, no Homebrew dependency.

- **Optimization headroom**: `icy_sixel` is actively developed and has not had
  the same years of profiling that libsixel has. The gap is a maturity
  difference, not a fundamental one; auto-vectorization of the quantization loop
  alone could close much of it.

If raw Sixel throughput is your priority and you are comfortable with a C
dependency, `viu` is the faster choice today. If you want a fully auditable,
dependency-minimal binary that compiles anywhere, `px2ansi-rs` is it.

**Summary**

```text

--style ansi:  3.16× faster than viu -b (nixos.png)
               1.92× faster than viu -b (scream.png)

--style sixel: 1.30× slower than viu (nixos.png)
               2.07× slower than viu (scream.png)
```

- The Encoder Bottleneck: Sixel encoding requires complex quantization and
  bit-packing. `viu` uses libsixel (optimized C), while `icy_sixel` is
  prioritizing correctness and being 100% Rust over the aggressive
  micro-optimizations found in older C libraries.

### Fetch Performance

`--fetch` is designed for shell startup. By querying only the kernel fields
actually enabled in config, startup time is kept well under 20 ms:

|           | Mean        | System time |
| --------- | ----------- | ----------- |
| Before    | 72.6 ms     | 62 ms       |
| **After** | **16.8 ms** | **13 ms**   |

> Measured with `hyperfine --warmup 3` on NixOS, Rust nightly. The remaining ~13
> ms is process startup + PNG decode.

### Latency Metrics

```bash
px2ansi-rs -l show random
px2ansi-rs convert <file> --latency
```

Latency can also be enabled via config (`latency = true`). CLI flags override
config settings.

### Parallel Rendering (`--features parallel`)

Enable multi-core rendering with `rayon`:

```bash
cargo install px2ansi-rs --features parallel
# or combined with other features
cargo install px2ansi-rs --features parallel,sixel,rasterize
```

Most noticeable on large images with `--style ascii`, `--style kanji`, or
`--style chinese`. Half-block and Braille modes see less benefit.

[Back to TOC](#top)

---

## Rasterize Output to PNG

Use `--output-image` (`-O`) to convert terminal escape codes into a `.png` file.
Requires the `rasterize` feature (enabled by default).

```bash
px2ansi-rs convert tests/nixos.png --filter nearest --style ascii \
  --output-image nixos-rasterized.png
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-rasterized.png" width="300" alt="Rasterized output example">
</p>

**Available themes:** `tokyo-night` (default), `dracula`, `nord`,
`gruvbox-dark`, `one-dark`, `solarized-dark`, `black`, `white`

```bash
px2ansi-rs convert input.png -O output.png --raster-theme dracula
px2ansi-rs convert input.png -O output.png --raster-theme nord
```

Set a default in config:

```toml
raster_theme = "gruvbox-dark"
```

> [!WARNING]
> If `rasterize` was not compiled in, `--output-image` will error and ask you
> to rebuild: `cargo install px2ansi-rs --features rasterize`

[Back to TOC](#top)

---

## 📦 Using px2ansi as a Library

`px2ansi-rs` is organized as a Cargo workspace:

- **`px2ansi`** (library) — pure rendering logic, math, and character sets.
- **`px2ansi-rs`** (CLI) — terminal flags, config files, and user interaction.

This separation keeps the library fast, minimal, and embeddable without pulling
in CLI dependencies.

See [px2ansi on crates.io](https://crates.io/crates/px2ansi) for the API.

[Back to TOC](#top)

---

## Project Builds

- [slasher-horrorscripts](https://crates.io/crates/slasher-horrorscripts)

[Back to TOC](#top)

---

## Troubleshooting & Errors

`px2ansi-rs` uses `anyhow` for error handling.

| Symptom                     | Fix                                                       |
| --------------------------- | --------------------------------------------------------- |
| **Invalid style**           | Check valid `--style` values with `--help`.               |
| **Missing file**            | `convert` on a nonexistent file fails gracefully.         |
| **Broken pipe**             | Normal when piping into `head` or similar.                |
| **Missing index**           | Ensure `index.json` exists or pass `-I <PATH>`.           |
| **Low fuzzy score**         | Use a more specific query or `-i` for interactive search. |
| **Terminal gaps**           | Your terminal line-height may be greater than `1.0`.      |
| **Rasterize not available** | Rebuild: `cargo install px2ansi-rs --features rasterize`. |

### Man Page Generation

```bash
# Generate
cargo run --bin generate-manpage

# Preview without installing
man ./man/px2ansi-rs.1

# Install for your user (no sudo)
mkdir -p ~/.local/share/man/man1/
cp man/*.1 ~/.local/share/man/man1/

# System-wide
sudo cp man/*.1 /usr/local/share/man/man1/
sudo mandb
```

> [!TIP] For faster compile times during development, add the `mold` linker to
> `~/.cargo/config.toml`:
>
> ```toml
> [target.x86_64-unknown-linux-gnu]
> rustflags = ["-C", "link-arg=-fuse-ld=mold"]
> ```

[Back to TOC](#top)

---

## Similar Crates

- [rascii_art](https://crates.io/crates/rascii_art): A well-structured, readable
  implementation. Comparing against it was helpful for spotting aspect-ratio
  issues and discovering additional charsets.
- [ansimage](https://crates.io/crates/ansimage): Untested.
- [ansizalizer](https://github.com/Zebbeni/ansizalizer): A feature-rich TUI
  built with Ansipx and Bubble Tea (Go). Polished and worth a look.

[Back to TOC](#top)

---

## Changelog

[See CHANGELOG](../CHANGELOG.md)

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

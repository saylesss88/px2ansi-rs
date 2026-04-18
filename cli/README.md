<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/px2ansi-rs-png.png" alt="px2ansi-rs logo">
</p>

# px2ansi-rs

[![Crates.io](https://img.shields.io/crates/v/px2ansi-rs.svg)](https://crates.io/crates/px2ansi-rs)
[![Documentation](https://docs.rs/px2ansi-rs/badge.svg)](https://docs.rs/px2ansi-rs)
[![Nix Flake](https://img.shields.io/badge/Nix_Flake-Geared-dddd00?logo=nixos&logoColor=white)](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)
[![Nix](https://img.shields.io/badge/Nix-5277C3?style=flat&logo=nixos&logoColor=white)](https://nixos.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`px2ansi-rs` is a high-fidelity terminal art engine and asset manager.

It transforms images into terminal-native art using 10 rendering styles, from
classic ANSI blocks to high-density Braille and Kanji. With built-in indexing
and manifest support, it is designed to manage and display entire sprite
libraries with the same ease as `pokemon-colorscripts`.

Inspired by the original [px2ansi](https://github.com/Nellousan/px2ansi)
project, this is a complete reimplementation with indexing, fuzzy search, TUI
browsing, and advanced filters. It is approximately 25x faster.

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/output.gif" width="600" alt="px2ansi-rs demo">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-braille.png" width="400" alt="Braille rendering example">
</p>

<details>
<summary> NixOS Kanji </summary>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-kanji.png" width="400" alt="NixOS Kanji">
</p>

</details>

<details>
<summary> NixOS Chinese </summary>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-chinese.png" width="400" alt="NixOS Chinese">
</p>

</details>

<a id="top"></a>

## Table of contents

- [Features](#features)
  - [Optional Features](#optional-features)
- [Installation](#installation)
  - [From Source](#from-source)
  - [From crates.io](#from-crates.io)
- [Quick reference](#quick-reference)
- [Usage](#usage)
  - [Convert an Image](#convert-an-image)
    - [Save ANSI Output to a File](#save-ansi-output-to-a-file)
    - [Unicode Mode](#unicode-mode)
    - [Force width and filtering](#force-width-and-filtering)
    - [ASCII with density control](#ascii-with-density-control)
    - [Disable Color](#disable-color)
  - [Create an Index](#create-an-index)
  - [Show by Name](#show-by-name)
    - [Quick way with Fuzzy Matching](#quick-way-with-fuzzy-matching)
    - [Interactive Search](#interactive-search)
- [Configuration](#configuration)
- [Shell completions](#shell-completions)
- [Rendering styles](#rendering-styles)
- [Performance and workflow](#performance--workflow)
- [Rasterize output to PNG](#rasterize-output-to-png)
- [Using the Library Only](#-using-px2ansi-as-a-library)
- [Project builds](#project-builds)
- [Troubleshooting](#troubleshooting--errors)
- [License](#license)

---

## Features

- **Fuzzy search** — `show pika` → Pikachu.
- **Interactive TUI** — `show -i` to browse sprites.
- **Truecolor + transparency** — Full 24-bit RGB with alpha support.
- **Smart resize** — Auto-fits terminal width.
- **Custom dimensions** — Use `--width` to adjust output size.
- **5 filters** — `nearest` for pixel art through `lanczos3` for photos.
- **10 styles** — `ansi`, `unicode`, `fade`, `ascii`, `braille`, `full-block`,
  `dense`, `chinese`, `kanji`, and `sixel`.
- **Embedded font rasterization** — `IosevkaCharonMono-Regular.ttf` is bundled
  for rasterization.
- **Optional monochrome output** — Use `--no-color` to disable ANSI color
  escapes (applies to ascii, fade, braille, kanji, and chinese modes).
- **ASCII density control** — Use `--density light|medium|heavy` to tune
  character ramp complexity.
- Optionally rasterize ANSI output back into PNG (with selectable themes).
- Optional Sixel output for terminals that support it.

- **High-Performance Backend**: SIMD-accelerated pixel processing (wide) with
  optional multi-core parallelism (rayon).

`px2ansi-rs` is built on top of [`px2ansi`](https://crates.io/crates/px2ansi), a
standalone Rust library that exposes the full rendering engine as a public API.

### Optional Features

Sixel, Rasterization, and rayon are all optional features (all enabled by
default).

```bash
# Minimal — no sixel, no rasterization, no rayon
cargo install px2ansi-rs --no-default-features

# Sixel terminal output only
cargo install px2ansi-rs --no-default-features --features sixel

# Only enable rayon and simd
cargo install px2ansi-rs --no-default-features --features parallel simd

# Only enable rasterization
cargo install px2ansi-rs --no-default-features --features rasterize

# Everything
cargo install px2ansi-rs --features full
```

---

## Installation

### From source

```bash
git clone https://github.com/saylesss88/px2ansi-rs
cd px2ansi-rs
cargo install --path cli
```

### From crates.io

```bash
cargo install px2ansi-rs
```

[Back to TOC](#top)

---

## Quick reference

```text
High-fidelity terminal art engine and asset manager

Usage: px2ansi-rs [OPTIONS] <COMMAND>

Commands:
  convert      Convert a single image to ANSI/Unicode/Fade/Braille/Kanji/Full-block/Ascii
  index        Create a JSON index of a directory
  show         Display a sprite from the index
  list         List entries in the index
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -l, --latency        Show timing and execution metadata
  -I, --index <INDEX>  Path to the JSON index file (overrides config file setting)
  -h, --help           Print help
  -V, --version        Print version
```

[Back to TOC](#top)

---

## Usage

> [!NOTE]
> `px2ansi-rs` uses a subcommand-based interface: `convert`, `index`, `show`,
> and `list`.

Most subcommands have their own help menus:

```bash
px2ansi-rs convert --help
px2ansi-rs show --help
```

### Convert an image

Basic conversion to stdout with automatic terminal sizing:

```bash
px2ansi-rs convert image.png
px2ansi-rs convert image.png --style unicode
```

#### Save ANSI output to a file

Use `--output` (`-o`) to write the rendered ANSI text to a file instead of
stdout:

```bash
px2ansi-rs convert image.png --style braille --output out.txt
```

#### Unicode mode

To get the chunky `pokemon-colorscripts` look:

```bash
px2ansi-rs convert image.png --style full-block --filter nearest
```

#### Force width and filtering

```bash
px2ansi-rs convert sprite.png --width 50 --filter nearest
```

For larger images, `lanczos3` usually looks better:

```bash
px2ansi-rs convert tests/scream.png --filter lanczos3
```

#### Sixel

```bash
px2ansi-rs convert tests/nixos.png --style sixel
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-sixel.png" width="400" alt="Braille rendering example">
</p>

#### ASCII with density control

```bash
# Default (medium) density
px2ansi-rs convert tests/test.png --style ascii --filter nearest

# Light density (sparse characters)
px2ansi-rs convert tests/test.png --style ascii --density light

# Heavy density (block-heavy ramp) — same as --style dense
px2ansi-rs convert tests/test.png --style ascii --density heavy

# Shorthand for --style ascii --density heavy
px2ansi-rs convert tests/test.png --style dense

# Monochrome ASCII
px2ansi-rs convert tests/test.png --style ascii --filter nearest --no-color
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pika-ascii2.png" width="400" alt="ASCII Pikachu example">
</p>

#### Disable color

Use `--no-color` on any conversion to strip ANSI color escapes:

```bash
px2ansi-rs convert image.png --style braille --no-color
```

### Create an index

You can create a JSON manifest of a directory full of sprites:

```bash
px2ansi-rs index ./assets/sprites --output index.json
```

If `--output` is omitted, the index path falls back to the configured default
(or `index.json`).

### Show by name

Once indexed, you can display an image by its name without needing the full
path:

```bash
px2ansi-rs show pikachu --style ansi
px2ansi-rs show random
px2ansi-rs show random --style unicode
px2ansi-rs show random --style ansi --filter nearest
```

By default (when no name is given), `px2ansi-rs show` picks a random sprite:

```bash
# Equivalent to: px2ansi-rs show random
px2ansi-rs show
```

The `show` command also supports `--style`, `--filter`, `--density`, and
`--no-color`:

```bash
px2ansi-rs show pikachu --style ascii --density light
px2ansi-rs show pikachu --style braille --no-color
```

#### Quick way with fuzzy matching

```bash
# This may open bulbasaur
px2ansi-rs show bul
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/bul.png" style="max-width: 100%; height: auto;" width="400" alt="Bulbasaur search example">
</p>

#### Interactive search

If you want to browse visually, use interactive fuzzy search:

```bash
px2ansi-rs show -i
```

### List assets

```bash
px2ansi-rs list
px2ansi-rs list --count 10
```

Example output:

```text
Index: Showing 10 of 1333 entries:
  -  abomasnow            68x56px
  -  abomasnow-mega       68x56px
  -  abra                 68x56px
  -  absol                68x56px
  -  absol-mega           68x56px
  -  accelgor             68x56px
  -  aegislash            68x56px
  -  aegislash-blade      68x56px
  -  aerodactyl           68x56px
  -  aerodactyl-mega      68x56px
```

Use `-I` to point at a specific index:

```bash
px2ansi-rs -I /path/to/custom.json list
```

[Back to TOC](#top)

---

## Configuration

`px2ansi-rs` supports a configuration file for your preferred defaults.

### File location

- **Linux**: `~/.config/px2ansi-rs/default-config.toml`
- **macOS**: `~/Library/Application Support/px2ansi-rs/default-config.toml`
- **Windows**: `%AppData%\\px2ansi-rs\\config\\default-config.toml`

### Example `default-config.toml`

```toml
# Output style: "ansi", "unicode", "fade", "ascii", "kanji", "braille",
#               "full-block", "dense", "chinese", "sixel"
style = "ansi"

# Show execution timing metadata
latency = true

# Default filter: "nearest", "triangle", "catmull-rom", "gaussian", "lanczos3"
filter = "lanczos3"

# Index file to target (absolute path recommended)
index = "/home/your-user/pokesprite/pokemon-gen8/shiny/shiny-index.json"

# Default raster theme for --output-image
raster_theme = "tokyo-night"

# Optional: auto-save a rasterized PNG alongside terminal output
# output_image = "/tmp/preview.png"
```

You can point `show` at an index anywhere in your filesystem with `-I`:

```bash
px2ansi-rs show -I /home/your-user/pokesprite/pokemon-gen8/shiny/shiny-index.json
```

> [!NOTE]
> Any field omitted from the `.toml` file falls back to the built-in defaults.

#### Configuration on NixOS

```nix
    home.file = {
      ".config/px2ansi-rs/default-config.toml".text = ''
        filter = "nearest"
        latency = true
        index = "/home/jr/pokesprite/pokemon-gen8/shiny/index.json"
      '';
    };
```

### Hierarchy of truth

1. **CLI flags** always win.
2. **Config file** is used if no flag is provided.
3. **Built-in defaults** are used if the config file is missing.

The defaults are:

| Setting        | Default       |
| -------------- | ------------- |
| `style`        | `ansi`        |
| `filter`       | `nearest`     |
| `latency`      | `false`       |
| `index`        | `index.json`  |
| `raster_theme` | `tokyo-night` |
| `output_image` | none          |

[Back to TOC](#top)

---

## Shell completions

`px2ansi-rs` can generate completion scripts for Bash, Zsh, Fish, and
PowerShell.

### Quick setup

#### Zsh

```bash
source <(px2ansi-rs completions zsh)
```

#### Bash

```bash
source <(px2ansi-rs completions bash)
```

#### Fish

```fish
px2ansi-rs completions fish | source
```

### NixOS configuration

```nix
programs.zsh.initContent = ''
  export PATH="$HOME/projects/px2ansi-rs/target/debug:$PATH"

  if command -v px2ansi-rs >/dev/null; then
    source <(px2ansi-rs completions zsh)
  fi
'';
```

[Back to TOC](#top)

---

## Rendering styles

`px2ansi-rs` supports multiple ways to bring your sprites to life.

| Mode       | Flag                 | Description                                | Best for                     |
| ---------- | -------------------- | ------------------------------------------ | ---------------------------- |
| ANSI       | `--style ansi`       | Half-blocks (`▀▄`) — 2 pixels per row      | Compatibility and speed      |
| HD Unicode | `--style unicode`    | High-definition Unicode half-blocks        | High-fidelity assets         |
| Full Block | `--style full-block` | Solid `██` squares (double-width)          | 8-bit and 16-bit pixel art   |
| Braille    | `--style braille`    | 2×4 dot patterns                           | Fine detail and line art     |
| Fade       | `--style fade`       | Block shading (`░▒▓█`)                     | High-contrast logos          |
| ASCII      | `--style ascii`      | Character-density ramp (92 chars)          | Photos and classic ASCII art |
| Dense      | `--style dense`      | ASCII with heavy density (shorthand)       | Bold, block-heavy output     |
| Kanji      | `--style kanji`      | Japanese kanji density ramp (double-width) | Stylized output              |
| Chinese    | `--style chinese`    | Chinese density ramp (double-width)        | Stylized output              |
| Sixel      | `--style sixel`      | Pixel-accurate Sixel protocol output       | Supported terminals only     |

> [!NOTE]
> `--style ascii` also supports `--density light|medium|heavy`.
> `--style dense` is shorthand for `--style ascii --density heavy`.
> `--style sixel` is basically a 1 to 1 conversion.

By default, ANSI and Unicode modes use vertical packing to maximize resolution.

[Back to TOC](#top)

---

## Performance & workflow

`px2ansi-rs` is designed for high-performance terminal environments and works
best in a "build once, show many" workflow.

### SIMD (`--features simd`)

Enables SIMD-accelerated pixel processing for faster rendering of large images.

```sh
# Build with SIMD support
cargo install px2ansi-rs --features simd

# Or build locally
cargo build --release --features simd
```

Most noticeable on large images with `--style ascii`, `--style fade`,
`--style kanji`, or `--style chinese`. Half-block and Braille modes see less
benefit.

Requires a CPU with SSE2 (all x86_64) or NEON (ARM). The `wide` crate handles
dispatch automatically. No manual configuration needed.

### Sixel (`--features sixel`)

Renders true pixel images in Sixel-compatible terminals (foot, WezTerm, iTerm2).

```sh
cargo install px2ansi-rs --features sixel
px2ansi-rs convert image.png --style sixel
```

Falls back gracefully if the terminal does not support Sixel.

### Combining Features

```sh
cargo build --release --features simd,sixel
```

<details>
<summary> Testing against rascii_art and viu </summary>

| File         | Pixels    | File size | Size/pixel        |
| ------------ | --------- | --------- | ----------------- |
| `nixos.png`  | 1,210,592 | 90KB      | 0.076 bytes/pixel |
| `scream.png` | 636,300   | 588KB     | 0.924 bytes/pixel |

`nixos.png` is 6.5x larger in pixels but 6.5x smaller on disk 

`rascii` is a well-established and fast terminal art tool. These benchmarks are
a genuine comparison against a solid baseline, not a strawman.

| Image        | Dimensions | Tool                       | User(CPU) | Total(Mean | Improvement           | Runs |
| ------------ | ---------- | -------------------------- | --------- | ---------- | --------------------- | ---- |
| `scream.png` | 700x909    | `rascii --color`           | 6.1 ms    | 10.1 ms    | -                     | 198  |
| `scream.png` | 700x909    | `px2ansi-rs --style ascii` | 4.6 ms    | 8.8 ms     | 1.3x faster CPU logic | 207  |
| `nixos.png`  | 1183x1024  | `rascii --color`           | 4.6 ms    | 10.5 ms    | -                     | 193  |
| `nixos.png`  | 1183x1024  | `px2ansi-rs --style ascii` | 2.2 ms    | 7.7 ms     | 2x faster CPU logic   | 215  |

The actuall commands compared were `rascii <image> --color`, and
`px2ansi-rs convert <image> --style ascii`

## ⚡ Benchmarks

Benchmarked against [`viu`](https://github.com/atanunq/viu): a fast,
well-established terminal image viewer built on the same `viuer` backend that
`px2ansi-rs` uses for Sixel output.

All benchmarks run with `hyperfine --warmup 3` on the same machine. Images used:
`nixos.png` (1183×1024) and `scream.png` (700×909).

---

### Half-block rendering (`--style ansi` vs `viu --blocks`)

| Image        | `px2ansi-rs`        | `viu`            | Improvement (Total) | Winner          |
| ------------ | ------------------- | ---------------- | ------------------- | --------------- |
| `nixos.png`  | **8.5 ms** ± 0.7 ms | 18.6 ms ± 0.7 ms | 2.29x faster        | `px2ansi-rs` 🏆 |
| `scream.png` | **9.3 ms** ± 0.4 ms | 15.4 ms ± 0.6 ms | 1.64x faster        | `px2ansi-rs` 🏆 |

`px2ansi-rs` renders ANSI half-blocks **2.2× faster** than `viu` on large
images. User CPU time is 2.2 ms vs 10.6 ms — a 4.8× reduction in actual compute,
with the remainder being process startup and I/O.

---

### Sixel rendering (`--style sixel` vs `viu --static`)

| Image        | `px2ansi-rs`     | `viu`                | Gap/Delta   | Winner           |
| ------------ | ---------------- | -------------------- | ----------- | ---------------- |
| `nixos.png`  | 17.7 ms ± 0.4 ms | **17.9 ms** ± 0.6 ms | +0.2 ms(🚀) | Tie/`px2ansi-rs` |
| `scream.png` | 16.5 ms ± 0.8 ms | **15.4 ms** ± 0.6 ms | -0.7 ms     | `viu`            |

Sixel encoding is CPU-bound inside the shared `viuer` encoder, both tools use
the same underlying library. `px2ansi-rs` carries ~0.6–1.1 ms of additional
overhead from process startup and image preparation before handing off to the
encoder, putting it marginally behind `viu` in this mode.

---

### Pure compute (`> /dev/null`, `nixos.png`)

Redirecting output to `/dev/null` removes terminal rendering latency and
isolates raw encode time:

| Mode        | `px2ansi-rs`             | `viu`                 | Speedup       |
| ----------- | ------------------------ | --------------------- | ------------- |
| Half-blocks | **10.4 ms** (2.5 ms CPU) | 20.4 ms (10.8 ms CPU) | **2× faster** |
| Sixel       | 21.5 ms (8.7 ms CPU)     | 20.7 ms (10.8 ms CPU) | ~equal        |

---

### Summary

```
px2ansi-rs --style ansi is the fastest benchmark overall:
  2.18× faster than viu --blocks  (nixos.png)
  1.80× faster than viu --blocks  (scream.png)
  ~equal to viu --static          (both images, Sixel)
```

> Sixel parity with `viu` is expected, both delegate encoding to the same
> `viuer` library. The ANSI/half-block gap reflects `px2ansi-rs`'s
> SIMD-accelerated luma scan and color deduplication reducing CPU work by ~4×.

</details>

### The indexing advantage

1. `index` scans your asset directory and creates a JSON manifest.
2. `show` uses the index to jump directly to the file.

### Latency metrics

Add the `-l` or `--latency` flag to show timing metrics:

```bash
px2ansi-rs -l show random
px2ansi-rs convert <file> --latency
```

> **Note**: Latency can also be enabled via the config file (`latency = true`).
> CLI flags override config settings.

### Testing with PokéSprite

```bash
git clone https://github.com/msikma/pokesprite.git
cd pokesprite/pokemon-gen8/regular
px2ansi-rs index . -o index.json -l
px2ansi-rs show random -l
```

[Back to TOC](#top)

---

## Rasterize output to PNG

Use `--output-image` (`-O`) to convert terminal escape codes into a `.png` file.
This requires the `rasterize` feature (enabled by default).

```bash
px2ansi-rs convert tests/nixos.png --filter nearest --style ascii --output-image nixos-rasterized.png
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-rasterized.png" width="300" alt="Rasterized output example">
</p>

> [!NOTE]
> Some styles look better than others. The default background theme is Tokyo
> Night.

### Choosing a theme

Use `--raster-theme` to select a background color for the rasterized PNG:

```bash
# Use default Tokyo Night theme
px2ansi-rs convert input.png -O output.png

# Use Dracula theme
px2ansi-rs convert input.png -O output.png --raster-theme dracula

# Use Nord theme
px2ansi-rs convert input.png -O output.png --raster-theme nord
```

**Available themes:** `tokyo-night` (default), `dracula`, `nord`,
`gruvbox-dark`, `one-dark`, `solarized-dark`, `black`, `white`

You can also set a default theme in your config file:

```toml
raster_theme = "gruvbox-dark"
```

> [!WARNING]
> If the `rasterize` feature is not compiled in, using `--output-image`
> will produce an error asking you to rebuild with the feature enabled.

[Back to TOC](#top)

---

## 📦 Using px2ansi as a Library

If you want to check out the `px2ansi` library, see [px2ansi](../lib)

> **Note on Project Structure**: This project is organized as a Cargo Workspace:
>
> - `px2ansi` (the library): Contains the pure rendering logic, math, and
>   character sets.
> - `px2ansi-rs` (the CLI): A frontend wrapper that handles terminal flags,
>   config files, and user interaction.

This separation ensures the library remains fast, minimal, and easy to embed in
other projects without pulling in unnecessary CLI dependencies.

---

## Project builds

- [slasher-horrorscripts](https://crates.io/crates/slasher-horrorscripts)

[Back to TOC](#top)

---

## Troubleshooting & errors

`px2ansi-rs` uses `anyhow` for error handling. Common issues:

- **Invalid style** — Using an unrecognized `--style` value will show an error
  with the list of valid options.
- **Missing file** — `convert` on a nonexistent file fails gracefully with an
  error message.
- **Broken pipe** — Happens when output is piped into a command that exits
  early, such as `head`. This is normal.
- **Missing index** — If `show` or `list` fails, ensure `index.json` exists in
  the current directory or pass `-I <PATH>`.
- **Low fuzzy score** — If a search returns no result, try a more specific query
  or use `-i`.
- **Terminal gaps** — If you see horizontal lines, your terminal line-height may
  be greater than `1.0`.
- **Rasterize not available** — If you see a message about the `rasterize`
  feature, rebuild with `cargo install px2ansi-rs --features rasterize`.

[Back to TOC](#top)

---

### 📖 Man Page Generation

The project includes a utility to generate manual pages for the primary CLI and
all subcommands using `clap_mangen`.

**Generating the files**

```bash
cargo run --bin generate-manpage
```

This will create a `man/` directory containing:

- `px2ansi-rs.1` (Main interface)
- `px2ansi-rs-build-index.1` (Subcommand specific)

**Viewing and Installation**

```bash
# Preview without installing
man ./man/px2ansi-rs.1

# Install system-wide (Linux)
sudo cp man/*.1 /usr/local/share/man/man1/
sudo mandb
```

---

## Similar crates

- [rascii_art](https://crates.io/crates/rascii_art): A well-structured, readable
  implementation. Comparing px2ansi-rs with rascii_art was especially helpful
  for spotting and fixing aspect-ratio issues in my own rendering logic, and it
  also gave me ideas for additional charsets.

- [ansimage](https://crates.io/crates/ansimage): Haven't had a chance to test
  this yet.

- [ansizalizer](https://github.com/Zebbeni/ansizalizer): A feature-rich TUI
  built with Ansipx and Bubble Tea (Go). It looks polished and could point
  toward a compelling future direction for this project.

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

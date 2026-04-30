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

<details>
<summary> Original NixOS image used for conversions </summary>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-original.png" width="400" alt="Original NixOS Logo">
</p>

</details>

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

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/fetcher.gif" width="600" alt="px2ansi-rs fetch demo">
</p>



<a id="top"></a>

## Table of contents

<details>
<summary> Table Of Contents </summary>

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
    - [Advanced Color Rendering](#advanced-color-rendering)
    - [Image Rotation](#image-rotation)
  - [Create an Index](#create-an-index)
  - [Show by Name](#show-by-name)
    - [Quick way with Fuzzy Matching](#quick-way-with-fuzzy-matching)
    - [Interactive Search](#interactive-search)
    - [Fetch Mode](#fetch-mode)
- [Configuration](#configuration)
- [Shell completions](#shell-completions)
- [Rendering styles](#rendering-styles)
- [Performance and workflow](#performance--workflow)
  - [Benchmarks](#-benchmarks)
    - [Latency Metrics](#latency-metrics)
  - [Testing with `PokéSprite`](testing-with-pokésprite)
- [Rasterize output to PNG](#rasterize-output-to-png)
  - [Choosing a Raster theme](#choosing-a-theme)
  - [Using the Library Only](#-using-px2ansi-as-a-library)
- [Project builds](#project-builds)
- [Troubleshooting](#troubleshooting--errors)
  - [Man Page Generation](#man-page-generation)
  - [Similar Crates](#similar-crates)
- [Changelog](#changelog)
- [License](#license)

</details>

---

## Features

- **Fuzzy search** — `show pika` → Pikachu.
- **Interactive TUI** — `show -i` to browse sprites.
- **Truecolor + transparency** — Full 24-bit RGB with alpha support (Oklab color
  space).
- **Smart resize** — Auto-fits terminal width.
- **Custom dimensions** — Use `--width` to adjust output size.
- **5 filters** — `nearest` for pixel art through `lanczos3` for photos.
- **10 styles** — `ansi`, `unicode`, `fade`, `ascii`, `braille`, `full-block`,
  `dense`, `chinese`, `kanji`, and `sixel`.
- **Embedded font rasterization** — `IosevkaCharonMono-Regular.ttf` is bundled
  for rasterization.
- **Optional monochrome output** — Use `--color-mode none` to disable ANSI color
  escapes (applies to ascii, fade, braille, kanji, and chinese modes).
- **ASCII density control** — Use `--density light|medium|heavy` to tune
  character ramp complexity.
- Optionally rasterize ANSI output back into PNG (with selectable themes).
- Optional Sixel output for terminals that support it.
- **High-Performance Backend**: SIMD-accelerated pixel processing (wide) with
  optional multi-core parallelism (rayon).
- Optional dithering for supported styles and images.
- Image rotation: You can spin the image, rotate the image, show a horizontal
  mirror.
- **Smart background detection** — Sixel mode queries the terminal's actual
  background color via OSC 11, so transparent image regions blend correctly
  instead of defaulting to black.
- **System Fetch**: Integrated fetch to display your system info next to your
  chosen image. Either static or rotating images work.

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
>  and `list`.

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

[Back to TOC](#top)

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

[Back to TOC](#top)


#### Sixel mode
Sixel renders pixel-accurate images in supported terminals (foot, kitty,
`WezTerm`, xterm). By default, transparent regions are passed through to the
terminal's native transparency handling.

To composite transparent pixels against your terminal's actual background
color, use `--composite-bg`. This queries the terminal via OSC 11 at render
time:
```bash
px2ansi-rs convert image.png --style sixel
px2ansi-rs convert image.png --style sixel --composite-bg
px2ansi-rs show pikachu --style sixel --composite-bg
```
If your terminal does not support OSC 11 (e.g. Windows Terminal),
`--composite-bg` has no effect and native transparency is used as the fallback.

> [!NOTE]
> Sixel requires a supporting terminal. Tested on foot and kitty.
> Ghostty works best with `background-opacity = 1.0` — semi-transparent
> backgrounds interact poorly with sixel compositing in all terminals.

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
px2ansi-rs convert tests/test.png --style ascii --filter nearest --color-mode none

# Dithering
px2ansi-rs convert tests/test.png --style ascii --filter nearest --color-mode 256 --dither
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pika-ascii2.png" width="400" alt="ASCII Pikachu example">
</p>

[Back to TOC](#top)

#### Advanced Color Rendering

`px2ansi-rs` goes beyond simple ANSI escapes by prioritizing perceptual accuracy
and terminal compatibility.

**Perceptual Quantization with Oklab**

When rendering in 256-color mode, mapping a 24-bit RGB pixel to a limited 8-bit
palette often results in "muddy" colors or incorrect brightness if using
standard Euclidean RGB distance.

This uses the **Oklab color space** for color quantization. Unlike RGB, Oklab is
perceptually uniform, meaning the numerical distance between two colors matches
how the human eye perceives difference.

- **Linear sRGB Conversion**: Raw u8 pixels are linearized using a high-
  performance lookup table (LUT) to account for gamma correction.

- **Perceptual Matching**: Colors are mapped to the xterm-256 palette by
  minimizing Delta E in the Oklab space, ensuring that teals stay teal and blues
  don't shift toward purple.

**Color Modes**

You can explicitly control the color depth using the `--color-mode` flag.

| Mode        | Description                                                                                                          |
| ----------- | -------------------------------------------------------------------------------------------------------------------- |
| `truecolor` | (Default) Uses 24-bit ANSI sequences (\x1b[38;2;R;G;Bm). Best for modern terminals (Alacritty, Kitty, iTerm2, etc.). |
| `ansi256`   | Quantizes images to the xterm-256 palette. Ideal for older terminal environments or a specific "retro" aesthetic.    |
| `none`      | Disables all ANSI color codes. Useful for piping output to text files or monochrome displays.                        |

**Intellegent Auto-Detection**

By default, `px2ansi-rs` attempts to detect the best supported mode for your
environment:

1. Checks the `COLORTERM` environment variable for `truecolor` or `24bit`.

2. Inspects `TERM` for `256color` compatibility.

3. Respects the `NO_COLOR` standard: If the `NO_COLOR` environment variable is
   set, all color output is automatically disabled.

```bash
# Force 256-color mode even if TrueColor is supported
px2ansi-rs convert <image> --color-mode 256
# Disable color for a monochrome ASCII look
px2ansi-rs convert <image> --color-mode none

# `color-mode` also works with `px2ansi-rs show`
px2ansi-rs show <image> --color-mode ...
```

> [!NOTE]
> In standard RGB space, the distance between two colors is calculated
> using the Pythagorean theorem. However, the human eye is significantly more
> sensitive to variations in Green than in Blue. If you use raw RGB distance to
> pick the "closest" 256-color match for a specific NixOS blue, the computer
> might pick a purple because, mathematically, the RGB numbers are "closer,"
> even though to a human, it looks completely wrong.

**What is Perceptual Matching?**

Perceptual matching is the process of converting colors into a Perceptually
Uniform Color Space, like Oklab, before calculating which palette color to use.

In a perceptually uniform space, a change of 0.1 in any direction (lightness,
redness, or blueness) corresponds to the same perceived change in color to a
human observer.

---

### Image Rotation

```bash
# z-axis canvas spin
px2ansi-rs convert skull.png --rotate

# Coin-flip on vertical axis — "sees the back"
px2ansi-rs convert skull.png --rotate --axis y

# Cartwheel on horizontal axis
px2ansi-rs convert skull.png --rotate --axis x

# Slower coin-flip
px2ansi-rs show skull --rotate --axis y --fps 4

# Static one-shot (axis flag ignored)
px2ansi-rs convert skull.png --rotate 90

# Static one-shot (axis flag ignored)
px2ansi-rs convert skull.png --rotate 180

# Unidirectional: always flips the same way
px2ansi-rs convert skull.png --rotate --axis y --unidirectional

# Z axis ignores --unidirectional
px2ansi-rs convert skull.png --rotate --axis z --unidirectional
```

<details>
<summary> Skull Image Used </summary>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/skull1.png" width="400" alt="Skull Before">
</p>

</details>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/newest-rotate.gif" width="600" alt="px2ansi-rs rotate demo">
</p>

### Create an index

You can create a JSON manifest of a directory full of sprites:

```bash
px2ansi-rs index ./assets/sprites --output index.json
```

If `--output` is omitted, the index path falls back to the configured default
(or `index.json`).

[Back to TOC](#top)

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

[Back to TOC](#top)

#### Interactive search

If you want to browse visually, use interactive fuzzy search:

```bash
px2ansi-rs show -i
```


#### Fetch Mode

```bash
# Static ASCII mode
px2ansi-rs convert nixos.png --style ascii --fetch
# Rotating Skull fetch
px2ansi-rs convert skull.png --style ascii --rotate --axis y --unidirectional --fetch
```

Fetch mode is terminal-width aware — the image is automatically scaled to fit
alongside the info block, and on narrow terminals (e.g. a tiling WM with a
half-width pane) it falls back to a stacked layout with the image above the
text.

**Configuring fetch settings** — place at `~/fetch.conf`:

```conf
# fetch.conf — customize your fetch display
# All fields default to true / built-in label if omitted.
show_hostname  = false   # already shown in the user@host header
show_arch      = true
show_cpu       = true
show_cpu_usage = true
show_disk      = true
show_local_ip  = true
show_shell     = true

# Rename any label
label_os       = System
label_cpu      = Processor
label_memory   = RAM
label_disk     = Storage

# Width of the left-hand label column (default 12)
key_width      = 8
```

> [!TIP]
> Layout is handled automatically — the image scales down to leave room for
> fetch text, and switches to a stacked layout if the terminal is too narrow.
> If text still wraps on an unusually small pane, lowering `key_width` reduces
> the width of the info block.


#### Random `PokéSprite `with Fetch

```bash
git clone https://github.com/msikma/pokesprite.git
## Create an Index
px2ansi-rs index /home/your-user/pokesprite/pokemon-gen8/shiny -o index.json
```

Add the index path to your config:

`~/.config/px2ansi-rs/default-config.toml`:

```toml
filter = "nearest"
index = "home/your-user/pokesprite/pokemon-gen8/shiny/index.json"
```

And finally add this to your shell config:

`.zshrc`|`.bashrc`:

```bash
# Defaults to random
px2ansi-rs show --fetch
# Or more explicitly
px2ansi-rs show random --fetch
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/fetch.png" width="400" alt="Fetch example">
</p>

---

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
`PowerShell`.

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

Requires a CPU with SSE2 (all `x86_64`) or NEON (ARM). The `wide` crate handles
dispatch automatically. No manual configuration needed.

### Sixel (`--features sixel`)

Renders true pixel images in Sixel-compatible terminals (`foot`, `WezTerm`,
`iTerm2`, `GhosTTY`).

```sh
cargo install px2ansi-rs --features sixel
px2ansi-rs convert image.png --style sixel
```

Falls back gracefully if the terminal does not support Sixel.

### Fetch Performance

`--fetch` is designed to be fast. By using `sysinfo::System::new_with_specifics`
and only querying the kernel for fields that are actually enabled in config,
startup time is kept well under 20ms:

| | mean | system time |
|---|---|---|
| before | 72.6 ms | 62 ms |
| **after** | **16.8 ms** | **13 ms** |

> Measured with `hyperfine --warmup 3` on NixOS, Rust nightly.
> The remaining ~13ms is process startup + PNG decode.

### Combining Features

```sh
cargo build --release --features simd,sixel
```

<details>
<summary> Testing with `rascii_art` and viu </summary>

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

I recently moved from `viuer` to using `icy_sixel` because it's a pure rust
implementation. `icy_sixel` is slightly slower than what `viuer` uses which is
`sixel-sys`, FFI bindings to `libsixel`, a C library.

| Image        | `px2ansi-rs`     | `viu`                | Gap/Delta   | Winner           |
| ------------ | ---------------- | -------------------- | ----------- | ---------------- |
| `nixos.png`  | 17.7 ms ± 0.4 ms | **17.9 ms** ± 0.6 ms | +0.2 ms(🚀) | Tie/`px2ansi-rs` |
| `scream.png` | 16.5 ms ± 0.8 ms | **15.4 ms** ± 0.6 ms | -0.7 ms     | `viu`            |

Sixel encoding is CPU-bound inside the shared `viuer` encoder, both tools use
the same underlying library. `px2ansi-rs` carries ~0.6–1.1 ms of additional
overhead from process startup and image preparation before handing off to the
encoder, putting it marginally behind `viu` in this mode.

---

### 🎨 Dithering (--dither)

The `--dither` flag enables **Floyd-Steinberg error diffusion**. This technique
approximates shades and gradients that aren't natively available in your current
character set or color mode.

**When to use it**

Dithering is most useful when you are reducing the "color depth" of an image. It
replaces solid blocks of characters with "stippled" patterns that trick the eye
into seeing smoother transitions.

| Scenario        | Without Dithering                                   | With Dithering                                              |
| --------------- | --------------------------------------------------- | ----------------------------------------------------------- |
| Grayscale/ASCII | Harsh "banding" in shadows and skin tones           | Smooth gradients; looks like a high-detail newspaper print. |
| Flat Logos      | Clean, solid colors.                                | Can look "noisy" or "grainy" (Usually better off)           |
| Photographs     | Details can get lost in solid blocks of characters. | Retains "optical depth" and fine textures.                  |

**Supported Styles & Modes**

- **Grayscale** (`--color-mode none`): Highly Recommended. This is where
  dithering shines. It uses the density of your characters (e.g., `@%#*+=-:. `)
  to simulate shades of gray.

- **Color-Preserving**: Even in color modes, `px2ansi-rs` uses a
  Luminance-Remapped Dither. It calculates the dither map to determine
  brightness and then scales the original RGB values to match, preserving the
  "hue" of your image while adding high-frequency detail.

**Usage Example**

```bash
# Convert a portrait to high-detail grayscale ASCII
px2ansi-rs convert face.jpg --style ascii --color-mode none --dither

# Enhance a colored landscape with a dithered "sparkle"
px2ansi-rs convert view.png --style unicode --dither
```

Braille Style: Works well to create a "halftone" effect using the high-density
dots of the Braille character set.

### Pure compute (`> /dev/null`, `nixos.png`)

Redirecting output to `/dev/null` removes terminal rendering latency and
isolates raw encode time:

| Mode        | `px2ansi-rs`             | `viu`                 | Speedup       |
| ----------- | ------------------------ | --------------------- | ------------- |
| Half-blocks | **10.4 ms** (2.5 ms CPU) | 20.4 ms (10.8 ms CPU) | **2× faster** |
| Sixel       | 21.5 ms (8.7 ms CPU)     | 20.7 ms (10.8 ms CPU) | ~equal        |

---

### Summary

```text
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

### Testing with `PokéSprite`

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
> If the `rasterize` feature is not compiled in, using `--output-image` will
> produce an error asking you to rebuild with the feature
> enabled.

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

# 1. Preview without installing (works for local files)
man ./man/px2ansi-rs.1

# 2. Install for just your user (No `sudo` needed!)
mkdir -p ~/.local/share/man/man1/
cp man/*.1 ~/.local/share/man/man1/

# 3. Install system-wide (requires `sudo`)
sudo mkdir -p /usr/local/share/man/man1/
sudo cp man/*.1 /usr/local/share/man/man1/
sudo mandb
```

Now you can access the manual pages with:

```bash
man px2ansi-rs
man 1 px2ansi-rs-show
```

---

### Dev Tips

> [!TIP]
> For faster compile times during development, you can use the `mold` linker
> by adding this to your local `~/.cargo/config.toml`:
>
> ```toml
>  [target.x86_64-unknown-linux-gnu]
>  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
> ```
>
> This requires `mold` to be installed

## Similar crates

- [rascii_art](https://crates.io/crates/rascii_art): A well-structured, readable
  implementation. Comparing `px2ansi-rs` with `rascii_art` was especially
  helpful for spotting and fixing aspect-ratio issues in my own rendering logic,
  and it also gave me ideas for additional charsets.

- [ansimage](https://crates.io/crates/ansimage): Haven't had a chance to test
  this yet.

- [ansizalizer](https://github.com/Zebbeni/ansizalizer): A feature-rich TUI
  built with Ansipx and Bubble Tea (Go). It looks polished and could point
  toward a compelling future direction for this project.

## Changelog

- [See Changelog](../CHANGELOG.md)

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

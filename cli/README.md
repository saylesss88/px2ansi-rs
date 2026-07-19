# px2ansi-rs

[![Crates.io](https://img.shields.io/crates/v/px2ansi-rs.svg)](https://crates.io/crates/px2ansi-rs)
[![Documentation](https://docs.rs/px2ansi-rs/badge.svg)](https://docs.rs/px2ansi-rs)
[![Nix Flake](https://img.shields.io/badge/Nix_Flake-Geared-dddd00?logo=nixos&logoColor=white)](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)
[![Nix](https://img.shields.io/badge/Nix-5277C3?style=flat&logo=nixos&logoColor=white)](https://nixos.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`px2ansi-rs` is a high-fidelity terminal image renderer and asset manager. It
converts images into terminal-native art using 10 rendering styles, from classic
ANSI blocks to high-density Braille and Kanji. With built-in indexing, fuzzy
search, and TUI browsing for managing entire sprite libraries.

Inspired by [px2ansi](https://github.com/Nellousan/px2ansi); this is a complete
Rust reimplementation.

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/output.gif" width="600" alt="px2ansi-rs demo">
</p>

<details>
<summary>Original NixOS image</summary>
<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-original.png" width="400" alt="Original NixOS Logo">
</p>
</details>

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


<a id="top"></a>

<details>
<summary>Table of Contents</summary>

- [Features](#features)
- [Installation](#installation)
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
- [Rasterize Output to PNG](#rasterize-output-to-png)
- [Using px2ansi as a Library](#using-px2ansi-as-a-library)
- [Troubleshooting](#troubleshooting)
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
- **Truecolor + transparency**: 24-bit RGB with true alpha via Oklab color space
- **Smart resize**: auto-fits terminal width; `--width` for manual control
- **5 resize filters**: `nearest` through `lanczos3`
- **ASCII density control**: `--density light|medium|heavy`
- **Monochrome output**: `--color-mode none`
- **Dithering**: Floyd-Steinberg error diffusion
- **Image rotation**: spin, flip, or mirror on x/y/z axes
- **Fetch mode**: display system info alongside static or rotating images
- **Sixel output**: pixel-accurate rendering with true alpha and OSC 11
  background detection
- **PNG rasterization**: convert ANSI output back to PNG with selectable themes
- **Auto-vectorized backend**: SIMD pixel processing via LLVM
  auto-vectorization; optional multi-core via `rayon`

Built on [`px2ansi`](https://crates.io/crates/px2ansi), a standalone library
exposing the full rendering engine as a public API.

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
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -l, --latency          Show timing and execution metadata
  -I, --index <INDEX>    Path to the JSON index file (overrides config)
  -h, --help             Print help
  -V, --version          Print version
```


---

## Usage

> [!NOTE]
> `px2ansi-rs` uses a subcommand-based interface: `convert`, `index`, `show`,
> and `list`.

### Convert an Image

```bash
# Basic (auto-sized to terminal width)
px2ansi-rs convert image.png
px2ansi-rs convert image.png --style unicode

# Save output to file
px2ansi-rs convert image.png --style braille --output out.txt

# Width and filter
px2ansi-rs convert sprite.png --width 50 --filter nearest
px2ansi-rs convert photo.png --filter lanczos3

# Full-block mode (pokemon-colorscripts look)
px2ansi-rs convert image.png --style full-block --filter nearest

# ASCII with density
px2ansi-rs convert image.png --style ascii --density light
px2ansi-rs convert image.png --style ascii --density heavy

# Monochrome and dithering
px2ansi-rs convert image.png --style ascii --color-mode none
px2ansi-rs convert image.png --style ascii --color-mode 256 --dither
```

**Getting help**

- `px2ansi-rs --help`
- `px2ansi-rs <subcommand> --help`
- For shell completions: `px2ansi-rs completions zsh|bash|fish`

### Color Modes

When rendering in 256-color mode, `px2ansi-rs` quantizes using the **Oklab color
space** perceptually uniform, gamma-corrected via lookup table so colors stay
accurate instead of drifting.

| Mode        | Description                                                    |
| ----------- | -------------------------------------------------------------- |
| `truecolor` | (Default) 24-bit ANSI sequences                                |
| `ansi256`   | Quantizes to xterm-256 palette using Oklab perceptual matching |
| `none`      | Disables all color escapes                                     |

Auto-detection checks `COLORTERM`, then `TERM`, then respects `NO_COLOR`.

```bash
px2ansi-rs convert <image> --color-mode 256
px2ansi-rs convert <image> --color-mode none
```

### Image Rotation

```bash
px2ansi-rs convert skull.png --rotate           # z-axis spin (default)
px2ansi-rs convert skull.png --rotate --axis y  # flip on vertical axis
px2ansi-rs convert skull.png --rotate --axis x  # flip on horizontal axis
px2ansi-rs show skull --rotate --axis y --fps 4 # slower spin
px2ansi-rs convert skull.png --rotate 90        # static one-shot
px2ansi-rs convert skull.png --rotate --axis y --unidirectional
# shorthand for the above command
 px2ansi-rs convert skull.png -r -a y -u
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/newest-rotate.gif" width="600" alt="px2ansi-rs rotate demo">
</p>

### Create an Index

```bash
px2ansi-rs index ./assets/sprites --output index.json
```

### Show by Name

```bash
px2ansi-rs show pikachu --style ansi
px2ansi-rs show random
px2ansi-rs show bul          # fuzzy match
px2ansi-rs show -i           # interactive TUI
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/bul.png" width="400" alt="Bulbasaur search example">
</p>

### Fetch Mode

```bash
px2ansi-rs convert nixos.png --style ascii --fetch
px2ansi-rs convert skull.png --style ascii --rotate --axis y --fetch
px2ansi-rs show --fetch
px2ansi-rs show random --fetch
```

Layout is terminal-width aware; falls back to stacked on narrow panes.

**`~/fetch.conf`**

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
key_width      = 8
```

**`PokéSprite` setup**

```bash
git clone https://github.com/msikma/pokesprite.git
px2ansi-rs index /home/your-user/pokesprite/pokemon-gen8/shiny -o index.json
```

`~/.config/px2ansi-rs/default-config.toml`:

```toml
filter = "nearest"
index  = "/home/your-user/pokesprite/pokemon-gen8/shiny/index.json"
```

`.zshrc` / `.bashrc`:

```bash
px2ansi-rs show --fetch
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/fetch.png" width="400" alt="Fetch example">
</p>

### List Assets

```bash
px2ansi-rs list
px2ansi-rs list --count 10  # Or px2ansi-rs list -c 10
px2ansi-rs -I /path/to/custom.json list
```

---

## Configuration

| OS      | Path                                                           |
| ------- | -------------------------------------------------------------- |
| Linux   | `~/.config/px2ansi-rs/default-config.toml`                     |
| macOS   | `~/Library/Application Support/px2ansi-rs/default-config.toml` |
| Windows | `%AppData%\px2ansi-rs\config\default-config.toml`              |

```toml
style        = "ansi"
latency      = true
filter       = "lanczos3"
index        = "/home/your-user/pokesprite/pokemon-gen8/shiny/index.json"
raster_theme = "tokyo-night"
```

CLI flags > config file > built-in defaults.

| Setting        | Default       |
| -------------- | ------------- |
| `style`        | `ansi`        |
| `filter`       | `nearest`     |
| `latency`      | `false`       |
| `index`        | `index.json`  |
| `raster_theme` | `tokyo-night` |

**NixOS**

```nix
home.file.".config/px2ansi-rs/default-config.toml".text = ''
  filter  = "nearest"
  latency = true
  index   = "/home/jr/pokesprite/pokemon-gen8/shiny/index.json"
'';
```

---

## Shell Completions

<details>
<summary> 🐚 Shell Completions </summary>

```bash
source <(px2ansi-rs completions zsh)
source <(px2ansi-rs completions bash)
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

</details>

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
| Dense      | `--style dense`      | ASCII heavy density shorthand              | Bold, block-heavy output   |
| Kanji      | `--style kanji`      | Japanese kanji density ramp (double-width) | Stylized output            |
| Chinese    | `--style chinese`    | Chinese density ramp (double-width)        | Stylized output            |
| Sixel      | `--style sixel`      | Pixel-accurate sixel protocol output       | Supported terminals only   |

> [!NOTE]
> `--style ascii` supports `--density light|medium|heavy`. `--style dense` is
> shorthand for `--style ascii --density heavy`.

---

### Parallel Rendering (`--features parallel`)

```bash
cargo install px2ansi-rs --features parallel,sixel,rasterize
```

Most noticeable on large images with `--style ascii`, `--style kanji`, or
`--style chinese`.

---

## Rasterize Output to PNG

Requires the `rasterize` feature (enabled by default).

```bash
px2ansi-rs convert tests/nixos.png --filter nearest --style ascii \
  --output-image nixos-rasterized.png
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-rasterized.png" width="300" alt="Rasterized output example">
</p>

**Themes:** `tokyo-night` (default), `dracula`, `nord`, `gruvbox-dark`,
`one-dark`, `solarized-dark`, `black`, `white`

```bash
px2ansi-rs convert input.png -O output.png --raster-theme dracula
```

```toml
raster_theme = "gruvbox-dark"
```

> [!WARNING]
> If `rasterize` was not compiled in, `--output-image` will error:
> `cargo install px2ansi-rs --features rasterize`

---

## Using px2ansi as a Library

`px2ansi-rs` is a Cargo workspace:

- **`px2ansi`** pure rendering logic, math, and character sets
- **`px2ansi-rs`** CLI, config files, and user interaction

See [px2ansi on crates.io](https://crates.io/crates/px2ansi).

---

## Troubleshooting

| Symptom                     | Fix                                                      |
| --------------------------- | -------------------------------------------------------- |
| **Invalid style**           | Check valid `--style` values with `--help`               |
| **Missing file**            | `convert` on a nonexistent file fails gracefully         |
| **Broken pipe**             | Normal when piping into `head` or similar                |
| **Missing index**           | Ensure `index.json` exists or pass `-I <PATH>`           |
| **Low fuzzy score**         | Use a more specific query or `-i` for interactive search |
| **Terminal gaps**           | Your terminal line-height may be greater than `1.0`      |
| **Rasterize not available** | Rebuild: `cargo install px2ansi-rs --features rasterize` |

### Man Page

```bash
cargo run --bin generate-manpage
man ./man/px2ansi-rs.1

# Install (no sudo)
mkdir -p ~/.local/share/man/man1/
cp man/*.1 ~/.local/share/man/man1/
```

> [!TIP]
>  For faster compile times during development, add `mold` to
> `~/.cargo/config.toml`:
>
> ```toml
> [target.x86_64-unknown-linux-gnu]
> rustflags = ["-C", "link-arg=-fuse-ld=mold"]
> ```

---

## Similar Crates

- [viu](https://crates.io/crates/viu)
- [rascii_art](https://crates.io/crates/rascii_art): well-structured
  implementation; useful reference for aspect-ratio and charset design
- [ansimage](https://crates.io/crates/ansimage)
- [ansizalizer](https://github.com/Zebbeni/ansizalizer): feature-rich TUI in Go,
  worth a look

---

## Changelog

[See CHANGELOG](../CHANGELOG.md)

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

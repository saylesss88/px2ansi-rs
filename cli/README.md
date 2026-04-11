<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/px2ansi-rs-png.png" alt="px2ansi-rs logo">
</p>

# px2ansi-rs

[![Crates.io](https://img.shields.io/crates/v/px2ansi.svg)](https://crates.io/crates/px2ansi-rs)
[![Documentation](https://docs.rs/px2ansi/badge.svg)](https://docs.rs/px2ansi-rs)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

`px2ansi-rs` is a high-fidelity terminal art engine and asset manager.

It transforms images into terminal-native art using 9 rendering styles, from
classic ANSI blocks to high-density Braille and Kanji.
With built-in indexing and manifest support, it is designed to manage and
display entire sprite libraries with the same ease as `pokemon-colorscripts`.

Inspired by the original [px2ansi](https://github.com/Nellousan/px2ansi)
project, this is a complete reimplementation with indexing, fuzzy search, TUI
browsing, and advanced filters. It is approximately 25x faster.

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/output.gif" width="600" alt="px2ansi-rs demo">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/nixos-braille.png" width="400" alt="Braille rendering example">
</p>

<a id="top"></a>

## Table of contents


- [Features](#features)
- [Installation](#installation)
- [Quick reference](#quick-reference)
- [Usage](#usage)
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
- **8 styles** — `ansi`, `unicode`, `fade`, `ascii`, `braille`, `full-block`,
  `chinese`, and `kanji`.
- **Embedded font rasterization** — `IosevkaCharonMono-Regular.ttf` is bundled
  for rasterization.
- **Optional monochrome output** — Use `--no-color` where it makes sense.
- Optionally rasterize ANSI output back into PNG
- Optional Sixel output for terminals that support it

`px2ansi-rs` is built on top of [`px2ansi`](https://crates.io/crates/px2ansi), 
a standalone Rust library that exposes the full rendering engine as a public
API.

---

## Installation

### From source

```bash
git clone https://github.com/saylesss88/px2ansi-rs
cd px2ansi-rs
cargo install --path .
```

### From crates.io

```bash
cargo install px2ansi-rs
```

[Back to TOC](#top)

---

## Quick reference

```text
Pixel art tools

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
  -I, --index <INDEX>
  -h, --help           Print help
  -V, --version        Print version
```

[Back to TOC](#top)

---

## Usage

> [!NOTE]
> `px2ansi-rs` uses a subcommand-based interface: `convert`, `index`, `show`,
> and `list`.

Most subcommands also have their own help menus:

```bash
px2ansi-rs show --help
```

### 1. Convert an image

Basic conversion to stdout with automatic terminal sizing:

```bash
px2ansi-rs convert image.png
px2ansi-rs convert image.png --style unicode
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

Simple ASCII output:

```bash
px2ansi-rs convert tests/test.png --style ascii --filter nearest
px2ansi-rs convert tests/test.png --style ascii --filter nearest --no-color
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pika-ascii.png" width="400" alt="ASCII Pikachu example">
</p>

### 2. Create an index

You can create a JSON manifest of a directory full of sprites:

```bash
px2ansi-rs index ./assets/sprites --output index.json
```

### 3. Show by name

Once indexed, you can display an image by its name without needing the full
path:

```bash
px2ansi-rs show pikachu --style ansi
px2ansi-rs show random
px2ansi-rs show random --style unicode
px2ansi-rs show random --style ansi --filter nearest
```

By default, `px2ansi-rs show` displays a random sprite from `index.json` in the
current folder.

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

### 4. List assets

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
# Output style: "ansi", "unicode", "fade", "ascii", "kanji", "braille", "full-block"
style = "ansi"

# Show execution timing metadata
latency = true

# Default filter: "nearest", "triangle", "catmull-rom", "gaussian", "lanczos3"
filter = "lanczos3"

# Index file to target
index = "/home/your-user/pokesprite/pokemon-gen8/shiny/shiny-index.json"
```

You can point `show` at an index anywhere in your filesystem with `-I`:

```bash
px2ansi-rs show -I /home/your-user/pokesprite/pokemon-gen8/shiny/shiny-index.json
```

> [!NOTE]
> Any field omitted from the `.toml` file falls back to the built-in defaults.

### Hierarchy of truth

1. **CLI flags** always win.
2. **Config file** is used if no flag is provided.
3. **Built-in defaults** are used if the config file is missing.

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

| Mode       | Flag                 | Description                           | Best for                     |
| ---------- | -------------------- | ------------------------------------- | ---------------------------- |
| ANSI       | `--style ansi`       | Half-blocks (`▀▄`) — 2 pixels per row | Compatibility and speed      |
| HD Unicode | `--style unicode`    | High-definition Unicode half-blocks   | High-fidelity assets         |
| Full Block | `--style full-block` | Solid `██` squares                    | 8-bit and 16-bit pixel art   |
| Braille    | `--style braille`    | 2×4 dot patterns                      | Fine detail and line art     |
| Fade       | `--style fade`       | Block shading (`░▒▓█`)                | High-contrast logos          |
| ASCII      | `--style ascii`      | Character-density ramp                | Photos and classic ASCII art |
| Kanji      | `--style kanji`      | Japanese kanji density ramp           | Stylized output              |
| Chinese    | `--style chinese`    | Chinese density ramp                  | Stylized output              |

> [!NOTE]
> `--style ascii` also supports `--density light|medium|heavy`. `--style dense`
> is shorthand for `--style ascii --density heavy`.

By default, ANSI and Unicode modes use vertical packing to maximize resolution.

[Back to TOC](#top)

---

## Performance & workflow

`px2ansi-rs` is designed for high-performance terminal environments and works
best in a “build once, show many” workflow.

### The indexing advantage

1. `index` scans your asset directory and creates a JSON manifest.
2. `show` uses the index to jump directly to the file.

### Latency metrics

Add the `-l` or `--latency` flag to show timing metrics:

```bash
px2ansi-rs -l show random
px2ansi-rs convert <file> --latency
```

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

Use `--output-image` to convert terminal escape codes into a `.png` file:

```bash
px2ansi-rs convert tests/nixos.png --filter nearest --style ascii --output-image nixos-rasterized.png
```

<p align="center">
  <img src="https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/px-rasterize.png" width="300" alt="Rasterized output example">
</p>

> [!NOTE]
> Some styles look better than others. The default background is Tokyo Night,
> and that may change in the future.

```bash
# Use default Tokyo Night theme
px2ansi-rs convert input.png -O output.png

# Use Dracula theme
px2ansi-rs convert input.png -O output.png --raster-theme dracula

# Use Nord theme
px2ansi-rs convert input.png -O output.png --raster-theme nord

# Set default in config (~/.config/px2ansi-rs/default-config.toml)
raster_theme = "gruvbox-dark"
```

[Back to TOC](#top)

---

## 📦 Using px2ansi as a Library

If you want to check out the `px2ansi` library, see [px2ansi](../lib)

> **Note on Project Structure**: This project is organized as a Cargo Workspace:
>
> - `px2ansi` (the library): Contains the pure rendering logic, math, and
> character sets.
>
> - `px2ansi-rs` (the CLI): A frontend wrapper that handles terminal flags,
> config files, and user interaction.

This separation ensures the library remains fast, minimal, and easy to embed in
other projects without pulling in unnecessary CLI dependencies.

---

## Project builds

- [slasher-horrorscripts](https://crates.io/crates/slasher-horrorscripts)

[Back to TOC](#top)

---

## Troubleshooting & errors

`px2ansi-rs` uses `anyhow` for error handling. Common issues:

- **Broken pipe** — Happens when output is piped into a command that exits
  early, such as `head`. This is normal.
- **Missing index** — If `show` fails, ensure `index.json` exists in the current
  directory or pass `--index <PATH>`.
- **Low fuzzy score** — If a search returns no result, try a more specific query
  or use `-i`.
- **Terminal gaps** — If you see horizontal lines, your terminal line-height may
  be greater than `1.0`.

[Back to TOC](#top)

---

### 📖 Man Page Generation

The project includes a utility to generate manual pages for the primary CLI and all subcommands (like build-index) using clap_mangen.

**Generating the files**
To generate the `.1` roff files, run the included `generate-manpage` binary:

```bash
cargo run --bin generate-manpage
# Or once px2ansi-rs is installed simply:
generate-manpage
```

This will create a `man/` directory in your project root containing:

- `px2ansi-rs.1` (Main interface)

- `px2ansi-rs-build-index.1` (Subcommand specific)

**Viewing and Installation**
You can preview the generated pages without installing them:

```bash
man ./man/px2ansi-rs.1
```

To make them available system-wide (on most Linux distributions):

```bash
# Copy to your local manpath
sudo cp man/*.1 /usr/local/share/man/man1/

# Update the man database
sudo mandb
```

---

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)


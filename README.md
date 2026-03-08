# px2ansi

A high-performance Rust port of [px2ansi](https://github.com/Nellousan/px2ansi).

**px2ansi** converts pixel art images into ANSI escape codes for display in
modern terminals. It supports both ANSI half-blocks for high-density rendering
and Unicode full-blocks for a retro, "colorscript" style.

It is significantly faster than the original Python implementation and ships as
a single, static binary.

(Input)
![screenshot1](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pik1.png)

(Output)
![screenshot2](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pikaclean.png)

> If you see horizontal lines or gaps in the image, check your terminal's Line
> Height or Vertical Offset settings. For the best experience, set line-height
> to 1.0 and use a "Mono" spaced font. Terminals like foot, kitty, and wezterm
> generally provide the best results out of the box.

## Features

- 🚀 **Fast**: Written in Rust, optimized for speed (~25x faster than Python).

- 🎨 **Truecolor**: Supports full 24-bit RGB colors.

- 📐 Smart Resizing: Automatically detects terminal width and resizes large
  images to fit.

- 📂 JSON Indexing: Built-in tool to scan directories and generate a searchable
  manifest of your art library.

- 🖼️ Flexible Filtering: Use `nearest` for sharp pixel art or `lanczos3` for
  fmooth photos.

- 🧩 Transparency: Correctly handles alpha channels (rendering transparent
  pixels as terminal background).

---

## Installation

### From Source

```bash
git clone https://github.com/saylesss88/px2ansi-rs
cd px2ansi-rs
cargo install --path .
```

`crates.io`

```bash
cargo install px2ansi-rs
```

## Usage

`px2ansi-rs` now uses a subcommand-based interface: `convert`, `index`, and
`show`

1. Convert an Image

Basic conversion to stdout (auto-resizes to fit your terminal):

```Bash
px2ansi-rs convert image.png
```

**Unicode Mode** (Retro Style)

To get the chunky "Pokemon Colorscript" look:

```Bash
px2ansi-rs convert image.png --mode unicode --filter nearest
```

**Force Width & Filtering**

```Bash
px2ansi-rs convert sprite.png --width 50 --filter nearest
```

For bigger images `lanczos3` seems to look better:

```bash
px2ansi-rs convert tests/scream.png --filter lanczos3
```

2. The Library Indexer

You can create a JSON manifest of a directory full of sprites. This is useful
for building art collections or scripts.

```Bash
px2ansi-rs index ./assets/sprites --output index.json
```

3. Show by Name

Once indexed, you can display an image by its name (file stem) without needing
the full path:

```Bash
px2ansi-rs show pikachu --mode ansi
# Show a random sprite from your index
px2ansi-rs show random
px2ansi-rs show random --mode unicode
px2ansi-rs show random --mode ansi --filter nearest
```

If you clone the repo, I've included some test `.png` files:

```bash
git clone https://github.com/saylesss88/px2ansi-rs
cd px2ansi-rs
px2ansi-rs convert tests/test.png --filter nearest
# Create an index
px2ansi-rs index tests -o index.json
px2ansi-rs show random
px2ansi-rs show scream --filter lanczos3
```

---

## Resize Filters (`--filter`)

- `nearest` — Nearest-neighbor. Fastest; best for pixel art / hard edges.
- `triangle` — Linear filter (bilinear).
- `catmull-rom` — Cubic filter.
- `gaussian` — Gaussian filter.
- `lanczos3` — Lanczos filter (window 3). Default.

## Example Project build with px2ansi-rs

- [slasher-horrorscripts](https://crates.io/crates/slasher-horrorscripts)

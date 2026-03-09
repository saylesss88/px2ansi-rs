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

## ⚡ Performance & Workflow

`px2ansi-rs` is designed for high-performance terminal environments. While it
can convert images on the fly, it is optimized for a "Build Once, Show Many"
workflow.

By default, the latency timer is visible. To suppress it for a cleaner output,
use the `-s` or `--silent` flag. **The Indexing Advantage**

### The Indexing Advantage

Standard image-to-ANSI tools must decode, resize, and re-calculate ANSI escape
sequences every time they are run. `px2ansi-rs` separates these concerns:

1. `index`: Scans your asset directory and creates a JSON manifest. This avoids
   slow recursive directory walks during daily use.

2. `show`: Uses the index to jump directly to the file. When combined with the
   `--silent` flag, this provides an "instant-on" experience suitable for shell
   startup scripts (`.zshrc`, `config.nu`).

**Benchmarking**

System: AMD AM06 Pro (Ryzen) | OS: NixOS

**Benchmarking Targets:**

- **Sprite (test.png):** 96x96 (~9k pixels) -> 2ms (Nearest)
- **High-Fi (scream.png):** 700x909 (~636k pixels) -> 15ms (Nearest) / 17ms
  (Lanczos3)

Performance is divided into two categories: Sprites (low resolution/nearest
filter) and High-Fidelity (high resolution/complex filters).

The following measurements reflect the performance of the tool in a real-world
environment using a release build (`opt-level = 3`).

| Operation         | Target Asset        | Filter   | Latency |
| ----------------- | ------------------- | -------- | ------- |
| Convert           | `test.png`          | Nearest  | 3ms     |
| Convert           | `scream.png`(Hi-Fi) | Lanczos3 | 15ms    |
| Convert           | `scream.png`(Hi-Fi) | Nearest  | 9ms     |
| Summon(`show`)    | `test`(96x96)       | Nearest  | 2ms     |
| Summon(`show`)    | `test`(Unicode)     | Nearest  | 0ms     |
| Summon(`show`)    | `scream`            | Nearest  | 12ms    |
| Manifest(`index`) | 2-Asset Test        | N/A      | 7ms     |

> Note: While Lanczos3 provides the highest visual quality, it is mathematically
> intensive. For shell greetings, using the show command with pre-indexed
> sprites is recommended for a sub-10ms "instant" feel.

Silent Mode For use in automation or terminal greetings, use the `-s` or
`--silent` flag to suppress performance metrics and output only the raw ANSI
art:

```Bash
# Don't show latency on screen
px2ansi -s show random
px2ansi convert <file> --silent
```

---

## Resize Filters (`--filter`)

- `nearest` — Nearest-neighbor. Fastest; best for pixel art / hard edges.
- `triangle` — Linear filter (bilinear).
- `catmull-rom` — Cubic filter.
- `gaussian` — Gaussian filter.
- `lanczos3` — Lanczos filter (window 3). Default.

- [guide.encode.moe resampling](https://guide.encode.moe/encoding/resampling.html)

## Example Project build with px2ansi-rs

- [slasher-horrorscripts](https://crates.io/crates/slasher-horrorscripts)

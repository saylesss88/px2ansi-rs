# px2ansi

A high-performance Rust port of [px2ansi](https://github.com/Nellousan/px2ansi).

**px2ansi** converts pixel art images into ANSI escape codes for display in
modern terminals. It uses 24-bit truecolor and unicode half-block characters
(`▀` / `▄`) to render images with precision.

It is significantly faster than the original Python implementation and ships as
a single, static binary.

## Features

- 🚀 **Fast**: Written in Rust, optimized for speed (~25x faster than Python).
- 🎨 **Truecolor**: Supports full 24-bit RGB colors.
- 🧩 **Transparency**: Correctly handles alpha channels (rendering transparent
  pixels as terminal background).
- 📦 **Simple**: Single binary, no dependencies required at runtime.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/px2ansi-rs
cd px2ansi-rs
cargo install --path .
```

`crates.io`

```bash
cargo install px2ansi-rs
```

## Usage

CLI

Convert an image and print to stdout:

```bash
px2ansi path/to/image.png
```

Save output to a file:

```bash
px2ansi image.png -o art.txt
```

Try it out!

You can test it right now with the included `test.png` (a small pixel art
example):

```bash
px2ansi tests/test.png
```

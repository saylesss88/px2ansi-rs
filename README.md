# px2ansi-rs

A high-performance Rust port of [px2ansi](https://github.com/Nellousan/px2ansi).

`px2ansi-rs` is a high-performance image-to-ANSI/image-to-Unicode converter
written in Rust. While inspired by the original Python px2ansi project, this is
a complete reimagining built from the ground up for speed, featuring a dedicated
indexing system, advanced resampling filters, Unicode support, fuzzy search and
more.

It is significantly faster than the original Python implementation and ships as
a single, static binary.

(Input)
![screenshot1](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pik1.png)

(Output)
![screenshot2](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/pikaclean.png)

(Input Hi-Fi)
![scream](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/scream.png)

(Output Hi-Fi source: pngegg.com)
![scream](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/scream-demo.png)

> If you see horizontal lines or gaps in the image, check your terminal's Line
> Height or Vertical Offset settings. For the best experience, set line-height
> to 1.0 and use a "Mono" spaced font. Terminals like foot, kitty, and wezterm
> generally provide the best results out of the box.

## Features

- 🚀 **Fast**: Written in Rust, optimized for speed (~25x faster than Python).

- **Fuzzy Search**: Don't remember the exact name? `show pika` finds Pikachu.

- **Shell Completions**: `px2ansi-rs` can automatically generate the completion
  scripts for Bash, Zsh, Fish, and PowerShell.

- **Interactive Mode**: Browse your entire collection with a live-filtered TUI.

- **Multiple Modes**: Support for high-detail `ansi` or retro `unicode` blocks.

- 🎨 **Truecolor**: Supports full 24-bit RGB colors.

- 📐 **Smart Resizing**: Automatically detects terminal width and resizes large
  images to fit.

- 📂 **JSON Indexing**: Built-in tool to scan directories and generate a
  searchable manifest of your art library.

- 🖼️ **Flexible Filtering**: Use `nearest` for sharp pixel art or `lanczos3` for
  smooth photos.

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

### From `crates.io`

```bash
cargo install px2ansi-rs
```

### Command Table

| Command                                | Render Mode | Pixel Type      | Best For...                                                    |
| :------------------------------------- | :---------- | :-------------- | :------------------------------------------------------------- |
| `px2ansi-rs --help`                    | N/A         | N/A             | Listing command syntax                                         |
| `px2ansi-rs <TAB>`                     | N/A         | N/A             | Shell Completion, if you sourced it for your shell             |
| `px2ansi-rs ... --mode ansi`           | ANSI        | Half-block (▀)  | **Maximum Compatibility:** Standard 2-pixel vertical packing.  |
| `px2ansi-rs ... --mode unicode`        | Unicode     | Half-block (▀)  | **HD Unicode:** High-fidelity detail using modern symbol sets. |
| `px2ansi-rs ... --mode unicode --full` | Unicode     | Full-block (██) | **Retro Square:** 1:1 "pixel-perfect" square aesthetic.        |
| `px2ansi-rs index <dir>`               | Either      | N/A             | Creating a manifest                                            |
| `px2ansi-rs show -i`                   | Either      | Either          | Interactive fuzzy search                                       |
| `px2ansi-rs show random`               | Either      | Context aware   | Automation: Terminal greeting/random asset rotation            |
| `px2ansi-rs show chariz`               | Either      | Either          | Fuzzy Match e.g, 'chariz' opens Charizard                      |
| `px2ansi-rs list`                      | Either      | Either          | List all assets                                                |
| `px2ansi-rs list --count 10`           | Either      | Either          | List 10 assets                                                 |
| `px2ansi-rs ... ... -l`                | Either      | Either          | Opt-in for performance metrics                                 |

- The `--full` toggle is specifically optimized for **Unicode mode** to achieve
  a "pixel-perfect" square look.

### Usage

`px2ansi-rs` now uses a subcommand-based interface: `convert`, `index`, `show`,
and `list`

1. Convert an Image

Basic conversion to stdout (auto-resizes to fit your terminal):

```Bash
px2ansi-rs convert image.png
# These basically look the same for both modes
px2ansi-rs convert image.png --mode unicode
```

**Unicode Mode** (Retro Style)

To get the chunky "Pokemon Colorscript" look:

```Bash
px2ansi-rs convert image.png --mode unicode --full --filter nearest
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

The Quick Way (Supports fuzzy matching)

```bash
# Matches names similar to the actual name showing a match score
# This will open bulbasaur
px2ansi-rs show bul <ENTER>
```

![screenshot3](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/bul.png)

**Interactive Search (The "Browser" Mode)**

Don't want to type names? Open the interactive fuzzy-finder to scroll through
your entire index:

```bash
px2ansi-rs show -i
```

4. List Assets

**List All**

```bash
px2ansi-rs list
```

**List 10**

```bash
px2ansi-rs list --count 10
Index: Showing 10 of 1333 entries:
  • abomasnow            68x56px
  • abomasnow-mega       68x56px
  • abra                 68x56px
  • absol                68x56px
  • absol-mega           68x56px
  • accelgor             68x56px
  • aegislash            68x56px
  • aegislash-blade      68x56px
  • aerodactyl           68x56px
  • aerodactyl-mega      68x56px
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
px2ansi-rs show scream --filter triangle --width 50
```

---

### 🐚 Shell Completions

`px2ansi-rs` can automatically generate completion scripts for Bash, Zsh, Fish,
and PowerShell. This ensures that all subcommands (convert, show, index, list)
and flags are available via the TAB key.

**Quick Setup (Recommended)**

The fastest way to enable completions is to source them directly from the binary
in your shell configuration file.

**Zsh**

Add this to your `~/.zshrc` (or your NixOS Zsh module):

```bash
source <(px2ansi-rs completions zsh)
```

**Bash**

Add this to your `~/.bashrc`:

```bash
source <(px2ansi-rs completions bash)
```

**Fish**

Add this to `~/.config/fish/config.fish`:

```fish
px2ansi-rs completions fish | source
```

❄️ **NixOS Configuration**

For NixOS users developing locally, you can use Home Manager to ensure
completions are always active. Add the following to your Zsh module:

```nix
programs.zsh.initContent = ''
  # Ensure your local build is in the PATH
  export PATH="$HOME/projects/px2ansi-rs/target/debug:$PATH"

  # Inject completions dynamically if the binary exists
  if command -v px2ansi-rs >/dev/null; then
    source <(px2ansi-rs completions zsh)
  fi
'';
```

---

### 🎨 Rendering Modes

`px2ansi-rs` supports multiple ways to bring your sprites to life. Whether you
want crisp modern detail or chunky retro vibes, we've got you covered.

| Mode         | Command Flag     | Description                    | Best For                               |
| ------------ | ---------------- | ------------------------------ | -------------------------------------- |
| ANSI         | `--mode ansi`    | Standard 2-pixels-per-row      | Maximum compatibility & speed          |
| HD Unicode   | `--mode unicode` | Hi-Def Unicode half-blocks 1:1 | High-Fidelity assets                   |
| Retro Square | `--full`         | 1 pixel is a solid ██ square   | 8-bit/16-bit pixel art & retro styling |

By default, both ANSI and Unicode modes now utilize a "vertical packing"
technique to maximize resolution.

- **The Technique**: Instead of using one character for one pixel, both modes
  fit two vertical pixels into a single character cell. This is achieved by
  using the Unicode half-block (▀) and manipulating the foreground and
  background colors simultaneously.

- **The Result**: Without `--full`: Both modes provide the same high-density
  detail and use the full terminal width. They look identical because they are
  using the same underlying pixel-packing logic to maintain 1:1 scaling.
  - With `--full`: The logic switches from "packing" to "square-blocking," where
    each individual pixel is rendered as a double-wide full block (██). This
    creates the distinct chunky, retro aesthetic and causes the two modes to
    diverge visually.

---

## ⚡ Performance & Workflow

`px2ansi-rs` is designed for high-performance terminal environments. While it
can convert images on the fly, it is optimized for a "Build Once, Show Many"
workflow.

By default, the latency timer is not visible. To add performance metrics add the
`-l` `--latency` flag.

### The Indexing Advantage

Standard image-to-ANSI tools must decode, resize, and re-calculate ANSI escape
sequences every time they are run. `px2ansi-rs` separates these concerns:

1. `index`: Scans your asset directory and creates a JSON manifest. This avoids
   slow recursive directory walks during daily use.

2. `show`: Uses the index to jump directly to the file.

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

Performance metrics are now opt-in:

```Bash
# Show latency on screen
px2ansi -l show random
px2ansi convert <file> --latency
```

### Testing with the PokéSprite index over 1,300 entries

```bash
# Clone the repository (approx. 50MB)
git clone https://github.com/msikma/pokesprite.git

# Navigate to the Gen 8 sprites (the most modern and consistent)
cd pokesprite/pokemon-gen8/regular

# Create an Index of 1334 .png files
px2ansi-rs index . -o index.json -l
Index created successfully in 31ms!
```

```bash
px2ansi-rs show random -l
Finished in 0ms
```

Let's try the shiny set:

```bash
cd ~/pokesprite/pokemon-gen8/shiny/
px2ansi-rs index . -o shiny-index.json -l
Index created successfully in 30ms!

px2ansi-rs show gengar --filter nearest
Finished in 0ms
```

---

## Resize Filters (`--filter`)

- `nearest` — Nearest-neighbor. Fastest; best for pixel art / hard edges.
- `triangle` — Linear filter (bilinear).
- `catmull-rom` — Cubic filter.
- `gaussian` — Gaussian filter.
- `lanczos3` — Lanczos filter (window 3). Default.

- [guide.encode.moe resampling](https://guide.encode.moe/encoding/resampling.html)

## Project build with px2ansi-rs

- [slasher-horrorscripts](https://crates.io/crates/slasher-horrorscripts)

## ⚠️ Troubleshooting & Errors

`px2ansi-rs` uses robust error handling via `anyhow`. Here are common scenarios:

- **Broken Pipe**: Occurs if you pipe output into a tool that closes early
  (e.g., `px2ansi-rs show random | head -n 1`). This is normal CLI behavior.
- **Index Missing**: If `show` fails, ensure your `index.json` is in the current
  directory or specify it with `--index <PATH>`.
- **Fuzzy Score Threshold**: If a search returns no result, the "match score"
  was likely too low (below 30). Try a more specific search term or use `-i`.
- **Terminal Gaps**: If you see horizontal lines, your terminal's line-height is
  likely > 1.0.

## License

[GNU General Public License 3.0](https://github.com/saylesss88/px2ansi-rs/blob/main/LICENSE)

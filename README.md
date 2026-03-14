# px2ansi-rs

`px2ansi-rs` is a high-performance image-to-ANSI/Unicode toolkit written in
Rust. While inspired by the original Python
[px2ansi](https://github.com/Nellousan/px2ansi) project, this is a complete
reimplementation (~25x faster) with indexing, fuzzy search, TUI browsing, and
advanced filters.

![px2ansi-rs demo](https://raw.githubusercontent.com/saylesss88/px2ansi-rs/main/assets/px2ansi_demo.gif)

---

## Features

- **Fuzzy Search** — `show pika` → Pikachu
- **Interactive TUI** — `show -i` to browse
- **Truecolor + Transparency** — Full 24-bit RGB + alpha
- **Smart Resize** — Auto-fits terminal width
- **5 Filters** — `nearest` (pixel art) to `lanczos3` (photos)

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

---

## Quick Reference

| Use Case        | Command                                              | Mode    | Style | Notes                |
| --------------- | ---------------------------------------------------- | ------- | ----- | -------------------- |
| **Convert**     | `px2ansi-rs convert image.png`                       | auto    | ▀     | Terminal width       |
| **ANSI**        | `px2ansi-rs convert image.png --mode ansi`           | ANSI    | ▀     | Max compatibility    |
| **Unicode**     | `px2ansi-rs convert image.png --mode unicode`        | Unicode | ▀     | High detail          |
| **Retro**       | `px2ansi-rs convert image.png --mode unicode --full` | Unicode | ██    | Pixel perfect        |
| **Index**       | `px2ansi-rs index <dir> -o index.json`               | N/A     | N/A   | Creates `index.json` |
| **Interactive** | `px2ansi-rs show -i`                                 | auto    | auto  | Fuzzy TUI browser    |
| **Fuzzy**       | `px2ansi-rs show chariz`                             | auto    | auto  | → Charizard          |
| **Random**      | `px2ansi-rs show random`                             | auto    | auto  | Terminal greeting    |
| **List**        | `px2ansi-rs list --count 10`                         | N/A     | N/A   | First 10 assets      |

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

- By default, `px2ansi-rs show` shows a random sprite from the `index.json` in
  the current folder.

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

---

### ⚙️ Configuration

`px2ansi-rs` supports a configuration file to save your preferred defaults.

**File Location**

The config is stored in your standard system config directory:

- **Linux**: `~/.config/px2ansi-rs/default-config.toml`
- **macOS**: `~/Library/Application Support/px2ansi-rs/default-config.toml`
- **Windows**: `%AppData%\px2ansi-rs\config\default-config.toml`

**Example** `default-config.toml`

You can create this file manually to override the engine's built-in defaults:

```toml
# Output mode: "ansi" (2 pixels per cell) or "unicode"
mode = "ansi"
# Always show execution timing metadata
latency = true
# Default filter: "nearest", "triangle", "catmull-rom", "gaussian", "lanczos3"
filter = "lanczos3"
# Index file name to target (absolute path)
index = "/home/your-user/pokesprite/pokemon-gen8/shiny/shiny-index.json"
# Use double-width full blocks (██) for square pixels
full = false
```

You can call your index from anywhere in your filesystem by using the `-I` flag,
or adding the `index` path like we did above, or you can pass it from the cli
`px2ansi-rs show -I /home/your-user/pokesprite/pokemon-gen8/shiny/shiny-index.json`

> Note: Any field omitted from the `.toml` file will automatically fall back to
> the engine's built-in defaults.

**Hierarchy of Truth**

The engine resolves settings in this order:

1. **CLI Flags** (e.g., `--mode unicode`) always wins.

2. **Config File** (`default-config.toml`) used if no flag is provided.

3. **Hardcoded Defaults** used if the config file is missing.

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

You can specify a filter via the `--filter` flag or in your
`default-config.toml`.

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

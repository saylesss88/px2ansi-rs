# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `--fetch` mode: display system info alongside the rendered image, similar to
  neofetch/fastfetch
- `FetchConfig` struct to control which fields appear and how they are labelled,
  with optional per-field label overrides and configurable key-column width
- Config file support: `~/fetch.conf` is loaded automatically; unknown keys are
  silently ignored so forwards compatibility is preserved
- New fetch fields: CPU model, CPU usage, shell, disk usage (`/`), local IP
  address, and system architecture (in addition to OS, kernel, uptime, memory,
  process count, and locale)
- Terminal-width awareness: queries `$COLUMNS`, then `TIOCGWINSZ` on stderr and
  stdout so the layout is correct even when stdout is redirected or piped
- Automatic stacked fallback: when the terminal is too narrow for side-by-side
  display the image is printed above the fetch text rather than beside it
- ANSI-safe line truncation (`truncate_ansi`) that skips CSI escape sequences
  when measuring visible width and closes open SGR codes at the truncation point
  to prevent colour bleed
- Image width is now capped relative to terminal width before rendering so the
  fetch text always has room regardless of image size

### Changed
- `print_with_left_block_writer` now accepts an explicit `cols: usize` parameter
  so the image-rendering and layout steps agree on a single terminal-width
  measurement
- Half-block render path now consistently applies the `* 0.5` terminal-column
  correction to image width in all branches (previously the correction was
  missing from the `scale < 1.0` path, causing sprites to render at double
  their intended column width)
- `GAP` reduced from 3 to 2 columns and `MIN_RIGHT_BUDGET` reduced from 20 to
  12 columns so side-by-side layout is preserved on narrower panes (e.g. a
  tiling window manager with the terminal occupying roughly half the screen)

### Fixed
- Fetch text no longer wraps back to the left edge of the screen on half-width
  terminal panes in tiling window managers such as `mango-wc`
- `chars.next().unwrap()` inside `truncate_ansi` replaced with `if let` to
  eliminate the panic path when parsing a truncated escape sequence
- `u32::try_from` result is now properly unwrapped with `unwrap_or` rather than
  being used directly as a `u32`, which previously caused a type error when
  calling `.min()` on it

## [0.1.0] - 2025-01-23

### Added
- Initial release
- Convert images to ANSI art using half-block, ASCII, Chinese, and Kanji
  character sets via `px2ansi`
- `--fetch` placeholder displaying OS, kernel, uptime, memory, CPU usage,
  process count, and locale alongside the rendered image

[Unreleased]: https://github.com/saylesss88/px2ansi-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/saylesss88/px2ansi-rs/releases/tag/v0.1.0


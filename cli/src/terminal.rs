//! Terminal capability queries.
//!
//! This module provides utilities for querying terminal properties at runtime,
//! such as the background color via the OSC 11 escape sequence.

use crossterm::terminal;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

/// Queries the terminal for its current background color using the OSC 11
/// escape sequence.
///
/// Temporarily enables raw mode so the terminal's response can be read back
/// from stdin without waiting for a newline. Raw mode is restored
/// unconditionally via [`RawModeGuard`] even if parsing fails or an early
/// return occurs.
///
/// # Returns
///
/// Returns `Some([r, g, b])` if the terminal responds with a valid
/// `rgb:RRRR/GGGG/BBBB` color. Returns `None` if:
/// - Raw mode cannot be enabled
/// - The terminal does not respond within 500 ms
/// - The response cannot be parsed
///
/// # Terminal support
///
/// Most modern terminals support OSC 11 (foot, kitty, WezTerm, xterm).
/// Ghostty supports it as of recent versions. Windows Terminal does not.
#[must_use]
pub fn query_terminal_bg() -> Option<[u8; 3]> {
    terminal::enable_raw_mode().ok()?;
    let _guard = RawModeGuard;

    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b]11;?\x1b\\").ok()?;
    stdout.flush().ok()?;

    let response = read_osc_response(Duration::from_millis(500))?;

    // Temporary debug — remove after confirming
    eprintln!("[osc11] raw response: {:?}", response);

    let result = parse_osc11(&response);
    eprintln!("[osc11] parsed: {:?}", result);
    result
} // pub fn query_terminal_bg() -> Option<[u8; 3]> {
  //     terminal::enable_raw_mode().ok()?;
  //     // RawModeGuard restores the terminal unconditionally on drop, so any
  //     // early `?` return below will not leave the terminal in raw mode.
  //     let _guard = RawModeGuard;

//     let mut stdout = io::stdout();
//     // OSC 11 query: ESC ] 11 ; ? ST
//     stdout.write_all(b"\x1b]11;?\x1b\\").ok()?;
//     stdout.flush().ok()?;

//     let response = read_osc_response(Duration::from_millis(500))?;
//     parse_osc11(&response)
// }

/// RAII guard that restores the terminal from raw mode on drop.
///
/// Constructed after [`terminal::enable_raw_mode`] succeeds. Because this
/// implements [`Drop`], raw mode is always restored even if the enclosing
/// function returns early via `?` or panics.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Ignore errors — we are in a destructor and cannot propagate them.
        let _ = terminal::disable_raw_mode();
    }
}

/// Reads a single OSC response from stdin, with a timeout.
///
/// Sets stdin to non-blocking mode, polls byte-by-byte, and restores blocking
/// mode before returning. The read stops when either:
/// - The OSC string terminator `ST` (`\x1b\\`) is seen
/// - A `BEL` (`\x07`) terminator is seen (xterm compatibility)
/// - The deadline is reached
/// - Any unexpected I/O error occurs
///
/// Returns `None` if nothing was read before the deadline.
#[cfg(unix)]
fn read_osc_response(timeout: Duration) -> Option<String> {
    let deadline = Instant::now() + timeout;
    let mut buf = Vec::new();
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];

    set_stdin_nonblocking(&mut stdin, true);

    loop {
        if Instant::now() >= deadline {
            break;
        }
        match stdin.read(&mut byte) {
            Ok(1) => {
                buf.push(byte[0]);
                let ends_with_st =
                    buf.len() >= 2 && buf[buf.len() - 2] == b'\x1b' && buf[buf.len() - 1] == b'\\';
                let ends_with_bel = byte[0] == b'\x07';
                if ends_with_st || ends_with_bel {
                    break;
                }
            }
            Ok(_) => break,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    }

    set_stdin_nonblocking(&mut stdin, false);

    if buf.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buf).into_owned())
    }
}

/// Toggles non-blocking I/O on stdin using `fcntl(2)`.
///
/// # Safety
///
/// `fcntl` is called with `F_GETFL` / `F_SETFL` on a valid file descriptor
/// obtained from `stdin`. The fd is guaranteed to be open for the lifetime of
/// this call because `stdin` is borrowed. The flag manipulation is a standard
/// POSIX pattern and cannot cause memory unsafety.
#[cfg(unix)]
fn set_stdin_nonblocking(stdin: &mut io::Stdin, nonblocking: bool) {
    use std::os::unix::io::AsRawFd;

    // SAFETY: see function-level doc comment.
    unsafe {
        let fd = stdin.as_raw_fd();
        let flags = libc::fcntl(fd, libc::F_GETFL, 0);
        if flags == -1 {
            return;
        }
        let new_flags = if nonblocking {
            flags | libc::O_NONBLOCK
        } else {
            flags & !libc::O_NONBLOCK
        };
        libc::fcntl(fd, libc::F_SETFL, new_flags);
    }
}

/// Parses an OSC 11 terminal response into an RGB triple.
///
/// Expects a string containing `rgb:RRRR/GGGG/BBBB` where each component is a
/// 4-digit 16-bit hex value. Only the high byte (first two hex digits) of each
/// channel is used, which is standard practice for OSC color responses.
///
/// # Example response
///
/// ```text
/// \x1b]11;rgb:1a1a/1b1b/2626\x1b\\
/// ```
///
/// Returns `None` if the expected pattern is not found or any component fails
/// to parse.
fn parse_osc11(response: &str) -> Option<[u8; 3]> {
    let start = response.find("rgb:")?;
    let rgb_str = &response[start + 4..];
    let parts: Vec<&str> = rgb_str.splitn(3, '/').collect();

    if parts.len() < 3 {
        return None;
    }

    let r = u8::from_str_radix(parts[0].get(..2)?, 16).ok()?;
    let g = u8::from_str_radix(parts[1].get(..2)?, 16).ok()?;

    let end_of_blue = parts[2]
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(parts[2].len());

    let blue_hex = parts[2].get(..end_of_blue)?;
    let b = u8::from_str_radix(blue_hex.get(..2)?, 16).ok()?;

    Some([r, g, b])
}
// fn parse_osc11(response: &str) -> Option<[u8; 3]> {
//     let start = response.find("rgb:")?;
//     let rgb_str = &response[start + 4..];
//     let parts: Vec<&str> = rgb_str.splitn(3, '/').collect();
//     if parts.len() < 3 {
//         return None;
//     }
//     let r = u8::from_str_radix(parts[0].get(..2)?, 16).ok()?;
//     let g = u8::from_str_radix(parts[1].get(..2)?, 16).ok()?;
//     // Strip any trailing non-hex chars (ST, BEL, etc.) before parsing blue
//     let blue_str: String = parts[2]
//         .chars()
//         .take_while(|c| c.is_ascii_hexdigit())
//         .collect();
//     let b = u8::from_str_radix(blue_str.get(..2)?, 16).ok()?;

//     Some([r, g, b])
// }

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_osc11 ──────────────────────────────────────────────────────────

    /// Standard 4-digit-per-channel response as emitted by foot and kitty.
    #[test]
    fn parse_osc11_standard_4digit() {
        // Tokyo Night background: #1a1b26 → 1a1a/1b1b/2626
        let response = "\x1b]11;rgb:1a1a/1b1b/2626\x1b\\";
        assert_eq!(parse_osc11(response), Some([0x1a, 0x1b, 0x26]));
    }

    /// Some terminals (xterm) emit 2-digit channels instead of 4.
    #[test]
    fn parse_osc11_2digit_channels() {
        let response = "\x1b]11;rgb:1a/1b/26\x1b\\";
        assert_eq!(parse_osc11(response), Some([0x1a, 0x1b, 0x26]));
    }

    /// BEL terminator variant (\x07) used by some xterm-compatible terminals.
    #[test]
    fn parse_osc11_bel_terminator() {
        let response = "\x1b]11;rgb:ffff/0000/ffff\x07";
        assert_eq!(parse_osc11(response), Some([0xff, 0x00, 0xff]));
    }

    /// Pure white background.
    #[test]
    fn parse_osc11_white_background() {
        let response = "\x1b]11;rgb:ffff/ffff/ffff\x1b\\";
        assert_eq!(parse_osc11(response), Some([0xff, 0xff, 0xff]));
    }

    /// Pure black background.
    #[test]
    fn parse_osc11_black_background() {
        let response = "\x1b]11;rgb:0000/0000/0000\x1b\\";
        assert_eq!(parse_osc11(response), Some([0x00, 0x00, 0x00]));
    }

    /// Gruvbox light background (#fbf1c7 → fbfb/f1f1/c7c7).
    #[test]
    fn parse_osc11_gruvbox_light() {
        let response = "\x1b]11;rgb:fbfb/f1f1/c7c7\x1b\\";
        assert_eq!(parse_osc11(response), Some([0xfb, 0xf1, 0xc7]));
    }

    /// Response with extra junk before the rgb: tag — real terminals sometimes
    /// include extra fields.
    #[test]
    fn parse_osc11_extra_prefix() {
        let response = "\x1b]11;rgba:garbage;rgb:1a1a/1b1b/2626\x1b\\";
        assert_eq!(parse_osc11(response), Some([0x1a, 0x1b, 0x26]));
    }

    /// Missing rgb: tag — should return None, not panic.
    #[test]
    fn parse_osc11_missing_rgb_tag() {
        let response = "\x1b]11;unknown\x1b\\";
        assert_eq!(parse_osc11(response), None);
    }

    /// Truncated response — only two channels present.
    #[test]
    fn parse_osc11_truncated() {
        let response = "\x1b]11;rgb:1a1a/1b1b\x1b\\";
        assert_eq!(parse_osc11(response), None);
    }

    /// Empty string — should return None cleanly.
    #[test]
    fn parse_osc11_empty() {
        assert_eq!(parse_osc11(""), None);
    }

    /// Garbage input — should return None, not panic.
    #[test]
    fn parse_osc11_garbage() {
        assert_eq!(parse_osc11("rgb:zzzz/zzzz/zzzz"), None);
    }

    // ── set_stdin_nonblocking ────────────────────────────────────────────────

    /// Smoke test: toggling non-blocking on stdin and back should not panic
    /// or corrupt the fd. We can't easily assert the flag value from safe
    /// Rust, so we just verify it doesn't explode.
    #[test]
    #[cfg(unix)]
    fn set_nonblocking_roundtrip() {
        let mut stdin = std::io::stdin();
        set_stdin_nonblocking(&mut stdin, true);
        set_stdin_nonblocking(&mut stdin, false);
        // If we get here without SIGSEGV or a panic, the unsafe fcntl
        // calls are behaving correctly.
    }
}

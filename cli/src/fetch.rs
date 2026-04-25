//! `--fetch` mode: display system info alongside the image (or spinning animation).

use ansi_width::ansi_width;
use anyhow::Result;
use colored::Colorize;
use image::DynamicImage;
use px2ansi::RenderOptions;
use std::env;
use std::io::Write;
use sysinfo::System;

/// # Errors
///
/// This function will return an error if any of the following occur:
///
/// * **Rendering Failure**: The internal [`render.render()`] call fails. This typically happens
///   if the image dimensions are incompatible with the selected [`CharsetMode`] or if the
///   underlying color quantization fails.
/// * **UTF-8 Lossy Conversion**: While [`String::from_utf8_lossy`] is used to prevent hard
///   panics on invalid sequences, any upstream failure in the buffer population that violates
///   basic string constraints during the write process may propagate through the [`Result`].
/// * **IO Errors**: The `writer` fails to process the output stream. This is common if
///   the terminal is disconnected, a pipe is broken (`SIGPIPE`), or there is insufficient
///   space in the buffer.
pub fn print_fetch_with_image<W: Write>(
    img: &DynamicImage,
    render: &RenderOptions,
    writer: &mut W,
) -> Result<()> {
    let mut img_buf: Vec<u8> = Vec::new();
    render.render(img, &mut img_buf)?;
    let img_str = String::from_utf8_lossy(&img_buf);
    crate::fetch::print_with_left_block_writer(&img_str, writer)
}

fn username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "victim".to_string())
}

#[must_use]
pub fn linux_locale() -> String {
    env::var("LC_ALL")
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "C".to_string())
}
#[must_use]
pub fn fetch_lines() -> Vec<String> {
    // new_all ensures we get CPU and process lists immediately
    let mut sys = System::new_all();
    sys.refresh_all();

    let os = System::name().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "?.?.?".to_string());
    let uptime = System::uptime();
    let hostname = System::host_name().unwrap_or_else(|| "my-host".to_string());

    // RAM: Convert bytes to MiB for readability
    let total_mem = sys.total_memory() / 1024 / 1024;
    let used_mem = sys.used_memory() / 1024 / 1024;

    // CPU: Get the global usage percentage
    let cpu_usage = sys.global_cpu_usage();

    // Process count: How many "souls" are running on the system
    let process_count = sys.processes().len();

    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let mins = (uptime % 3600) / 60;

    let uptime_str = if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else {
        format!("{hours}h {mins}m")
    };

    let key = |k: &str| format!("{k:<10}").red().bold();

    let locale = linux_locale();

    vec![
        format!("{}@{}", username().red().bold(), hostname.white()),
        "-----------------------".white().to_string(),
        format!("{}: {}", key("OS"), os.white()),
        format!("{}: {}", key("Kernel"), kernel.white()),
        format!("{}: {}", key("Uptime"), uptime_str.white()),
        format!(
            "{}: {} / {} MiB",
            key("Memory"),
            used_mem.to_string().yellow(),
            total_mem.to_string().white()
        ),
        format!("{}: {:.1}%", key("CPU"), cpu_usage.to_string().yellow()),
        format!(
            "{}: {}",
            key("Processes"),
            process_count.to_string().purple()
        ),
        format!("{}: {}", key("User Locale"), locale.purple()),
        // For "Slashers 13" pass script_count:usize to this func
        // format!("{}: {}", key("Slashers"), script_count.to_string().cyan()),
    ]
}

/// # Errors
///
/// This function will return an error if any of the following occur:
///
/// * **IO Write Failure**: The function fails to write to the provided `writer`.
///   This is common if the output stream is closed unexpectedly (e.g., a broken
///   pipe when piping to `head` or `less`).
/// * **System Info Retrieval**: While `fetch_lines()` typically handles internal
///   errors gracefully, any underlying failure in gathering system information
///   that causes a panic or an empty return may result in an improperly
///   formatted layout.
///
/// # Panics
///
/// This function is designed to be panic-safe through the use of `get()` and
/// `unwrap_or("")`. However, it assumes that [`ansi_width`] correctly handles
/// the provided string slices.
pub fn print_with_left_block_writer<W: Write>(left: &str, writer: &mut W) -> Result<()> {
    let left_lines: Vec<&str> = left.lines().collect();
    let right_lines = fetch_lines();
    let left_width = left_lines.iter().map(|l| ansi_width(l)).max().unwrap_or(0);
    let pad = left_width + 3;
    let max_lines = left_lines.len().max(right_lines.len());

    writeln!(writer)?;
    for i in 0..max_lines {
        let l = *left_lines.get(i).unwrap_or(&"");
        let r = right_lines.get(i).map_or("", String::as_str);
        let l_w = ansi_width(l);
        let spaces = pad.saturating_sub(l_w);
        write!(writer, "{l}")?;
        write!(writer, "{:width$}", "", width = spaces)?;
        writeln!(writer, "{r}")?;
    }
    writeln!(writer)?;
    Ok(())
}

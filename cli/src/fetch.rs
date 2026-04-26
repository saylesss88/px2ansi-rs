//! `--fetch` mode: display system info alongside the image (or spinning animation).
use ansi_width::ansi_width;
use anyhow::Result;
use colored::Colorize;
use image::DynamicImage;
use px2ansi::{CharsetMode, RenderOptions};
use std::env;
use std::fs;
use std::io::Write;
use sysinfo::{Disks, Networks, System};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Controls which fields appear in the fetch output and how they're labeled.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct FetchConfig {
    // Header
    pub show_header: bool,
    pub show_separator: bool,
    // System
    pub show_os: bool,
    pub show_kernel: bool,
    pub show_arch: bool,
    pub show_hostname: bool,
    // Hardware
    pub show_cpu: bool,
    pub show_cpu_usage: bool,
    pub show_memory: bool,
    pub show_disk: bool,
    // Runtime
    pub show_uptime: bool,
    pub show_processes: bool,
    pub show_shell: bool,
    // Environment
    pub show_locale: bool,
    pub show_local_ip: bool,
    // Label overrides (None → use built-in default)
    pub label_os: Option<String>,
    pub label_kernel: Option<String>,
    pub label_arch: Option<String>,
    pub label_hostname: Option<String>,
    pub label_cpu: Option<String>,
    pub label_cpu_usage: Option<String>,
    pub label_memory: Option<String>,
    pub label_disk: Option<String>,
    pub label_uptime: Option<String>,
    pub label_processes: Option<String>,
    pub label_shell: Option<String>,
    pub label_locale: Option<String>,
    pub label_local_ip: Option<String>,
    /// Width of the key column (padded with spaces).
    pub key_width: usize,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            show_header: true,
            show_separator: true,
            show_os: true,
            show_kernel: true,
            show_arch: true,
            show_hostname: false, // already in header
            show_cpu: true,
            show_cpu_usage: true,
            show_memory: true,
            show_disk: true,
            show_uptime: true,
            show_processes: true,
            show_shell: true,
            show_locale: true,
            show_local_ip: true,
            label_os: None,
            label_kernel: None,
            label_arch: None,
            label_hostname: None,
            label_cpu: None,
            label_cpu_usage: None,
            label_memory: None,
            label_disk: None,
            label_uptime: None,
            label_processes: None,
            label_shell: None,
            label_locale: None,
            label_local_ip: None,
            key_width: 12,
        }
    }
}

impl FetchConfig {
    /// Parse a minimal key = value config file.
    ///
    /// Lines starting with `#` are ignored. Recognised keys:
    /// ```text
    /// show_os      = false
    /// label_os     = Operating System
    /// key_width    = 14
    /// ```
    #[must_use]
    pub fn from_file(path: &str) -> Self {
        let mut cfg = Self::default();
        let Ok(text) = fs::read_to_string(path) else {
            return cfg;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "show_header" => cfg.show_header = v != "false",
                "show_separator" => cfg.show_separator = v != "false",
                "show_os" => cfg.show_os = v != "false",
                "show_kernel" => cfg.show_kernel = v != "false",
                "show_arch" => cfg.show_arch = v != "false",
                "show_hostname" => cfg.show_hostname = v != "false",
                "show_cpu" => cfg.show_cpu = v != "false",
                "show_cpu_usage" => cfg.show_cpu_usage = v != "false",
                "show_memory" => cfg.show_memory = v != "false",
                "show_disk" => cfg.show_disk = v != "false",
                "show_uptime" => cfg.show_uptime = v != "false",
                "show_processes" => cfg.show_processes = v != "false",
                "show_shell" => cfg.show_shell = v != "false",
                "show_locale" => cfg.show_locale = v != "false",
                "show_local_ip" => cfg.show_local_ip = v != "false",
                "label_os" => cfg.label_os = Some(v.to_string()),
                "label_kernel" => cfg.label_kernel = Some(v.to_string()),
                "label_arch" => cfg.label_arch = Some(v.to_string()),
                "label_hostname" => cfg.label_hostname = Some(v.to_string()),
                "label_cpu" => cfg.label_cpu = Some(v.to_string()),
                "label_cpu_usage" => cfg.label_cpu_usage = Some(v.to_string()),
                "label_memory" => cfg.label_memory = Some(v.to_string()),
                "label_disk" => cfg.label_disk = Some(v.to_string()),
                "label_uptime" => cfg.label_uptime = Some(v.to_string()),
                "label_processes" => cfg.label_processes = Some(v.to_string()),
                "label_shell" => cfg.label_shell = Some(v.to_string()),
                "label_locale" => cfg.label_locale = Some(v.to_string()),
                "label_local_ip" => cfg.label_local_ip = Some(v.to_string()),
                "key_width" => {
                    if let Ok(n) = v.parse() {
                        cfg.key_width = n;
                    }
                }
                _ => {}
            }
        }
        cfg
    }

    /// Convenience: load from `~/.config/yourapp/fetch.conf` if it exists,
    /// otherwise return defaults.
    #[must_use]
    pub fn load_default() -> Self {
        if let Some(home) = env::var_os("HOME") {
            let path = format!("{}/fetch.conf", home.to_string_lossy());
            if fs::metadata(&path).is_ok() {
                return Self::from_file(&path);
            }
        }
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Info helpers
// ---------------------------------------------------------------------------

fn username() -> String {
    env::var("USER").unwrap_or_else(|_| "user".to_string())
}

#[must_use]
pub fn linux_locale() -> String {
    env::var("LC_ALL")
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "C".to_string())
}

fn current_shell() -> String {
    // $SHELL is the login shell path, e.g. /bin/zsh
    env::var("SHELL").map_or_else(
        |_| "unknown".to_string(),
        |s| s.rsplit('/').next().unwrap_or(&s).to_string(),
    )
}

fn cpu_model(sys: &System) -> String {
    sys.cpus()
        .first()
        .map_or_else(|| "Unknown CPU".to_string(), |c| c.brand().to_string())
}

fn local_ip() -> String {
    let networks = Networks::new_with_refreshed_list();
    for (iface, data) in &networks {
        // Skip loopback
        if iface == "lo" || iface.starts_with("lo") {
            continue;
        }
        for addr in data.ip_networks() {
            let ip = addr.addr;
            if ip.is_ipv4() && !ip.to_string().starts_with("127.") {
                return ip.to_string();
            }
        }
    }
    "N/A".to_string()
}

fn disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();
    // Find the root mount
    for disk in &disks {
        if disk.mount_point().to_str() == Some("/") {
            let total = disk.total_space() / 1024 / 1024 / 1024;
            let free = disk.available_space() / 1024 / 1024 / 1024;
            let used = total.saturating_sub(free);
            return format!("{used} / {total} GiB");
        }
    }
    "N/A".to_string()
}

fn arch() -> String {
    // std::env::consts::ARCH gives the compile-target arch
    std::env::consts::ARCH.to_string()
}

fn uptime_string(uptime: u64) -> String {
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let mins = (uptime % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else {
        format!("{hours}h {mins}m")
    }
}

// ---------------------------------------------------------------------------
// fetch_lines  (now config-driven)
// ---------------------------------------------------------------------------

#[must_use]
pub fn fetch_lines() -> Vec<String> {
    fetch_lines_with_config(&FetchConfig::load_default())
}

#[must_use]
pub fn fetch_lines_with_config(cfg: &FetchConfig) -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut lines = Vec::new();
    let w = cfg.key_width;

    // Macro to handle the "Label: Value" pattern conditionally
    macro_rules! add {
        ($show:expr, $label:expr, $opt:expr, $val:expr) => {
            if $show {
                let key = format!("{:<w$}", $opt.as_deref().unwrap_or($label))
                    .red()
                    .bold();
                lines.push(format!("{key}: {}", $val));
            }
        };
    }

    if cfg.show_header {
        lines.push(format!(
            "{}@{}",
            username().red().bold(),
            System::host_name().unwrap_or_default().white()
        ));
    }
    if cfg.show_separator {
        lines.push("─".repeat(28).white().to_string());
    }

    add!(
        cfg.show_os,
        "OS",
        &cfg.label_os,
        System::name().unwrap_or_default().white()
    );
    add!(
        cfg.show_kernel,
        "Kernel",
        &cfg.label_kernel,
        System::kernel_version().unwrap_or_default().white()
    );
    add!(cfg.show_arch, "Arch", &cfg.label_arch, arch().white());
    add!(
        cfg.show_hostname,
        "Hostname",
        &cfg.label_hostname,
        System::host_name().unwrap_or_default().white()
    );
    add!(
        cfg.show_uptime,
        "Uptime",
        &cfg.label_uptime,
        uptime_string(System::uptime()).white()
    );
    add!(
        cfg.show_shell,
        "Shell",
        &cfg.label_shell,
        current_shell().white()
    );
    add!(cfg.show_cpu, "CPU", &cfg.label_cpu, cpu_model(&sys).white());

    if cfg.show_cpu_usage {
        add!(
            true,
            "CPU Usage",
            &cfg.label_cpu_usage,
            format!("{:.1}%", sys.global_cpu_usage()).yellow()
        );
    }
    if cfg.show_memory {
        let (u, t) = (sys.used_memory() >> 20, sys.total_memory() >> 20);
        add!(
            true,
            "Memory",
            &cfg.label_memory,
            format!("{} / {} MiB", u.to_string().yellow(), t.to_string().white())
        );
    }

    add!(
        cfg.show_disk,
        "Disk (/)",
        &cfg.label_disk,
        disk_usage().yellow()
    );
    add!(
        cfg.show_processes,
        "Processes",
        &cfg.label_processes,
        sys.processes().len().to_string().purple()
    );
    add!(
        cfg.show_locale,
        "Locale",
        &cfg.label_locale,
        linux_locale().purple()
    );
    add!(
        cfg.show_local_ip,
        "Local IP",
        &cfg.label_local_ip,
        local_ip().cyan()
    );

    lines
} // pub fn fetch_lines_with_config(cfg: &FetchConfig) -> Vec<String> {
  //     let mut sys = System::new_all();
  //     sys.refresh_all();

//     let mut lines = Vec::new();
//     let w = cfg.key_width;

//     // 1. Logic for formatting keys
//     let fmt_key = |default: &str, ovr: &Option<String>| {
//         format!("{:<w$}", ovr.as_deref().unwrap_or(default))
//             .red()
//             .bold()
//     };

//     // 2. Logic for generating the line string (no more borrow errors!)
//     let gen_line =
//         |show: bool, label: &str, ovr: &Option<String>, value: colored::ColoredString| {
//             if show {
//                 Some(format!("{}: {}", fmt_key(label, ovr), value))
//             } else {
//                 None
//             }
//         };

//     // --- Header & Separator (Direct push works now) ---
//     if cfg.show_header {
//         lines.push(format!(
//             "{}@{}",
//             username().red().bold(),
//             System::host_name().unwrap_or_default().white()
//         ));
//     }
//     if cfg.show_separator {
//         lines.push("─".repeat(28).white().to_string());
//     }

//     // --- System Info Rows ---
//     // Use .extend() with an array of Options to push multiple lines cleanly
//     lines.extend(
//         [
//             gen_line(
//                 cfg.show_os,
//                 "OS",
//                 &cfg.label_os,
//                 System::name().unwrap_or_default().white(),
//             ),
//             gen_line(
//                 cfg.show_kernel,
//                 "Kernel",
//                 &cfg.label_kernel,
//                 System::kernel_version().unwrap_or_default().white(),
//             ),
//             gen_line(cfg.show_arch, "Arch", &cfg.label_arch, arch().white()),
//             gen_line(
//                 cfg.show_uptime,
//                 "Uptime",
//                 &cfg.label_uptime,
//                 uptime_string(System::uptime()).white(),
//             ),
//             gen_line(
//                 cfg.show_shell,
//                 "Shell",
//                 &cfg.label_shell,
//                 current_shell().white(),
//             ),
//             gen_line(cfg.show_cpu, "CPU", &cfg.label_cpu, cpu_model(&sys).white()),
//             // Handle special cases with inline logic
//             if cfg.show_cpu_usage {
//                 gen_line(
//                     true,
//                     "CPU Usage",
//                     &cfg.label_cpu_usage,
//                     format!("{:.1}%", sys.global_cpu_usage()).yellow(),
//                 )
//             } else {
//                 None
//             },
//             if cfg.show_memory {
//                 let (u, t) = (
//                     sys.used_memory() / 1_048_576,
//                     sys.total_memory() / 1_048_576,
//                 );
//                 gen_line(
//                     true,
//                     "Memory",
//                     &cfg.label_memory,
//                     format!("{} / {} MiB", u.to_string().yellow(), t.to_string().white()).into(),
//                 )
//             } else {
//                 None
//             },
//             gen_line(
//                 cfg.show_disk,
//                 "Disk (/)",
//                 &cfg.label_disk,
//                 disk_usage().yellow(),
//             ),
//             gen_line(
//                 cfg.show_processes,
//                 "Processes",
//                 &cfg.label_processes,
//                 sys.processes().len().to_string().purple(),
//             ),
//             gen_line(
//                 cfg.show_locale,
//                 "Locale",
//                 &cfg.label_locale,
//                 linux_locale().purple(),
//             ),
//             gen_line(
//                 cfg.show_local_ip,
//                 "Local IP",
//                 &cfg.label_local_ip,
//                 local_ip().cyan(),
//             ),
//         ]
//         .into_iter()
//         .flatten(),
//     );

//     lines
// }
// pub fn fetch_lines_with_config(cfg: &FetchConfig) -> Vec<String> {
//     let mut sys = System::new_all();
//     sys.refresh_all();

//     let w = cfg.key_width;
//     // Build a closure so each field uses the right label + width
//     let key = |default: &str, override_: &Option<String>| {
//         let label = override_.as_deref().unwrap_or(default);
//         format!("{label:<w$}").red().bold()
//     };

//     // Gather raw values up-front (cheap to compute, only used if shown)
//     let os = System::name().unwrap_or_else(|| "Unknown".to_string());
//     let kernel = System::kernel_version().unwrap_or_else(|| "?.?.?".to_string());
//     let hostname = System::host_name().unwrap_or_else(|| "my-host".to_string());
//     let uptime_str = uptime_string(System::uptime());
//     let total_mem = sys.total_memory() / 1024 / 1024;
//     let used_mem = sys.used_memory() / 1024 / 1024;
//     let cpu_usage = sys.global_cpu_usage();
//     let process_count = sys.processes().len();

//     let mut lines: Vec<String> = Vec::new();

//     // Header: user@host
//     if cfg.show_header {
//         lines.push(format!("{}@{}", username().red().bold(), hostname.white()));
//     }
//     if cfg.show_separator {
//         lines.push("─".repeat(28).white().to_string());
//     }

//     if cfg.show_os {
//         lines.push(format!("{}: {}", key("OS", &cfg.label_os), os.white()));
//     }
//     if cfg.show_kernel {
//         lines.push(format!(
//             "{}: {}",
//             key("Kernel", &cfg.label_kernel),
//             kernel.white()
//         ));
//     }
//     if cfg.show_arch {
//         lines.push(format!(
//             "{}: {}",
//             key("Arch", &cfg.label_arch),
//             arch().white()
//         ));
//     }
//     if cfg.show_hostname {
//         lines.push(format!(
//             "{}: {}",
//             key("Hostname", &cfg.label_hostname),
//             hostname.white()
//         ));
//     }
//     if cfg.show_uptime {
//         lines.push(format!(
//             "{}: {}",
//             key("Uptime", &cfg.label_uptime),
//             uptime_str.white()
//         ));
//     }
//     if cfg.show_shell {
//         lines.push(format!(
//             "{}: {}",
//             key("Shell", &cfg.label_shell),
//             current_shell().white()
//         ));
//     }
//     if cfg.show_cpu {
//         lines.push(format!(
//             "{}: {}",
//             key("CPU", &cfg.label_cpu),
//             cpu_model(&sys).white()
//         ));
//     }
//     if cfg.show_cpu_usage {
//         lines.push(format!(
//             "{}: {:.1}%",
//             key("CPU Usage", &cfg.label_cpu_usage),
//             cpu_usage.to_string().yellow()
//         ));
//     }
//     if cfg.show_memory {
//         lines.push(format!(
//             "{}: {} / {} MiB",
//             key("Memory", &cfg.label_memory),
//             used_mem.to_string().yellow(),
//             total_mem.to_string().white()
//         ));
//     }
//     if cfg.show_disk {
//         lines.push(format!(
//             "{}: {}",
//             key("Disk (/)", &cfg.label_disk),
//             disk_usage().yellow()
//         ));
//     }
//     if cfg.show_processes {
//         lines.push(format!(
//             "{}: {}",
//             key("Processes", &cfg.label_processes),
//             process_count.to_string().purple()
//         ));
//     }
//     if cfg.show_locale {
//         lines.push(format!(
//             "{}: {}",
//             key("Locale", &cfg.label_locale),
//             linux_locale().purple()
//         ));
//     }
//     if cfg.show_local_ip {
//         lines.push(format!(
//             "{}: {}",
//             key("Local IP", &cfg.label_local_ip),
//             local_ip().cyan()
//         ));
//     }

//     lines
// }

// ---------------------------------------------------------------------------
// Image rendering  (unchanged logic, unchanged public API)
// ---------------------------------------------------------------------------

/// # Errors
/// See module-level docs.
pub fn print_fetch_with_image<W: Write>(
    img: &DynamicImage,
    render: &RenderOptions,
    writer: &mut W,
) -> Result<()> {
    let target_px_height: u32 = 90;
    let (orig_w, orig_h) = (img.width(), img.height());
    let mut img_buf: Vec<u8> = Vec::new();

    if matches!(
        render.charset(),
        CharsetMode::Ascii | CharsetMode::Chinese | CharsetMode::Kanji
    ) {
        let target_cols: u32 = 50;
        let aspect = f64::from(orig_h) / f64::from(orig_w);
        let char_aspect_correction = 0.5_f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let target_rows = ((f64::from(target_cols) * aspect) * char_aspect_correction)
            .round()
            .max(1.0) as u32;
        let ascii_img = img.resize_exact(
            target_cols,
            target_rows,
            image::imageops::FilterType::Nearest,
        );
        let capped = render.with_width(target_cols);
        capped.render(&ascii_img, &mut img_buf)?;
    } else {
        let resized;
        let img_to_render = if orig_h > target_px_height {
            let scale = f64::from(target_px_height) / f64::from(orig_h);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let target_w = (f64::from(orig_w) * scale * 0.5).round().max(1.0) as u32;
            let rgba = img.to_rgba8();
            resized = image::DynamicImage::ImageRgba8(rgba).resize(
                target_w,
                target_px_height,
                image::imageops::FilterType::Nearest,
            );
            &resized
        } else {
            img
        };
        render.render(img_to_render, &mut img_buf)?;
    }

    let img_str = String::from_utf8_lossy(&img_buf);
    crate::fetch::print_with_left_block_writer(&img_str, writer)
}

// ---------------------------------------------------------------------------
// Layout writer
// ---------------------------------------------------------------------------

/// # Errors
/// IO write failure.
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
        let spaces = pad.saturating_sub(ansi_width(l));
        write!(writer, "{l}")?;
        write!(writer, "{:width$}", "", width = spaces)?;
        writeln!(writer, "{r}")?;
    }
    writeln!(writer)?;
    Ok(())
}

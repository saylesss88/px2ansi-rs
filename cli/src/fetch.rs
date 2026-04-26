//! `--fetch` mode: display system info alongside the image (or spinning animation).
use ansi_width::ansi_width;
use anyhow::Result;
use colored::Colorize;
use image::{imageops::FilterType, DynamicImage};
use px2ansi::{CharsetMode, RenderOptions};
use std::env;
use std::fs;
use std::io::Write;
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, RefreshKind, System,
};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct FetchConfig {
    pub show_header: bool,
    pub show_separator: bool,
    pub show_os: bool,
    pub show_kernel: bool,
    pub show_arch: bool,
    pub show_hostname: bool,
    pub show_cpu: bool,
    pub show_cpu_usage: bool,
    pub show_memory: bool,
    pub show_disk: bool,
    pub show_uptime: bool,
    pub show_processes: bool,
    pub show_shell: bool,
    pub show_locale: bool,
    pub show_local_ip: bool,
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
            show_hostname: false,
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
// Terminal width
// ---------------------------------------------------------------------------

/// Returns terminal width in columns. Tries `COLUMNS` env var first (set by
/// most shells), then the `libc` `TIOCGWINSZ` ioctl, then falls back to 80.
fn term_cols() -> usize {
    // 1. Try env var first
    if let Some(n) = env::var("COLUMNS")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .filter(|&n| n > 0)
    {
        return n;
    }

    // 2. Unix-specific ioctl fallback
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fds = [std::io::stderr().as_raw_fd(), std::io::stdout().as_raw_fd()];
        for fd in fds {
            let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
            if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_col > 0 {
                return ws.ws_col as usize;
            }
        }
    }

    80 // 3. Global fallback
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
// fetch_lines
// ---------------------------------------------------------------------------

#[must_use]
pub fn fetch_lines() -> Vec<String> {
    fetch_lines_with_config(&FetchConfig::load_default())
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn fetch_lines_with_config(cfg: &FetchConfig) -> Vec<String> {
    // Only refresh what we actually need — skips full process list,
    // network enumeration, and disk scanning unless requested.
    let mut refresh = RefreshKind::nothing();

    let need_cpu = cfg.show_cpu || cfg.show_cpu_usage;
    let need_mem = cfg.show_memory;
    let need_procs = cfg.show_processes;

    if need_cpu {
        refresh = refresh.with_cpu(CpuRefreshKind::nothing().with_cpu_usage());
    }
    if need_mem {
        refresh = refresh.with_memory(MemoryRefreshKind::nothing().with_ram());
    }
    if need_procs {
        refresh = refresh.with_processes(ProcessRefreshKind::nothing());
    }

    let sys = System::new_with_specifics(refresh);

    // Lazy: only hit the kernel for these if the field is shown.
    let disk_str = if cfg.show_disk {
        disk_usage()
    } else {
        String::new()
    };
    let ip_str = if cfg.show_local_ip {
        local_ip()
    } else {
        String::new()
    };

    let mut lines = Vec::new();
    let w = cfg.key_width;

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
        lines.push("─".repeat(24).white().to_string());
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
        disk_str.yellow()
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
        ip_str.cyan()
    );

    lines
}

// ---------------------------------------------------------------------------
// ANSI-safe line truncation
// ---------------------------------------------------------------------------

/// Truncate `s` so its *visible* width ≤ `max_cols`, appending "…" if cut.
/// Correctly skips over CSI escape sequences when counting width.
fn truncate_ansi(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    if ansi_width(s) <= max_cols {
        return s.to_string();
    }
    let target = max_cols.saturating_sub(1); // reserve 1 col for '…'
    let mut out = String::new();
    let mut col = 0usize;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            out.push(ch);
            if chars.peek() == Some(&'[') {
                if let Some(bracket) = chars.next() {
                    out.push(bracket);
                }
                for inner in chars.by_ref() {
                    out.push(inner);
                    if inner.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if col + w > target {
            break;
        }
        out.push(ch);
        col += w;
    }
    out.push_str("\x1b[0m"); // close any open colour
    out.push('…');
    out
}

// ---------------------------------------------------------------------------
// Image rendering
// ---------------------------------------------------------------------------

///  Render or IO failure.
/// # Errors
pub fn print_fetch_with_image<W: Write>(
    img: &DynamicImage,
    render: &RenderOptions,
    writer: &mut W,
) -> Result<()> {
    let cols = term_cols();
    let max_img_cols = u32::try_from(cols.saturating_sub(38).max(20)).unwrap_or(20);
    let (orig_w, orig_h) = (img.width(), img.height());
    let mut img_buf = Vec::new();

    if matches!(
        render.charset(),
        CharsetMode::Ascii | CharsetMode::Chinese | CharsetMode::Kanji
    ) {
        let target_cols = 50_u32.min(max_img_cols);
        let aspect = f64::from(orig_h) / f64::from(orig_w);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let target_rows = ((f64::from(target_cols) * aspect) * 0.5).max(1.0) as u32;
        let ascii_img = img.resize_exact(target_cols, target_rows, FilterType::Nearest);
        render
            .with_width(target_cols)
            .render(&ascii_img, &mut img_buf)?;
    } else {
        let max_px_w = max_img_cols * 2;
        let scale = (90.0 / f64::from(orig_h)).min(f64::from(max_px_w) / f64::from(orig_w));
        // Always apply the 0.5 terminal-column correction to width.
        // Half-block mode stacks 2 pixels vertically per row, but each
        // pixel still occupies 1 column horizontally — so to get square-ish
        // cells we halve the pixel width before handing to the renderer.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let tw = (f64::from(orig_w) * scale * 0.5).max(1.0) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let th = (f64::from(orig_h) * scale).max(1.0) as u32;
        let img_to_render = if tw != orig_w || th != orig_h {
            img.resize(tw, th, FilterType::Nearest)
        } else {
            img.clone()
        };
        render.render(&img_to_render, &mut img_buf)?;
    }

    let img_str = String::from_utf8_lossy(&img_buf);
    print_with_left_block_writer(&img_str, writer, cols)
}

// ---------------------------------------------------------------------------
// Layout writer
// ---------------------------------------------------------------------------

/// Minimum columns the right-hand fetch block must have before we fall back
/// to a stacked (image-above, text-below) layout.
// const MIN_RIGHT_BUDGET: usize = 20;
const MIN_RIGHT_BUDGET: usize = 12;

/// Gap (in terminal columns) between the image and the fetch text.
// const GAP: usize = 3;
const GAP: usize = 1;

/// # Errors
/// IO write failure.
pub fn print_with_left_block_writer<W: Write>(
    image_block: &str,
    writer: &mut W,
    cols: usize,
) -> Result<()> {
    let left_lines: Vec<&str> = image_block.lines().collect();
    let left_width = left_lines.iter().map(|l| ansi_width(l)).max().unwrap_or(0);
    let pad = left_width + GAP;
    let right_budget = cols.saturating_sub(pad);

    let info_lines = fetch_lines();

    if right_budget < MIN_RIGHT_BUDGET {
        writeln!(writer)?;
        for line in &left_lines {
            writeln!(writer, "{line}")?;
        }
        writeln!(writer)?;
        for line in &info_lines {
            writeln!(writer, "{line}")?;
        }
        writeln!(writer)?;
        return Ok(());
    }

    let max_lines = left_lines.len().max(info_lines.len());
    writeln!(writer)?;
    for i in 0..max_lines {
        let l = *left_lines.get(i).unwrap_or(&"");
        let r = info_lines.get(i).map_or("", String::as_str);

        write!(writer, "{l:<pad$}")?;
        writeln!(writer, "{}", truncate_ansi(r, right_budget))?;
    }
    writeln!(writer)?;
    Ok(())
}

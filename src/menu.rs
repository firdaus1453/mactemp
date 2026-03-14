// menu.rs — Menu bar title formatting and dropdown menu construction
//
// Builds the compact status bar title and the detailed dropdown menu
// that appears when clicking the menu bar item. Uses the
// `system_status_bar_macos` crate's Menu and MenuItem types.

use std::sync::mpsc::Sender;
use system_status_bar_macos::{Menu, MenuItem};

use crate::autostart;
use crate::smc::TemperatureData;
use crate::system_stats::SystemData;

/// Formats the compact menu bar title string.
///
/// Priority order: CPU Temp → CPU Usage → RAM Usage → GPU Temp (if available)
///
/// Examples:
///   "CPU 72°C | CPU 31% | RAM 55%"
///   "CPU 72°C | GPU 68°C | CPU 31% | RAM 55%"
///   "CPU 31% | RAM 55%"  (if temperatures unavailable)
pub fn build_title(temps: &TemperatureData, stats: &SystemData) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(4);

    // CPU Temperature (highest priority)
    if let Some(cpu_temp) = temps.cpu_temp {
        parts.push(format!("CPU {:.0}°C", cpu_temp));
    }

    // GPU Temperature (optional, shown if available)
    if let Some(gpu_temp) = temps.gpu_temp {
        parts.push(format!("GPU {:.0}°C", gpu_temp));
    }

    // CPU Usage
    parts.push(format!("CPU {:.0}%", stats.cpu_usage));

    // RAM Usage
    parts.push(format!("RAM {:.0}%", stats.ram_percentage));

    parts.join(" | ")
}

/// Builds the detailed dropdown menu shown when clicking the status bar item.
///
/// Layout:
///   ━━ System Stats ━━
///   CPU Usage: 31%
///   CPU Temperature: 72°C
///   GPU Temperature: 68°C
///   ─────────────
///   RAM Used: 9.2 GB
///   RAM Total: 16.0 GB
///   RAM Usage: 57%
///   ─────────────
///   Refresh
///   Auto Launch at Login: ✓ / ✗
///   ─────────────
///   Quit
pub fn build_menu(
    temps: &TemperatureData,
    stats: &SystemData,
    refresh_sender: Sender<()>,
) -> Menu {
    let mut items: Vec<MenuItem> = Vec::with_capacity(16);

    // Header
    items.push(MenuItem::new("━━ System Stats ━━", None, None));

    // CPU section
    items.push(MenuItem::new(
        format!("  CPU Usage: {:.1}%", stats.cpu_usage),
        Some(Box::new(|| {})),
        None,
    ));

    if let Some(cpu_temp) = temps.cpu_temp {
        items.push(MenuItem::new(
            format!("  CPU Temperature: {:.1}°C", cpu_temp),
            Some(Box::new(|| {})),
            None,
        ));
    } else {
        items.push(MenuItem::new("  CPU Temperature: N/A", Some(Box::new(|| {})), None));
    }

    // GPU section (only show if temperature is available)
    if let Some(gpu_temp) = temps.gpu_temp {
        items.push(MenuItem::new(
            format!("  GPU Temperature: {:.1}°C", gpu_temp),
            Some(Box::new(|| {})),
            None,
        ));
    }

    // Separator
    items.push(MenuItem::new("─────────────────", None, None));

    // RAM section
    items.push(MenuItem::new(
        format!("  RAM Used:  {:.1} GB", stats.ram_used_gb),
        Some(Box::new(|| {})),
        None,
    ));
    items.push(MenuItem::new(
        format!("  RAM Total: {:.1} GB", stats.ram_total_gb),
        Some(Box::new(|| {})),
        None,
    ));
    items.push(MenuItem::new(
        format!("  RAM Usage: {:.1}%", stats.ram_percentage),
        Some(Box::new(|| {})),
        None,
    ));

    // Separator
    items.push(MenuItem::new("─────────────────", None, None));

    // Refresh button — sends a tick to trigger immediate update
    items.push(MenuItem::new(
        "↻ Refresh",
        Some(Box::new(move || {
            let _ = refresh_sender.send(());
        })),
        None,
    ));

    // Auto-launch toggle
    let autostart_status = if autostart::is_enabled() { "✓" } else { "✗" };
    items.push(MenuItem::new(
        format!("  Auto Launch at Login: {}", autostart_status),
        Some(Box::new(|| {
            let new_state = autostart::toggle();
            eprintln!(
                "[mactemp] Auto-launch toggled: {}",
                if new_state { "enabled" } else { "disabled" }
            );
        })),
        None,
    ));

    // Separator
    items.push(MenuItem::new("─────────────────", None, None));

    // Quit button
    items.push(MenuItem::new(
        "✕ Quit",
        Some(Box::new(|| {
            eprintln!("[mactemp] Quitting...");
            std::process::exit(0);
        })),
        None,
    ));

    Menu::new(items)
}

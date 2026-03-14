// system_stats.rs — CPU usage and RAM monitoring via sysinfo
//
// Uses the `sysinfo` crate to read system-level CPU utilization and
// memory statistics. The sysinfo crate uses macOS native APIs under
// the hood (host_processor_info for CPU, host_statistics64 for memory).

use sysinfo::System;

/// Bytes-to-gigabytes conversion factor.
const BYTES_TO_GB: f64 = 1_073_741_824.0; // 1024^3

/// Holds CPU and RAM usage statistics.
#[derive(Debug, Clone)]
pub struct SystemData {
    /// Overall CPU usage as a percentage (0.0 - 100.0)
    pub cpu_usage: f32,
    /// Used RAM in gigabytes
    pub ram_used_gb: f64,
    /// Total RAM in gigabytes
    pub ram_total_gb: f64,
    /// RAM usage as a percentage (0.0 - 100.0)
    pub ram_percentage: f64,
}

/// Wrapper around `sysinfo::System` for periodic stats collection.
///
/// Keeps the System instance alive between reads so that CPU usage
/// calculations have a previous sample to compare against (sysinfo
/// needs at least two samples to compute CPU usage accurately).
pub struct StatsReader {
    system: System,
}

impl StatsReader {
    /// Creates a new stats reader with an initial system refresh.
    /// The first CPU usage reading may be 0% — accurate readings
    /// start after the second `read_stats()` call.
    pub fn new() -> Self {
        let mut system = System::new();
        // Initial refresh to establish baseline for CPU usage calculation
        system.refresh_cpu_usage();
        system.refresh_memory();
        Self { system }
    }

    /// Reads current CPU usage and RAM statistics.
    /// CPU usage is computed as the delta since the last call.
    pub fn read_stats(&mut self) -> SystemData {
        // Refresh CPU and memory data
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();

        // Calculate overall CPU usage from global CPU info
        let cpu_usage = self.system.global_cpu_usage();

        // Calculate RAM statistics
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();

        let ram_total_gb = total_memory as f64 / BYTES_TO_GB;
        let ram_used_gb = used_memory as f64 / BYTES_TO_GB;
        let ram_percentage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        SystemData {
            cpu_usage,
            ram_used_gb,
            ram_total_gb,
            ram_percentage,
        }
    }
}

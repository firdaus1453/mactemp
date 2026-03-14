// smc.rs — SMC (System Management Controller) temperature sensor reader
//
// Reads CPU and GPU temperatures from the macOS SMC hardware interface.
// The SMC is a chip present in every Mac that manages thermal sensors, fans,
// and power. Temperature values are read via IOKit's AppleSMC service using
// FourCC sensor keys (e.g., TC0D for CPU die, TG0D for GPU die).
//
// We read the "die" sensor (directly on the chip) rather than "proximity"
// (near the chip). The die sensor matches what `powermetrics` reports as
// "CPU die temperature" and is the more accurate reading.
//
// No root privileges are required — the SMC service is accessible to the
// current user on macOS.

use macsmc::Smc;

/// Holds temperature readings from SMC sensors.
/// Values are in Celsius. `None` means the sensor is unavailable.
#[derive(Debug, Clone)]
pub struct TemperatureData {
    pub cpu_temp: Option<f64>,
    pub gpu_temp: Option<f64>,
}

impl Default for TemperatureData {
    fn default() -> Self {
        Self {
            cpu_temp: None,
            gpu_temp: None,
        }
    }
}

/// Wrapper around the macsmc::Smc connection for reading temperatures.
/// Keeps the SMC connection alive to avoid reconnection overhead on each read.
///
/// NOTE: `Smc` is `!Send + !Sync`, so this must live on the main thread.
pub struct SmcReader {
    smc: Smc,
}

impl SmcReader {
    /// Attempts to connect to the SMC. Returns `None` if SMC is unavailable
    /// (e.g., running in a VM or on unsupported hardware).
    pub fn new() -> Option<Self> {
        match Smc::connect() {
            Ok(smc) => {
                eprintln!("[mactemp] SMC connection established");
                Some(Self { smc })
            }
            Err(err) => {
                eprintln!("[mactemp] SMC unavailable: {:?}", err);
                None
            }
        }
    }

    /// Reads CPU and GPU temperatures from SMC sensors.
    /// Each sensor is read independently — if one fails, the other
    /// can still return a valid reading.
    pub fn read_temperatures(&mut self) -> TemperatureData {
        let cpu_temp = self.read_cpu_temp();
        let gpu_temp = self.read_gpu_temp();

        TemperatureData { cpu_temp, gpu_temp }
    }

    /// Reads CPU die temperature from the SMC.
    /// The "die" sensor measures temperature directly on the CPU chip,
    /// matching what `powermetrics --samplers smc` reports as
    /// "CPU die temperature". Falls back to proximity if die reads 0.
    fn read_cpu_temp(&mut self) -> Option<f64> {
        match self.smc.cpu_temperature() {
            Ok(temps) => {
                // Prefer die temperature (matches powermetrics output)
                let die_value = *temps.die;
                if die_value > 0.0 && die_value < 150.0 {
                    return Some(die_value as f64);
                }
                // Fallback to proximity if die is unavailable
                let prox_value = *temps.proximity;
                if prox_value > 0.0 && prox_value < 150.0 {
                    Some(prox_value as f64)
                } else {
                    None
                }
            }
            Err(err) => {
                eprintln!("[mactemp] Failed to read CPU temperature: {:?}", err);
                None
            }
        }
    }

    /// Reads GPU die temperature from the SMC.
    /// May return `None` on Macs without a discrete GPU or if the
    /// GPU sensor is not available (e.g., dGPU is powered off).
    fn read_gpu_temp(&mut self) -> Option<f64> {
        match self.smc.gpu_temperature() {
            Ok(temps) => {
                // Prefer die temperature for consistency with CPU reading
                let die_value = *temps.die;
                if die_value > 0.0 && die_value < 150.0 {
                    return Some(die_value as f64);
                }
                // Fallback to proximity if die is unavailable
                let prox_value = *temps.proximity;
                if prox_value > 0.0 && prox_value < 150.0 {
                    Some(prox_value as f64)
                } else {
                    None
                }
            }
            Err(err) => {
                // GPU temperature is expected to fail on some hardware
                // (e.g., integrated-only GPU configurations), so we only
                // log at debug level to avoid noise.
                eprintln!("[mactemp] GPU temperature unavailable: {:?}", err);
                None
            }
        }
    }
}

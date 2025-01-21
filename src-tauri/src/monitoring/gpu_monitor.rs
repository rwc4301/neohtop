use std::fmt;

#[cfg(feature = "nvidia")]
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
#[cfg(feature = "nvidia")]
use nvml_wrapper::{Device, Nvml};
#[cfg(feature = "nvidia")]
use std::sync::Once;

use super::GPUStats;

#[cfg(feature = "nvidia")]
static NVML_INIT: Once = Once::new();

/// Monitors GPU statistics
#[derive(Debug)]
pub struct GPUMonitor {
    #[cfg(feature = "nvidia")]
    nvml: Option<Nvml>,
}

impl GPUMonitor {
    /// Creates a new GPU monitor instance
    pub fn new() -> Self {
        #[cfg(not(feature = "nvidia"))]
        {
            eprintln!("NVIDIA support is disabled");

            Self {}
        }
        #[cfg(feature = "nvidia")]
        {
            let nvml = match Nvml::init() {
                Ok(nvml) => {
                    NVML_INIT.call_once(|| {});
                    Some(nvml)
                }
                Err(e) => {
                    eprintln!("Failed to initialize NVML: {}", e);
                    None
                }
            };

            Self { nvml }
        }
    }

    /// Collects current GPU statistics
    pub fn collect_stats(&self) -> Option<Vec<GPUStats>> {
        #[cfg(feature = "nvidia")]
        {
            let nvml = self.nvml.as_ref()?;

            let devices = nvml.device_count().ok()?;
            let mut stats = Vec::with_capacity(devices as usize);

            for i in 0..devices {
                if let Ok(device) = nvml.device_by_index(i) {
                    if let Some(stat) = self.collect_device_stats(&device) {
                        stats.push(stat);
                    }
                }
            }

            Some(stats)
        }

        #[cfg(not(feature = "nvidia"))]
        {
            None
        }
    }

    #[cfg(feature = "nvidia")]
    fn collect_device_stats(&self, device: &Device) -> Option<GPUStats> {
        let name = device.name().ok()?;
        let utilization = device.utilization_rates().ok()?.gpu as f32;
        let memory = device.memory_info().ok()?;
        let temperature = device.temperature(TemperatureSensor::Gpu).ok()?;
        let power = device.power_usage().ok()?;

        Some(GPUStats {
            name,
            utilization,
            memory_used: memory.used,
            memory_total: memory.total,
            temperature: temperature as i32,
            power_usage: power,
        })
    }
}

// impl fmt::Debug for GPUMonitor {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("GPUMonitor")
//             #[cfg(feature = "nvidia")]
//             .field("has_nvidia_gpu", &self.nvml.is_some())
//             #[cfg(not(feature = "nvidia"))]
//             .field("has_nvidia_gpu", &false)
//             .finish()
//     }
// }

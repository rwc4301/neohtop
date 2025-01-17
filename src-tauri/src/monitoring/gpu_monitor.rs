use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::{Device, Nvml};
use std::sync::Once;

use super::GPUStats;

static NVML_INIT: Once = Once::new();

/// Monitors GPU statistics
#[derive(Debug)]
pub struct GPUMonitor {
    nvml: Option<Nvml>,
}

impl GPUMonitor {
    /// Creates a new GPU monitor instance
    pub fn new() -> Self {
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

    /// Collects current GPU statistics
    pub fn collect_stats(&self) -> Option<Vec<GPUStats>> {
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

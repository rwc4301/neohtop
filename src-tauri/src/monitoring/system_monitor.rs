//! System statistics monitoring
//!
//! This module handles collection and monitoring of system-wide statistics
//! including CPU, memory, network, and disk usage.

use super::gpu_monitor::GPUMonitor;
use super::SystemStats;
use std::fmt::Debug;
use std::path::Path;
use std::time::Instant;
use sysinfo::{CpuExt, Disk, DiskExt, NetworkExt, NetworksExt, SystemExt};

/// Monitors system-wide statistics
#[derive(Debug)]
pub struct SystemMonitor {
    /// Tracks network usage between updates
    last_network_update: (Instant, u64, u64),
    gpu_monitor: GPUMonitor,
}

impl SystemMonitor {
    /// Creates a new system monitor instance
    ///
    /// # Arguments
    ///
    /// * `sys` - System information provider for initial readings
    pub fn new(sys: &sysinfo::System) -> Self {
        let initial_rx: u64 = sys
            .networks()
            .iter()
            .map(|(_, data)| data.total_received())
            .sum();
        let initial_tx: u64 = sys
            .networks()
            .iter()
            .map(|(_, data)| data.total_transmitted())
            .sum();

        Self {
            last_network_update: (Instant::now(), initial_rx, initial_tx),
            gpu_monitor: GPUMonitor::new(),
        }
    }

    /// Collects current system statistics
    ///
    /// # Arguments
    ///
    /// * `sys` - System information provider
    pub fn collect_stats(&mut self, sys: &sysinfo::System) -> SystemStats {
        let (network_rx, network_tx) = self.calculate_network_stats(sys);
        let (disk_total, disk_used, disk_free) = self.calculate_disk_stats(sys);
        let gpu_stats = self.gpu_monitor.collect_stats();

        SystemStats {
            cpu_usage: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
            memory_total: sys.total_memory(),
            memory_used: sys.used_memory(),
            memory_free: sys.total_memory() - sys.used_memory(),
            memory_cached: sys.total_memory()
                - (sys.used_memory() + (sys.total_memory() - sys.used_memory())),
            uptime: sys.uptime(),
            load_avg: [
                sys.load_average().one,
                sys.load_average().five,
                sys.load_average().fifteen,
            ],
            network_rx_bytes: network_rx,
            network_tx_bytes: network_tx,
            disk_total_bytes: disk_total,
            disk_used_bytes: disk_used,
            disk_free_bytes: disk_free,
            gpu_stats,
        }
    }

    /// Filters disks based on platform-specific criteria
    #[cfg(not(target_os = "windows"))]
    fn filter_disks(disks: &[Disk]) -> Vec<&Disk> {
        disks
            .iter()
            .filter(|disk| disk.mount_point() == Path::new("/"))
            .collect()
    }

    /// Windows-specific disk filtering
    #[cfg(target_os = "windows")]
    fn filter_disks(disks: &[Disk]) -> Vec<&Disk> {
        disks.iter().collect()
    }

    /// Calculates network usage rates
    fn calculate_network_stats(&mut self, sys: &sysinfo::System) -> (u64, u64) {
        let current_rx: u64 = sys
            .networks()
            .iter()
            .map(|(_, data)| data.total_received())
            .sum();
        let current_tx: u64 = sys
            .networks()
            .iter()
            .map(|(_, data)| data.total_transmitted())
            .sum();

        let elapsed = self.last_network_update.0.elapsed().as_secs_f64();
        let rx_rate = ((current_rx - self.last_network_update.1) as f64 / elapsed) as u64;
        let tx_rate = ((current_tx - self.last_network_update.2) as f64 / elapsed) as u64;

        self.last_network_update = (Instant::now(), current_rx, current_tx);
        (rx_rate, tx_rate)
    }

    /// Calculates disk usage statistics
    fn calculate_disk_stats(&self, sys: &sysinfo::System) -> (u64, u64, u64) {
        let disks = Self::filter_disks(sys.disks());
        let total: u64 = disks.iter().map(|disk| disk.total_space()).sum();
        let used: u64 = disks
            .iter()
            .map(|disk| disk.total_space() - disk.available_space())
            .sum();
        let free: u64 = disks.iter().map(|disk| disk.available_space()).sum();
        (total, used, free)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::System;

    /// Tests creation of system monitor
    #[test]
    fn test_system_monitor_creation() {
        let sys = System::new();
        let monitor = SystemMonitor::new(&sys);
        assert!(monitor.last_network_update.1 >= 0);
        assert!(monitor.last_network_update.2 >= 0);
    }

    /// Tests system statistics collection
    #[test]
    fn test_stats_collection() {
        let mut sys = System::new();
        let mut monitor = SystemMonitor::new(&sys);
        sys.refresh_all();

        let stats = monitor.collect_stats(&sys);
        assert!(!stats.cpu_usage.is_empty());
        assert!(stats.memory_total > 0);
    }
}

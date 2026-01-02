use std::time::Instant;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::types::{CoreUsage, SystemStats};

/// CPU statistics collector using sysinfo
pub struct StatsCollector {
    system: System,
}

impl StatsCollector {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
        );

        // Initial refresh - sysinfo requires two refreshes to get meaningful values
        system.refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();

        Self { system }
    }

    /// Refresh CPU stats and return current snapshot
    pub fn refresh(&mut self) -> SystemStats {
        self.system.refresh_cpu_usage();

        let core_usages: Vec<CoreUsage> = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(idx, cpu)| CoreUsage {
                core_id: idx,
                usage_percent: cpu.cpu_usage(),
                frequency_mhz: Some(cpu.frequency()),
            })
            .collect();

        let total_usage = if core_usages.is_empty() {
            0.0
        } else {
            core_usages.iter().map(|u| u.usage_percent).sum::<f32>() / core_usages.len() as f32
        };

        SystemStats {
            timestamp: Instant::now(),
            core_usages,
            total_usage,
        }
    }

    /// Get number of CPUs
    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

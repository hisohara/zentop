use std::time::Instant;

/// CPU usage for a single core
#[derive(Debug, Clone)]
pub struct CoreUsage {
    pub core_id: usize,
    pub usage_percent: f32,
    pub frequency_mhz: Option<u64>,
}

/// Aggregated usage for a group of cores
#[derive(Debug, Clone)]
pub struct GroupUsage {
    pub name: String,
    pub id: usize,
    pub usage_percent: f32,
    pub min_usage: f32,
    pub max_usage: f32,
    pub core_count: usize,
    pub member_usages: Vec<CoreUsage>,
}

impl GroupUsage {
    pub fn from_cores(name: String, id: usize, usages: &[CoreUsage]) -> Self {
        let usage_percent = if usages.is_empty() {
            0.0
        } else {
            usages.iter().map(|u| u.usage_percent).sum::<f32>() / usages.len() as f32
        };

        let min_usage = usages
            .iter()
            .map(|u| u.usage_percent)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max_usage = usages
            .iter()
            .map(|u| u.usage_percent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        Self {
            name,
            id,
            usage_percent,
            min_usage,
            max_usage,
            core_count: usages.len(),
            member_usages: usages.to_vec(),
        }
    }
}

/// Complete system stats snapshot
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub timestamp: Instant,
    pub core_usages: Vec<CoreUsage>,
    pub total_usage: f32,
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            core_usages: Vec::new(),
            total_usage: 0.0,
        }
    }
}

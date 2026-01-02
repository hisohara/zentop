use super::types::{ZenGeneration, ZenTopology};

/// Detect AMD Zen generation based on topology characteristics
pub fn detect_zen_generation(topology: &ZenTopology) -> ZenGeneration {
    if topology.ccds.is_empty() || topology.total_cores == 0 {
        return ZenGeneration::Unknown;
    }

    // Calculate cores per CCD
    let cores_per_ccd = topology.total_cores / topology.ccds.len();

    // Zen/Zen+: 2 CCX per CCD, 4 cores per CCX = 8 cores per CCD
    // Zen2: 2 CCX per CCD, 4 cores per CCX = 8 cores per CCD (with I/O die)
    // Zen3/4/5: 1 CCX per CCD, 8 cores per CCX = 8 cores per CCD
    // Zen4c/5c: 2 CCX per CCD, 8 cores per CCX = 16 cores per CCD

    // Without CPUID detection, we can't distinguish Zen generations precisely
    // Use heuristics based on core counts
    match cores_per_ccd {
        1..=4 => ZenGeneration::Zen,
        5..=8 => ZenGeneration::Zen3, // Could be Zen2/3/4
        9..=16 => ZenGeneration::Zen4, // Zen4c/5c with more cores
        _ => ZenGeneration::Unknown,
    }
}

/// Get CPU family information from /proc/cpuinfo for better detection
pub fn detect_from_cpuinfo() -> Option<ZenGeneration> {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;

    // Look for AMD CPU family
    let mut family: Option<u32> = None;
    let mut model: Option<u32> = None;

    for line in cpuinfo.lines() {
        if line.starts_with("cpu family") {
            if let Some(val) = line.split(':').nth(1) {
                family = val.trim().parse().ok();
            }
        }
        if line.starts_with("model") && !line.starts_with("model name") {
            if let Some(val) = line.split(':').nth(1) {
                model = val.trim().parse().ok();
            }
        }
    }

    match (family?, model?) {
        // Family 23 (0x17): Zen, Zen+, Zen2
        (23, 1..=31) => Some(ZenGeneration::Zen),   // Zen, Zen+ (Summit Ridge, Pinnacle Ridge)
        (23, 49..=79) => Some(ZenGeneration::Zen2), // Zen2 (Matisse, Rome)
        (23, _) => Some(ZenGeneration::Zen2),

        // Family 25 (0x19): Zen3, Zen4
        (25, 0..=15) => Some(ZenGeneration::Zen3),  // Zen3 (Vermeer)
        (25, 16..=31) => Some(ZenGeneration::Zen4), // Zen4 (Raphael)
        (25, 32..=79) => Some(ZenGeneration::Zen3), // Zen3 (Milan)
        (25, _) => Some(ZenGeneration::Zen4),

        // Family 26 (0x1A): Zen5
        (26, _) => Some(ZenGeneration::Zen5),

        _ => None,
    }
}

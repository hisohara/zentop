/// Represents a single CPU processing unit (logical CPU)
#[derive(Debug, Clone)]
pub struct CpuCore {
    /// Logical CPU ID (matches sysinfo/OS index)
    pub id: usize,
    /// Physical core ID
    pub physical_id: usize,
    /// Core Complex Die ID
    pub ccd_id: usize,
    /// Core Complex ID (L3 cache group)
    pub ccx_id: usize,
    /// NUMA node ID
    pub numa_node: usize,
    /// Physical package/socket ID
    pub package_id: usize,
    /// SMT sibling (if SMT enabled)
    pub smt_sibling: Option<usize>,
}

impl Default for CpuCore {
    fn default() -> Self {
        Self {
            id: 0,
            physical_id: 0,
            ccd_id: 0,
            ccx_id: 0,
            numa_node: 0,
            package_id: 0,
            smt_sibling: None,
        }
    }
}

/// Core Complex Die - contains one or more CCX
#[derive(Debug, Clone)]
pub struct Ccd {
    pub id: usize,
    pub package_id: usize,
    /// Logical CPU IDs belonging to this CCD
    pub cores: Vec<usize>,
}

/// NUMA node grouping
#[derive(Debug, Clone)]
pub struct NumaNode {
    pub id: usize,
    /// Logical CPU IDs belonging to this NUMA node
    pub cores: Vec<usize>,
    /// Local memory in MB (if available)
    pub memory_mb: Option<u64>,
}

/// NPS (NUMA Per Socket) node
#[derive(Debug, Clone)]
pub struct NpsNode {
    pub id: usize,
    /// NUMA nodes in this NPS group
    pub numa_nodes: Vec<usize>,
    /// Logical CPU IDs belonging to this NPS node
    pub cores: Vec<usize>,
}

/// AMD Zen generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZenGeneration {
    Zen,
    Zen2,
    Zen3,
    Zen4,
    Zen5,
    Unknown,
}

impl std::fmt::Display for ZenGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZenGeneration::Zen => write!(f, "Zen"),
            ZenGeneration::Zen2 => write!(f, "Zen 2"),
            ZenGeneration::Zen3 => write!(f, "Zen 3"),
            ZenGeneration::Zen4 => write!(f, "Zen 4"),
            ZenGeneration::Zen5 => write!(f, "Zen 5"),
            ZenGeneration::Unknown => write!(f, "Unknown"),
        }
    }
}

/// NPS (NUMA Per Socket) mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpsMode {
    Nps1,
    Nps2,
    Nps4,
    Unknown,
}

impl std::fmt::Display for NpsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NpsMode::Nps1 => write!(f, "NPS1"),
            NpsMode::Nps2 => write!(f, "NPS2"),
            NpsMode::Nps4 => write!(f, "NPS4"),
            NpsMode::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Complete system topology
#[derive(Debug, Clone)]
pub struct ZenTopology {
    pub cpu_model: String,
    pub generation: ZenGeneration,
    pub nps_mode: NpsMode,
    pub cores: Vec<CpuCore>,
    pub ccds: Vec<Ccd>,
    pub numa_nodes: Vec<NumaNode>,
    pub nps_nodes: Vec<NpsNode>,
    pub packages: usize,
    pub total_cores: usize,
    pub total_threads: usize,
    pub smt_enabled: bool,
}

impl Default for ZenTopology {
    fn default() -> Self {
        Self {
            cpu_model: String::new(),
            generation: ZenGeneration::Unknown,
            nps_mode: NpsMode::Unknown,
            cores: Vec::new(),
            ccds: Vec::new(),
            numa_nodes: Vec::new(),
            nps_nodes: Vec::new(),
            packages: 0,
            total_cores: 0,
            total_threads: 0,
            smt_enabled: false,
        }
    }
}

use anyhow::{Context, Result};
use hwlocality::object::types::ObjectType;
use hwlocality::Topology;
use std::collections::HashMap;

use super::types::*;
use super::zen::detect_zen_generation;

/// Detect system topology using hwlocality
pub fn detect_topology() -> Result<ZenTopology> {
    let topo = Topology::new().context("Failed to initialize hwloc topology")?;

    let mut topology = ZenTopology::default();

    // Count packages (sockets)
    topology.packages = topo.objects_with_type(ObjectType::Package).count();
    if topology.packages == 0 {
        topology.packages = 1;
    }

    // Build CPU core mapping from PU objects
    let mut cores: Vec<CpuCore> = Vec::new();
    let mut physical_core_map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for pu in topo.objects_with_type(ObjectType::PU) {
        let os_idx = pu.os_index().map(|i| i as usize).unwrap_or(cores.len());

        // Find parent objects by walking up the tree
        let package_id = find_ancestor_index(&pu, ObjectType::Package).unwrap_or(0);
        // In Zen architecture, cores sharing L3 cache belong to the same CCD
        let l3_id = find_ancestor_index(&pu, ObjectType::L3Cache);
        let core_id = find_ancestor_index(&pu, ObjectType::Core);
        let numa_id = find_numa_node(&topo, &pu).unwrap_or(0);

        // CCD is determined by L3 cache sharing
        let ccd_id = l3_id.unwrap_or(0);
        let ccx_id = ccd_id; // In Zen, CCD and CCX are equivalent (same L3 cache group)
        let physical_id = core_id.unwrap_or(os_idx);

        let core = CpuCore {
            id: os_idx,
            physical_id,
            ccd_id,
            ccx_id,
            numa_node: numa_id,
            package_id,
            smt_sibling: None,
        };

        // Track physical cores for SMT detection
        physical_core_map
            .entry((package_id, physical_id))
            .or_default()
            .push(os_idx);

        cores.push(core);
    }

    // Sort cores by ID
    cores.sort_by_key(|c| c.id);

    // Detect SMT siblings
    for siblings in physical_core_map.values() {
        if siblings.len() > 1 {
            topology.smt_enabled = true;
            for &cpu_id in siblings {
                if let Some(core) = cores.iter_mut().find(|c| c.id == cpu_id) {
                    core.smt_sibling = siblings.iter().find(|&&s| s != cpu_id).copied();
                }
            }
        }
    }

    topology.total_threads = cores.len();
    topology.total_cores = physical_core_map.len();

    // Build CCD groups
    let mut ccd_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for core in &cores {
        ccd_map.entry(core.ccd_id).or_default().push(core.id);
    }
    topology.ccds = ccd_map
        .into_iter()
        .map(|(id, mut core_ids)| {
            core_ids.sort();
            let package_id = cores
                .iter()
                .find(|c| c.id == *core_ids.first().unwrap_or(&0))
                .map(|c| c.package_id)
                .unwrap_or(0);
            Ccd {
                id,
                package_id,
                cores: core_ids,
            }
        })
        .collect();
    topology.ccds.sort_by_key(|c| c.id);

    // Build NUMA node groups
    let mut numa_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for core in &cores {
        numa_map.entry(core.numa_node).or_default().push(core.id);
    }
    topology.numa_nodes = numa_map
        .into_iter()
        .map(|(id, mut core_ids)| {
            core_ids.sort();
            NumaNode {
                id,
                cores: core_ids,
                memory_mb: None,
            }
        })
        .collect();
    topology.numa_nodes.sort_by_key(|n| n.id);

    // Detect NPS mode
    topology.nps_mode = detect_nps_mode(topology.numa_nodes.len(), topology.packages);

    // Build NPS node groups
    topology.nps_nodes = build_nps_nodes(&topology.numa_nodes, topology.packages);

    topology.cores = cores;

    // Detect Zen generation
    topology.generation = detect_zen_generation(&topology);

    Ok(topology)
}

fn find_ancestor_index(
    obj: &hwlocality::object::TopologyObject,
    target_type: ObjectType,
) -> Option<usize> {
    let mut current = obj.parent();
    while let Some(parent) = current {
        if parent.object_type() == target_type {
            return Some(parent.logical_index() as usize);
        }
        current = parent.parent();
    }
    None
}

fn find_numa_node(
    topo: &Topology,
    obj: &hwlocality::object::TopologyObject,
) -> Option<usize> {
    // First try direct ancestor
    if let Some(idx) = find_ancestor_index(obj, ObjectType::NUMANode) {
        return Some(idx);
    }

    // Check NUMA nodes by cpuset intersection
    let obj_cpuset = obj.cpuset()?;
    for numa in topo.objects_with_type(ObjectType::NUMANode) {
        if let Some(numa_cpuset) = numa.cpuset() {
            // Check if obj's cpuset is a subset of numa's cpuset
            if numa_cpuset.includes(obj_cpuset) {
                return Some(numa.logical_index() as usize);
            }
        }
    }
    None
}

fn detect_nps_mode(numa_count: usize, package_count: usize) -> NpsMode {
    if package_count == 0 {
        return NpsMode::Unknown;
    }
    let nodes_per_socket = numa_count / package_count;
    match nodes_per_socket {
        0 | 1 => NpsMode::Nps1,
        2 => NpsMode::Nps2,
        4 => NpsMode::Nps4,
        _ => NpsMode::Unknown,
    }
}

fn build_nps_nodes(numa_nodes: &[NumaNode], packages: usize) -> Vec<NpsNode> {
    if packages == 0 || numa_nodes.is_empty() {
        return vec![NpsNode {
            id: 0,
            numa_nodes: numa_nodes.iter().map(|n| n.id).collect(),
            cores: numa_nodes.iter().flat_map(|n| n.cores.clone()).collect(),
        }];
    }

    let nodes_per_socket = numa_nodes.len() / packages;
    if nodes_per_socket == 0 {
        return vec![NpsNode {
            id: 0,
            numa_nodes: numa_nodes.iter().map(|n| n.id).collect(),
            cores: numa_nodes.iter().flat_map(|n| n.cores.clone()).collect(),
        }];
    }

    let mut nps_nodes = Vec::new();

    for socket in 0..packages {
        let start = socket * nodes_per_socket;
        let end = start + nodes_per_socket;
        let socket_numa: Vec<&NumaNode> = numa_nodes
            .iter()
            .filter(|n| n.id >= start && n.id < end)
            .collect();

        let mut all_cores: Vec<usize> = socket_numa
            .iter()
            .flat_map(|n| n.cores.clone())
            .collect();
        all_cores.sort();
        all_cores.dedup();

        nps_nodes.push(NpsNode {
            id: socket,
            numa_nodes: socket_numa.iter().map(|n| n.id).collect(),
            cores: all_cores,
        });
    }

    nps_nodes
}

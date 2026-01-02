use ratatui::{buffer::Buffer, layout::Rect};

use crate::stats::{GroupUsage, SystemStats};
use crate::topology::ZenTopology;
use crate::ui::theme::Theme;
use crate::ui::views::ccd_view::render_grouped_view;

/// Render NUMA grouped view
pub fn render_numa_view(
    area: Rect,
    buf: &mut Buffer,
    topology: &ZenTopology,
    stats: &SystemStats,
    show_smt: bool,
    scroll_offset: usize,
    theme: &Theme,
) {
    if area.height == 0 || topology.numa_nodes.is_empty() {
        return;
    }

    let mut groups: Vec<GroupUsage> = Vec::new();

    for numa in &topology.numa_nodes {
        let cores_in_numa: Vec<_> = if show_smt {
            numa.cores.clone()
        } else {
            numa.cores
                .iter()
                .filter(|&&cpu_id| {
                    topology
                        .cores
                        .get(cpu_id)
                        .map(|c| c.smt_sibling.map_or(true, |s| c.id < s))
                        .unwrap_or(true)
                })
                .copied()
                .collect()
        };

        let usages: Vec<_> = cores_in_numa
            .iter()
            .filter_map(|&cpu_id| {
                stats.core_usages.get(cpu_id).map(|u| crate::stats::CoreUsage {
                    core_id: cpu_id,
                    usage_percent: u.usage_percent,
                    frequency_mhz: u.frequency_mhz,
                })
            })
            .collect();

        let mem_str = numa
            .memory_mb
            .map(|m| format!(" - {} MB", m))
            .unwrap_or_default();

        let name = format!("NUMA {}{}", numa.id, mem_str);
        groups.push(GroupUsage::from_cores(name, numa.id, &usages));
    }

    render_grouped_view(area, buf, &groups, scroll_offset, theme);
}

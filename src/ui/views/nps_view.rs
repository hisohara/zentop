use ratatui::{buffer::Buffer, layout::Rect};

use crate::app::DisplayMode;
use crate::stats::{GroupUsage, SystemStats};
use crate::topology::ZenTopology;
use crate::ui::theme::Theme;
use crate::ui::views::ccd_view::render_grouped_view;

/// Render NPS grouped view
pub fn render_nps_view(
    area: Rect,
    buf: &mut Buffer,
    topology: &ZenTopology,
    stats: &SystemStats,
    show_smt: bool,
    scroll_offset: usize,
    display_mode: DisplayMode,
    theme: &Theme,
    socket_filter: Option<usize>,
) {
    if area.height == 0 || topology.nps_nodes.is_empty() {
        return;
    }

    let mut groups: Vec<GroupUsage> = Vec::new();

    for nps in &topology.nps_nodes {
        // Filter by socket: check first core's package_id
        if let Some(socket) = socket_filter {
            let nps_socket = nps.cores.first().and_then(|&cpu_id| {
                topology.cores.get(cpu_id).map(|c| c.package_id)
            });
            if nps_socket != Some(socket) {
                continue;
            }
        }
        let cores_in_nps: Vec<_> = if show_smt {
            nps.cores.clone()
        } else {
            nps.cores
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

        let usages: Vec<_> = cores_in_nps
            .iter()
            .filter_map(|&cpu_id| {
                stats.core_usages.get(cpu_id).map(|u| crate::stats::CoreUsage {
                    core_id: cpu_id,
                    usage_percent: u.usage_percent,
                    frequency_mhz: u.frequency_mhz,
                })
            })
            .collect();

        let name = format!("NPS {} (NUMA: {:?})", nps.id, nps.numa_nodes);
        groups.push(GroupUsage::from_cores(name, nps.id, &usages));
    }

    render_grouped_view(area, buf, &groups, scroll_offset, display_mode, theme);
}

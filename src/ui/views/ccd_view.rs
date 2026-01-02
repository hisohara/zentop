use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::stats::{GroupUsage, SystemStats};
use crate::topology::ZenTopology;
use crate::ui::theme::Theme;
use crate::ui::widgets::{CpuBar, GroupHeader};

/// Render CCD grouped view
pub fn render_ccd_view(
    area: Rect,
    buf: &mut Buffer,
    topology: &ZenTopology,
    stats: &SystemStats,
    show_smt: bool,
    scroll_offset: usize,
    theme: &Theme,
) {
    if area.height == 0 || topology.ccds.is_empty() {
        return;
    }

    // Build group usages
    let mut groups: Vec<GroupUsage> = Vec::new();

    for ccd in &topology.ccds {
        let cores_in_ccd: Vec<_> = if show_smt {
            ccd.cores.clone()
        } else {
            ccd.cores
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

        let usages: Vec<_> = cores_in_ccd
            .iter()
            .filter_map(|&cpu_id| {
                stats.core_usages.get(cpu_id).map(|u| crate::stats::CoreUsage {
                    core_id: cpu_id,
                    usage_percent: u.usage_percent,
                    frequency_mhz: u.frequency_mhz,
                })
            })
            .collect();

        groups.push(GroupUsage::from_cores(
            format!("CCD {}", ccd.id),
            ccd.id,
            &usages,
        ));
    }

    render_grouped_view(area, buf, &groups, scroll_offset, theme);
}

/// Common rendering for grouped views
pub fn render_grouped_view(
    area: Rect,
    buf: &mut Buffer,
    groups: &[GroupUsage],
    scroll_offset: usize,
    theme: &Theme,
) {
    if area.height == 0 {
        return;
    }

    // Calculate total lines needed
    let mut total_lines = 0;
    for group in groups {
        total_lines += 1; // Header
        total_lines += group.member_usages.len(); // Core bars
        total_lines += 1; // Spacing
    }

    let visible_height = area.height as usize;
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll = scroll_offset.min(max_scroll);

    let mut current_line = 0;
    let mut y = area.y;

    for group in groups {
        // Group header
        if current_line >= scroll && y < area.y + area.height {
            let header_area = Rect::new(area.x, y, area.width, 1);
            GroupHeader::new(&group.name, group.usage_percent, group.core_count, theme)
                .render(header_area, buf);
            y += 1;
        }
        current_line += 1;

        // Core bars
        for usage in &group.member_usages {
            if current_line >= scroll && y < area.y + area.height {
                let label = format!("CPU{:2}", usage.core_id);
                let bar_area = Rect::new(area.x, y, area.width, 1);
                CpuBar::new(&label, usage.usage_percent, theme).render(bar_area, buf);
                y += 1;
            }
            current_line += 1;
        }

        // Spacing
        if current_line >= scroll && y < area.y + area.height {
            y += 1;
        }
        current_line += 1;
    }
}

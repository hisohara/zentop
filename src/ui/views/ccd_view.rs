use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::app::DisplayMode;
use crate::stats::{GroupUsage, SystemStats};
use crate::topology::ZenTopology;
use crate::ui::layout::GridLayout;
use crate::ui::theme::Theme;
use crate::ui::widgets::{CompactCpuBar, CpuBar, GroupHeader, HeatmapCell};

/// Render CCD grouped view
pub fn render_ccd_view(
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
    if area.height == 0 || topology.ccds.is_empty() {
        return;
    }

    // Build group usages (filtered by socket if specified)
    let mut groups: Vec<GroupUsage> = Vec::new();

    for ccd in topology.ccds.iter().filter(|c| socket_filter.map_or(true, |s| c.package_id == s)) {
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

    render_grouped_view(area, buf, &groups, scroll_offset, display_mode, theme);
}

/// Common rendering for grouped views
pub fn render_grouped_view(
    area: Rect,
    buf: &mut Buffer,
    groups: &[GroupUsage],
    scroll_offset: usize,
    display_mode: DisplayMode,
    theme: &Theme,
) {
    if area.height == 0 {
        return;
    }

    match display_mode {
        DisplayMode::Full => {
            render_grouped_full(area, buf, groups, scroll_offset, theme);
        }
        DisplayMode::Compact => {
            render_grouped_compact(area, buf, groups, scroll_offset, theme);
        }
        DisplayMode::Heatmap => {
            render_grouped_heatmap(area, buf, groups, scroll_offset, theme);
        }
    }
}

/// Full mode: 1 core per line within each group
fn render_grouped_full(
    area: Rect,
    buf: &mut Buffer,
    groups: &[GroupUsage],
    scroll_offset: usize,
    theme: &Theme,
) {
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

/// Compact mode: multi-column within each group
fn render_grouped_compact(
    area: Rect,
    buf: &mut Buffer,
    groups: &[GroupUsage],
    scroll_offset: usize,
    theme: &Theme,
) {
    let layout = GridLayout::compact(area, 16); // Use 16 cols

    // Calculate total lines needed
    let mut total_lines = 0;
    for group in groups {
        total_lines += 1; // Header
        let rows_for_cores = (group.member_usages.len() + layout.cols - 1) / layout.cols;
        total_lines += rows_for_cores;
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

        // Core bars in multi-column
        let rows_for_cores = (group.member_usages.len() + layout.cols - 1) / layout.cols;
        for row in 0..rows_for_cores {
            if current_line >= scroll && y < area.y + area.height {
                for col in 0..layout.cols {
                    let idx = row * layout.cols + col;
                    if idx >= group.member_usages.len() {
                        break;
                    }
                    let usage = &group.member_usages[idx];
                    let x = area.x + (col as u16 * layout.cell_width);
                    let cell_area = Rect::new(x, y, layout.cell_width, 1);
                    CompactCpuBar::new(usage.core_id, usage.usage_percent, theme)
                        .render(cell_area, buf);
                }
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

/// Row label width for heatmap mode
const HEATMAP_ROW_LABEL_WIDTH: u16 = 5;

/// Heatmap mode: ultra-dense within each group with labels
fn render_grouped_heatmap(
    area: Rect,
    buf: &mut Buffer,
    groups: &[GroupUsage],
    scroll_offset: usize,
    theme: &Theme,
) {
    // Reserve space for row labels
    let content_width = area.width.saturating_sub(HEATMAP_ROW_LABEL_WIDTH);
    let content_x = area.x + HEATMAP_ROW_LABEL_WIDTH;

    let layout = GridLayout::heatmap(
        Rect::new(content_x, area.y, content_width, area.height),
        96,
    );

    // Calculate total lines needed (including column headers for each group)
    let mut total_lines = 0;
    for group in groups {
        total_lines += 1; // Group header
        total_lines += 1; // Column header
        let rows_for_cores = (group.member_usages.len() + layout.cols - 1) / layout.cols;
        total_lines += rows_for_cores;
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

        // Column header for this group
        if current_line >= scroll && y < area.y + area.height {
            // Show column markers every 16 columns, including the final count
            for col_marker in (0..=layout.cols).step_by(16) {
                let x = content_x + (col_marker as u16 * layout.cell_width);
                let label = format!("+{}", col_marker);
                if x + label.len() as u16 <= area.x + area.width {
                    buf.set_string(x, y, &label, theme.dim_style());
                }
            }
            y += 1;
        }
        current_line += 1;

        // Core cells in multi-column with row labels
        let rows_for_cores = (group.member_usages.len() + layout.cols - 1) / layout.cols;
        for row in 0..rows_for_cores {
            if current_line >= scroll && y < area.y + area.height {
                // Row label: show first core ID of this row
                if let Some(first_usage) = group.member_usages.get(row * layout.cols) {
                    let label = format!("{:>4}:", first_usage.core_id);
                    buf.set_string(area.x, y, &label, theme.dim_style());
                }

                for col in 0..layout.cols {
                    let idx = row * layout.cols + col;
                    if idx >= group.member_usages.len() {
                        break;
                    }
                    let usage = &group.member_usages[idx];
                    let x = content_x + (col as u16 * layout.cell_width);
                    let cell_area = Rect::new(x, y, layout.cell_width, 1);
                    HeatmapCell::new(usage.core_id, usage.usage_percent, theme)
                        .render(cell_area, buf);
                }
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

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::app::DisplayMode;
use crate::stats::SystemStats;
use crate::topology::{CpuCore, ZenTopology};
use crate::ui::layout::GridLayout;
use crate::ui::theme::Theme;
use crate::ui::widgets::{CompactCpuBar, CpuBar, HeatmapCell};

/// Render individual core view
pub fn render_core_view(
    area: Rect,
    buf: &mut Buffer,
    topology: &ZenTopology,
    stats: &SystemStats,
    show_smt: bool,
    scroll_offset: usize,
    display_mode: DisplayMode,
    theme: &Theme,
) {
    if area.height == 0 {
        return;
    }

    // Filter cores based on SMT setting
    let cores_to_show: Vec<_> = filter_cores(topology, show_smt);

    match display_mode {
        DisplayMode::Full => {
            render_full_view(area, buf, &cores_to_show, stats, scroll_offset, theme);
        }
        DisplayMode::Compact => {
            render_compact_view(area, buf, &cores_to_show, stats, scroll_offset, theme);
        }
        DisplayMode::Heatmap => {
            render_heatmap_view(area, buf, &cores_to_show, stats, scroll_offset, theme);
        }
    }
}

/// Filter cores based on SMT setting
pub fn filter_cores(topology: &ZenTopology, show_smt: bool) -> Vec<&CpuCore> {
    if show_smt {
        topology.cores.iter().collect()
    } else {
        topology
            .cores
            .iter()
            .filter(|c| c.smt_sibling.map_or(true, |s| c.id < s))
            .collect()
    }
}

/// Render full view (original 1-core-per-line)
fn render_full_view(
    area: Rect,
    buf: &mut Buffer,
    cores: &[&CpuCore],
    stats: &SystemStats,
    scroll_offset: usize,
    theme: &Theme,
) {
    let visible_count = area.height as usize;
    let total_cores = cores.len();

    let max_scroll = total_cores.saturating_sub(visible_count);
    let scroll = scroll_offset.min(max_scroll);

    for (i, core) in cores.iter().skip(scroll).take(visible_count).enumerate() {
        let usage = stats
            .core_usages
            .get(core.id)
            .map(|u| u.usage_percent)
            .unwrap_or(0.0);

        let label = format!("CPU{:2}", core.id);
        let bar_area = Rect::new(area.x, area.y + i as u16, area.width, 1);

        CpuBar::new(&label, usage, theme).render(bar_area, buf);
    }
}

/// Render compact view (16 cores per row)
fn render_compact_view(
    area: Rect,
    buf: &mut Buffer,
    cores: &[&CpuCore],
    stats: &SystemStats,
    scroll_offset: usize,
    theme: &Theme,
) {
    let layout = GridLayout::compact(area, cores.len());
    let visible_rows = area.height as usize;

    let max_scroll = layout.rows.saturating_sub(visible_rows);
    let scroll = scroll_offset.min(max_scroll);

    for (i, core) in cores.iter().enumerate() {
        let row = i / layout.cols;

        // Skip rows before scroll offset
        if row < scroll {
            continue;
        }

        // Stop if we've filled the visible area
        let display_row = row - scroll;
        if display_row >= visible_rows {
            break;
        }

        let col = i % layout.cols;
        let x = area.x + (col as u16 * layout.cell_width);
        let y = area.y + display_row as u16;

        let cell_area = Rect::new(x, y, layout.cell_width, 1);

        let usage = stats
            .core_usages
            .get(core.id)
            .map(|u| u.usage_percent)
            .unwrap_or(0.0);

        CompactCpuBar::new(core.id, usage, theme).render(cell_area, buf);
    }
}

/// Row label width for heatmap mode
const HEATMAP_ROW_LABEL_WIDTH: u16 = 5;

/// Render heatmap view (96 cores per row with labels)
fn render_heatmap_view(
    area: Rect,
    buf: &mut Buffer,
    cores: &[&CpuCore],
    stats: &SystemStats,
    scroll_offset: usize,
    theme: &Theme,
) {
    if area.height < 2 || area.width < HEATMAP_ROW_LABEL_WIDTH + 16 {
        return;
    }

    // Reserve space for row labels and column header
    let content_width = area.width.saturating_sub(HEATMAP_ROW_LABEL_WIDTH);
    let content_area = Rect::new(
        area.x + HEATMAP_ROW_LABEL_WIDTH,
        area.y + 1, // +1 for column header
        content_width,
        area.height.saturating_sub(1),
    );

    let layout = GridLayout::heatmap(content_area, cores.len());
    let visible_rows = content_area.height as usize;

    let max_scroll = layout.rows.saturating_sub(visible_rows);
    let scroll = scroll_offset.min(max_scroll);

    // Render column header
    render_heatmap_column_header(area, buf, layout.cols, layout.cell_width, theme);

    // Render cores with row labels
    for (i, core) in cores.iter().enumerate() {
        let row = i / layout.cols;

        if row < scroll {
            continue;
        }

        let display_row = row - scroll;
        if display_row >= visible_rows {
            break;
        }

        let col = i % layout.cols;

        // Render row label at the start of each row
        if col == 0 {
            let first_core_id = cores.get(row * layout.cols).map(|c| c.id).unwrap_or(0);
            let label = format!("{:>4}:", first_core_id);
            let y = content_area.y + display_row as u16;
            buf.set_string(area.x, y, &label, theme.dim_style());
        }

        let x = content_area.x + (col as u16 * layout.cell_width);
        let y = content_area.y + display_row as u16;

        let cell_area = Rect::new(x, y, layout.cell_width, 1);

        let usage = stats
            .core_usages
            .get(core.id)
            .map(|u| u.usage_percent)
            .unwrap_or(0.0);

        HeatmapCell::new(core.id, usage, theme).render(cell_area, buf);
    }
}

/// Render column header for heatmap (shows +0, +16, +32, etc.)
fn render_heatmap_column_header(
    area: Rect,
    buf: &mut Buffer,
    cols: usize,
    cell_width: u16,
    theme: &Theme,
) {
    let header_y = area.y;
    let content_x = area.x + HEATMAP_ROW_LABEL_WIDTH;

    // Show column markers every 16 columns, including the final count
    for col_marker in (0..=cols).step_by(16) {
        let x = content_x + (col_marker as u16 * cell_width);
        let label = format!("+{}", col_marker);
        if x + label.len() as u16 <= area.x + area.width {
            buf.set_string(x, header_y, &label, theme.dim_style());
        }
    }
}

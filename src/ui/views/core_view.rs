use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::stats::SystemStats;
use crate::topology::ZenTopology;
use crate::ui::theme::Theme;
use crate::ui::widgets::CpuBar;

/// Render individual core view
pub fn render_core_view(
    area: Rect,
    buf: &mut Buffer,
    topology: &ZenTopology,
    stats: &SystemStats,
    show_smt: bool,
    scroll_offset: usize,
    theme: &Theme,
) {
    if area.height == 0 {
        return;
    }

    // Filter cores based on SMT setting
    let cores_to_show: Vec<_> = if show_smt {
        topology.cores.iter().collect()
    } else {
        // Show only first thread of each physical core
        topology
            .cores
            .iter()
            .filter(|c| c.smt_sibling.map_or(true, |s| c.id < s))
            .collect()
    };

    let visible_count = area.height as usize;
    let total_cores = cores_to_show.len();

    // Clamp scroll offset
    let max_scroll = total_cores.saturating_sub(visible_count);
    let scroll = scroll_offset.min(max_scroll);

    for (i, core) in cores_to_show.iter().skip(scroll).take(visible_count).enumerate() {
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

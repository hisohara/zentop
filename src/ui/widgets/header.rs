use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};

use crate::app::{DisplayMode, ViewMode};
use crate::topology::ZenTopology;
use crate::ui::theme::Theme;

/// Application header widget
pub struct Header<'a> {
    topology: &'a ZenTopology,
    view_mode: ViewMode,
    display_mode: DisplayMode,
    show_smt: bool,
    total_usage: f32,
    theme: &'a Theme,
}

impl<'a> Header<'a> {
    pub fn new(
        topology: &'a ZenTopology,
        view_mode: ViewMode,
        display_mode: DisplayMode,
        show_smt: bool,
        total_usage: f32,
        theme: &'a Theme,
    ) -> Self {
        Self {
            topology,
            view_mode,
            display_mode,
            show_smt,
            total_usage,
            theme,
        }
    }
}

impl Widget for Header<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        // Line 1: Title and system info
        let title = format!(
            " zentop {} | {} | {} | {}C/{}T | SMT: {} ",
            env!("CARGO_PKG_VERSION"),
            self.topology.cpu_model,
            self.topology.nps_mode,
            self.topology.total_cores,
            self.topology.total_threads,
            if self.topology.smt_enabled {
                "ON"
            } else {
                "OFF"
            }
        );

        let title_style = self.theme.header_style();
        buf.set_string(area.x, area.y, &title, title_style);

        // Fill rest of line
        let remaining = area.width.saturating_sub(title.len() as u16);
        if remaining > 0 {
            buf.set_string(
                area.x + title.len() as u16,
                area.y,
                " ".repeat(remaining as usize),
                title_style,
            );
        }

        // Line 2: View mode and total usage
        if area.height >= 2 {
            let mode_str = match self.view_mode {
                ViewMode::Core => "[c]ore",
                ViewMode::Ccd => "cc[d]",
                ViewMode::Nps => "[n]ps",
            };

            let smt_str = if self.show_smt { "All" } else { "Physical" };

            let display_str = match self.display_mode {
                DisplayMode::Full => "Full",
                DisplayMode::Compact => "Compact",
                DisplayMode::Heatmap => "Heatmap",
            };

            let status = format!(
                " View: {} | Cores: {} | Mode: [m]{} | Total: {:.1}% | [h]elp [q]uit ",
                mode_str, smt_str, display_str, self.total_usage
            );

            let status_style = Style::default()
                .fg(self.theme.text_normal)
                .bg(self.theme.border);

            buf.set_string(area.x, area.y + 1, &status, status_style);

            let remaining = area.width.saturating_sub(status.len() as u16);
            if remaining > 0 {
                buf.set_string(
                    area.x + status.len() as u16,
                    area.y + 1,
                    " ".repeat(remaining as usize),
                    status_style,
                );
            }
        }
    }
}

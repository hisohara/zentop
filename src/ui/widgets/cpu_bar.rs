use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};

use crate::ui::theme::Theme;

/// CPU usage bar widget (htop-style)
pub struct CpuBar<'a> {
    label: &'a str,
    usage: f32,
    theme: &'a Theme,
    show_percentage: bool,
}

impl<'a> CpuBar<'a> {
    pub fn new(label: &'a str, usage: f32, theme: &'a Theme) -> Self {
        Self {
            label,
            usage,
            theme,
            show_percentage: true,
        }
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }
}

impl Widget for CpuBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        // Format: "CPU 0  [||||||||          ] 65.2%"
        let label_width = 7; // "CPU XX "
        let percent_width = if self.show_percentage { 7 } else { 0 }; // " XX.X%"
        let bracket_width = 2; // "[]"
        let bar_width = area
            .width
            .saturating_sub(label_width + percent_width + bracket_width) as usize;

        if bar_width < 2 {
            return;
        }

        let x = area.x;
        let y = area.y;

        // Render label
        let label_str = format!("{:>6} ", self.label);
        buf.set_string(x, y, &label_str, self.theme.text_style());

        // Render opening bracket
        let bar_start = x + label_width;
        buf.set_string(bar_start, y, "[", self.theme.dim_style());

        // Calculate filled portion
        let filled = ((self.usage / 100.0) * bar_width as f32).round() as usize;
        let filled = filled.min(bar_width);

        // Render bar content
        let bar_color = self.theme.usage_color(self.usage);
        let bar_style = Style::default().fg(bar_color);

        for i in 0..bar_width {
            let char = if i < filled { '|' } else { ' ' };
            buf.set_string(bar_start + 1 + i as u16, y, char.to_string(), bar_style);
        }

        // Render closing bracket
        buf.set_string(bar_start + 1 + bar_width as u16, y, "]", self.theme.dim_style());

        // Render percentage
        if self.show_percentage {
            let percent_str = format!("{:5.1}%", self.usage);
            let percent_x = bar_start + 2 + bar_width as u16;
            buf.set_string(percent_x, y, &percent_str, self.theme.text_style());
        }
    }
}

/// Compact CPU bar for multi-column layout (12 chars: "###[===]XX% ")
pub struct CompactCpuBar<'a> {
    core_id: usize,
    usage: f32,
    theme: &'a Theme,
}

impl<'a> CompactCpuBar<'a> {
    pub fn new(core_id: usize, usage: f32, theme: &'a Theme) -> Self {
        Self {
            core_id,
            usage,
            theme,
        }
    }
}

impl Widget for CompactCpuBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        // Format: "###[===]XX% " (12 chars)
        // Label: 3 chars (core id)
        // Bar: 5 chars ([===])
        // Percent: 3 chars (XX%)
        // Space: 1 char

        let x = area.x;
        let y = area.y;

        // Render core ID (right-aligned, 3 chars)
        let label = format!("{:>3}", self.core_id);
        buf.set_string(x, y, &label, self.theme.text_style());

        // Render bar [===] (5 chars)
        let bar_width = 3;
        let filled = ((self.usage / 100.0) * bar_width as f32).round() as usize;
        let filled = filled.min(bar_width);

        let bar_color = self.theme.usage_color(self.usage);
        let bar_style = Style::default().fg(bar_color);

        buf.set_string(x + 3, y, "[", self.theme.dim_style());
        for i in 0..bar_width {
            let char = if i < filled { '|' } else { ' ' };
            buf.set_string(x + 4 + i as u16, y, char.to_string(), bar_style);
        }
        buf.set_string(x + 7, y, "]", self.theme.dim_style());

        // Render percentage (3 chars: XX%)
        let pct = format!("{:>2.0}%", self.usage);
        buf.set_string(x + 8, y, &pct, self.theme.text_style());

        // Trailing space
        buf.set_string(x + 11, y, " ", self.theme.text_style());
    }
}

/// Heatmap cell for ultra-dense display (3 chars: "█ ")
pub struct HeatmapCell<'a> {
    core_id: usize,
    usage: f32,
    theme: &'a Theme,
}

impl<'a> HeatmapCell<'a> {
    pub fn new(core_id: usize, usage: f32, theme: &'a Theme) -> Self {
        Self {
            core_id,
            usage,
            theme,
        }
    }
}

impl Widget for HeatmapCell<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Use Unicode block characters for gradients
        let char = match self.usage {
            u if u < 12.5 => ' ',
            u if u < 25.0 => '\u{2581}', // Lower 1/8 block
            u if u < 37.5 => '\u{2582}', // Lower 2/8 block
            u if u < 50.0 => '\u{2583}', // Lower 3/8 block
            u if u < 62.5 => '\u{2584}', // Lower 4/8 block
            u if u < 75.0 => '\u{2585}', // Lower 5/8 block
            u if u < 87.5 => '\u{2586}', // Lower 6/8 block
            _ => '\u{2588}',             // Full block
        };

        let color = self.theme.usage_color(self.usage);
        let style = Style::default().fg(color);

        // Render block character (2 chars total: block + space)
        buf.set_string(area.x, area.y, char.to_string(), style);

        // Add separator if space allows
        if area.width >= 2 {
            buf.set_string(area.x + 1, area.y, " ", self.theme.dim_style());
        }
    }
}

/// Group header widget
pub struct GroupHeader<'a> {
    name: &'a str,
    usage: f32,
    core_count: usize,
    theme: &'a Theme,
}

impl<'a> GroupHeader<'a> {
    pub fn new(name: &'a str, usage: f32, core_count: usize, theme: &'a Theme) -> Self {
        Self {
            name,
            usage,
            core_count,
            theme,
        }
    }
}

impl Widget for GroupHeader<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        let header = format!(
            " {} ({} cores) - {:.1}% ",
            self.name, self.core_count, self.usage
        );

        let style = Style::default()
            .fg(self.theme.header_fg)
            .bg(self.theme.usage_color(self.usage));

        buf.set_string(area.x, area.y, &header, style);

        // Fill rest with background
        let remaining = area.width.saturating_sub(header.len() as u16);
        if remaining > 0 {
            let fill = " ".repeat(remaining as usize);
            buf.set_string(area.x + header.len() as u16, area.y, &fill, style);
        }
    }
}

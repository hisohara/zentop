use ratatui::style::{Color, Style};

/// Color scheme for CPU usage visualization
pub struct Theme {
    pub bar_low: Color,      // 0-25%
    pub bar_medium: Color,   // 25-50%
    pub bar_high: Color,     // 50-75%
    pub bar_critical: Color, // 75-100%
    pub text_normal: Color,
    pub text_dim: Color,
    pub text_highlight: Color,
    pub border: Color,
    pub header_bg: Color,
    pub header_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bar_low: Color::Green,
            bar_medium: Color::Cyan,
            bar_high: Color::Yellow,
            bar_critical: Color::Red,
            text_normal: Color::White,
            text_dim: Color::DarkGray,
            text_highlight: Color::Cyan,
            border: Color::DarkGray,
            header_bg: Color::Blue,
            header_fg: Color::White,
        }
    }
}

impl Theme {
    /// Get color for usage percentage
    pub fn usage_color(&self, usage: f32) -> Color {
        match usage {
            u if u < 25.0 => self.bar_low,
            u if u < 50.0 => self.bar_medium,
            u if u < 75.0 => self.bar_high,
            _ => self.bar_critical,
        }
    }

    /// Get style for usage bar
    pub fn bar_style(&self, usage: f32) -> Style {
        Style::default().fg(self.usage_color(usage))
    }

    /// Get normal text style
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text_normal)
    }

    /// Get dim text style
    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    /// Get header style
    pub fn header_style(&self) -> Style {
        Style::default().fg(self.header_fg).bg(self.header_bg)
    }
}

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::ui::theme::Theme;

/// Help overlay widget
pub struct HelpOverlay<'a> {
    theme: &'a Theme,
}

impl<'a> HelpOverlay<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }
}

impl Widget for HelpOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate centered popup area
        let popup_width = 50.min(area.width.saturating_sub(4));
        let popup_height = 16.min(area.height.saturating_sub(4));

        let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Clear background
        Clear.render(popup_area, buf);

        let help_text = r#"
  Keyboard Shortcuts
  ──────────────────

  View Modes:
    c    Core view (individual CPUs)
    d    CCD view (grouped by CCD)
    n    NPS view (grouped by NPS node)
    u    NUMA view (grouped by NUMA node)

  Display:
    s    Toggle SMT (all threads / physical only)

  Navigation:
    j/↓  Scroll down
    k/↑  Scroll up

  Other:
    h/?  Toggle this help
    q    Quit
"#;

        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.text_highlight));

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .style(self.theme.text_style())
            .alignment(Alignment::Left);

        paragraph.render(popup_area, buf);
    }
}

/// Calculate centered popup area
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    let popup_x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let popup_y = r.y + (r.height.saturating_sub(popup_height)) / 2;
    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
    Frame,
};

use crate::app::{App, ViewMode};
use crate::ui::theme::Theme;
use crate::ui::views::{render_ccd_view, render_core_view, render_nps_view};
use crate::ui::widgets::{Header, HelpOverlay};

/// Render the application UI
pub fn render(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    // Create layout: header (2 lines) + content
    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(area);

    // Render header
    let header = Header::new(
        &app.topology,
        app.view_mode,
        app.display_mode,
        app.show_smt,
        app.stats.total_usage,
        theme,
    );
    frame.render_widget(header, chunks[0]);

    // Render main content based on view mode
    let content_area = chunks[1];

    // Use a custom widget to render the view
    let view_widget = ViewWidget {
        app,
        theme,
    };
    frame.render_widget(view_widget, content_area);

    // Render help overlay if active
    if app.show_help {
        frame.render_widget(HelpOverlay::new(theme), area);
    }
}

/// Widget wrapper for rendering views
struct ViewWidget<'a> {
    app: &'a App,
    theme: &'a Theme,
}

impl Widget for ViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.app.topology.packages == 2 {
            // 2-socket system: split screen vertically
            let half_width = area.width / 2;
            let left_area = Rect::new(area.x, area.y, half_width, area.height);
            let right_area = Rect::new(area.x + half_width, area.y, area.width - half_width, area.height);

            // Render both sockets
            self.render_socket(left_area, buf, Some(0));
            self.render_socket(right_area, buf, Some(1));
        } else {
            // Single socket or other: use full area
            self.render_socket(area, buf, None);
        }
    }
}

impl ViewWidget<'_> {
    /// Render a socket's content with border frame
    fn render_socket(&self, area: Rect, buf: &mut Buffer, socket_filter: Option<usize>) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Create block with border
        let block = if let Some(socket_id) = socket_filter {
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Socket {} ", socket_id))
                .border_style(Style::default().fg(self.theme.border))
                .title_style(
                    Style::default()
                        .fg(self.theme.text_highlight)
                        .add_modifier(Modifier::BOLD),
                )
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border))
        };

        // Get inner area and render block
        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        match self.app.view_mode {
            ViewMode::Core => {
                render_core_view(
                    inner_area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.app.display_mode,
                    self.theme,
                    socket_filter,
                );
            }
            ViewMode::Ccd => {
                render_ccd_view(
                    inner_area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.app.display_mode,
                    self.theme,
                    socket_filter,
                );
            }
            ViewMode::Nps => {
                render_nps_view(
                    inner_area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.app.display_mode,
                    self.theme,
                    socket_filter,
                );
            }
        }
    }
}

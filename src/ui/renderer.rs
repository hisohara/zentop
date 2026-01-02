use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
    Frame,
};

use crate::app::{App, ViewMode};
use crate::ui::theme::Theme;
use crate::ui::views::{render_ccd_view, render_core_view, render_nps_view, render_numa_view};
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
        match self.app.view_mode {
            ViewMode::Core => {
                render_core_view(
                    area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.theme,
                );
            }
            ViewMode::Ccd => {
                render_ccd_view(
                    area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.theme,
                );
            }
            ViewMode::Nps => {
                render_nps_view(
                    area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.theme,
                );
            }
            ViewMode::Numa => {
                render_numa_view(
                    area,
                    buf,
                    &self.app.topology,
                    &self.app.stats,
                    self.app.show_smt,
                    self.app.scroll_offset,
                    self.theme,
                );
            }
        }
    }
}

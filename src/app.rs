use crate::stats::{StatsCollector, SystemStats};
use crate::topology::ZenTopology;

/// View mode for CPU display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Core,
    Ccd,
    Nps,
    Numa,
}

/// Application state
pub struct App {
    pub topology: ZenTopology,
    pub stats: SystemStats,
    pub view_mode: ViewMode,
    pub show_smt: bool,
    pub show_help: bool,
    pub should_quit: bool,
    pub scroll_offset: usize,
    collector: StatsCollector,
}

impl App {
    pub fn new(topology: ZenTopology) -> Self {
        let mut collector = StatsCollector::new();
        let stats = collector.refresh();

        Self {
            topology,
            stats,
            view_mode: ViewMode::Core,
            show_smt: true,
            show_help: false,
            should_quit: false,
            scroll_offset: 0,
            collector,
        }
    }

    /// Refresh CPU statistics
    pub fn refresh_stats(&mut self) {
        self.stats = self.collector.refresh();
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        if self.view_mode != mode {
            self.view_mode = mode;
            self.scroll_offset = 0; // Reset scroll when changing view
        }
    }

    /// Toggle SMT display
    pub fn toggle_smt(&mut self) {
        self.show_smt = !self.show_smt;
        self.scroll_offset = 0;
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Request quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

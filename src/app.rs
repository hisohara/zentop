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

/// Display density mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    #[default]
    Full,    // One core per line (original)
    Compact, // Multi-column, 16 cores per row
    Heatmap, // Ultra-dense block characters, 64 cores per row
}

/// Application state
pub struct App {
    pub topology: ZenTopology,
    pub stats: SystemStats,
    pub view_mode: ViewMode,
    pub display_mode: DisplayMode,
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
            display_mode: DisplayMode::default(),
            show_smt: true,
            show_help: false,
            should_quit: false,
            scroll_offset: 0,
            collector,
        }
    }

    /// Cycle display mode: Full -> Compact -> Heatmap -> Full
    pub fn cycle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Full => DisplayMode::Compact,
            DisplayMode::Compact => DisplayMode::Heatmap,
            DisplayMode::Heatmap => DisplayMode::Full,
        };
        self.scroll_offset = 0;
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

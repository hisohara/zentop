use ratatui::layout::Rect;

/// Grid layout calculator for multi-column CPU display
pub struct GridLayout {
    pub cols: usize,
    pub rows: usize,
    pub cell_width: u16,
}

impl GridLayout {
    /// Calculate layout for compact mode (16 cores per row, 12 chars each)
    pub fn compact(area: Rect, item_count: usize) -> Self {
        const CELL_WIDTH: u16 = 12;
        const COLS: usize = 16;

        let cols = COLS.min((area.width / CELL_WIDTH) as usize).max(1);
        let rows = (item_count + cols - 1) / cols;
        let cell_width = area.width / cols as u16;

        Self {
            cols,
            rows,
            cell_width,
        }
    }

    /// Calculate layout for heatmap mode (up to 80 cores per row, 2 chars each)
    /// Column count is limited to actual item count for smaller systems
    pub fn heatmap(area: Rect, item_count: usize) -> Self {
        const CELL_WIDTH: u16 = 2;
        const MAX_COLS: usize = 80;

        // Calculate max cols that fit on screen
        let screen_cols = (area.width / CELL_WIDTH) as usize;
        let screen_cols = (screen_cols / 16) * 16; // Round to multiple of 16
        let screen_cols = screen_cols.max(16);

        // Limit to actual item count, rounded up to multiple of 8
        let item_cols = ((item_count + 7) / 8) * 8; // Round up to multiple of 8
        let item_cols = item_cols.max(8);

        // Use minimum of screen capacity and item count
        let cols = screen_cols.min(MAX_COLS).min(item_cols);

        let rows = (item_count + cols - 1) / cols;

        Self {
            cols,
            rows,
            cell_width: CELL_WIDTH,
        }
    }

    /// Get cell rectangle for item at index
    pub fn cell_rect(&self, area: Rect, index: usize) -> Option<Rect> {
        let col = index % self.cols;
        let row = index / self.cols;

        let x = area.x + (col as u16 * self.cell_width);
        let y = area.y + row as u16;

        if y >= area.y + area.height {
            return None;
        }

        Some(Rect::new(x, y, self.cell_width, 1))
    }

    /// Check if all items fit in the area
    pub fn fits(&self, area: Rect) -> bool {
        self.rows <= area.height as usize
    }
}

//! Embedded TopBar / Status bar for Gastinel Desktop.

use gastinel_core::geom::{Point, Rect};

pub struct TopBar {
    pub bounds: Rect,
    pub height: u32,
    pub active_workspace: u32,
    pub workspace_count: u32,
    pub current_time_str: String,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
}

impl TopBar {
    pub const DEFAULT_HEIGHT: u32 = 32;

    pub fn new(screen_width: u32) -> Self {
        Self {
            bounds: Rect::new(0, 0, screen_width, Self::DEFAULT_HEIGHT),
            height: Self::DEFAULT_HEIGHT,
            active_workspace: 1,
            workspace_count: 5,
            current_time_str: String::from("00:00"),
            ram_used_mb: 0,
            ram_total_mb: 8192,
        }
    }

    pub fn resize(&mut self, screen_width: u32) {
        self.bounds.width = screen_width;
    }

    pub fn update_time(&mut self, time_str: &str) {
        self.current_time_str.clear();
        self.current_time_str.push_str(time_str);
    }

    pub fn update_memory(&mut self, used_mb: u64, total_mb: u64) {
        self.ram_used_mb = used_mb;
        self.ram_total_mb = total_mb;
    }

    pub fn set_active_workspace(&mut self, ws: u32) {
        self.active_workspace = ws;
    }

    /// Hit-test: checks if a point clicks on a workspace button (returns Some(workspace_id))
    pub fn hit_test_workspace(&self, p: Point) -> Option<u32> {
        if !self.bounds.contains(p) {
            return None;
        }

        // Workspace buttons on the left
        let start_x = 10;
        let btn_width = 30;
        let spacing = 6;

        for i in 1..=self.workspace_count {
            let x = start_x + ((i - 1) as i32) * (btn_width + spacing);
            let rect = Rect::new(x, 4, btn_width as u32, self.height - 8);
            if rect.contains(p) {
                return Some(i);
            }
        }

        None
    }
}

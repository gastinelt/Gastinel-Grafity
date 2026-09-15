//! Embedded TopBar / Status bar for Gastinel Desktop styled with Inter DA.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

pub struct TopBar {
    pub bounds: Rect,
    pub height: u32,
    pub active_workspace: u32,
    pub workspace_count: u32,
    pub current_time_str: String,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub zram_saved_mb: u64,
    pub ai_status_str: String,
}

impl TopBar {
    pub const DEFAULT_HEIGHT: u32 = 34;

    pub fn new(screen_width: u32) -> Self {
        Self {
            bounds: Rect::new(0, 0, screen_width, Self::DEFAULT_HEIGHT),
            height: Self::DEFAULT_HEIGHT,
            active_workspace: 1,
            workspace_count: 5,
            current_time_str: String::from("12:00"),
            ram_used_mb: 85,
            ram_total_mb: 8192,
            zram_saved_mb: 1420,
            ai_status_str: String::from("AI: Gemma 4 (8733MT/s)"),
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

    pub fn update_zram(&mut self, saved_mb: u64) {
        self.zram_saved_mb = saved_mb;
    }

    pub fn update_ai_status(&mut self, status: &str) {
        self.ai_status_str.clear();
        self.ai_status_str.push_str(status);
    }

    pub fn set_active_workspace(&mut self, ws: u32) {
        self.active_workspace = ws;
    }

    /// Generates draw primitives (quads & Inter text) into the frame render queue.
    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // 1. TopBar translucent background (acrylic dark)
        let bar_bg = QuadInstance::rounded_box(
            self.bounds,
            0.0,
            [0.08, 0.09, 0.11, 0.88], // Dark frosted acrylic
            [0.20, 0.22, 0.26, 0.60], // Subtle bottom border
            1.0,
            0.1,
        );
        queue.push(bar_bg);

        // 2. Workspace indicators (pills)
        let start_x = 12;
        let pill_w = 32;
        let pill_h = self.height - 10;
        let spacing = 6;

        for i in 1..=self.workspace_count {
            let x = start_x + ((i - 1) as i32) * (pill_w + spacing);
            let pill_rect = Rect::new(x, 5, pill_w as u32, pill_h);

            let is_active = i == self.active_workspace;
            let (fill, border, text_color) = if is_active {
                // Electric blue accent
                (
                    [0.22, 0.47, 0.94, 0.95],
                    [0.40, 0.65, 1.00, 0.80],
                    [1.0, 1.0, 1.0, 1.0],
                )
            } else {
                // Dim pill
                (
                    [0.14, 0.15, 0.18, 0.70],
                    [0.22, 0.24, 0.28, 0.50],
                    [0.65, 0.68, 0.75, 1.0],
                )
            };

            queue.push(QuadInstance::rounded_box(
                pill_rect,
                6.0,
                fill,
                border,
                1.0,
                0.2,
            ));

            // Inter text for workspace number
            let num_str = i.to_string();
            atlas.layout_text(
                &num_str,
                Point::new(x + 12, 22),
                12,
                if is_active { FontWeight::Bold } else { FontWeight::Medium },
                text_color,
                glyphs,
            );
        }

        // 3. Center Clock (Inter SemiBold 13px)
        let clock_x = (self.bounds.width as i32 / 2) - 20;
        atlas.layout_text(
            &self.current_time_str,
            Point::new(clock_x, 22),
            13,
            FontWeight::SemiBold,
            [0.95, 0.96, 0.98, 1.0],
            glyphs,
        );

        // 4. Right: AI 2B Engine pill + RAM & zram Monitor stats
        // AI Badge (e.g. "AI: 2B (8733MT/s)")
        let ai_pill_w = 126;
        let ai_pill_h = self.height - 10;
        let ai_x = self.bounds.width as i32 - 340;
        let ai_rect = Rect::new(ai_x, 5, ai_pill_w, ai_pill_h);

        queue.push(QuadInstance::rounded_box(
            ai_rect,
            6.0,
            [0.22, 0.16, 0.40, 0.85], // Deep indigo acrylic
            [0.45, 0.35, 0.80, 0.70], // Violet border
            1.0,
            0.2,
        ));

        atlas.layout_text(
            &self.ai_status_str,
            Point::new(ai_x + 10, 22),
            11,
            FontWeight::SemiBold,
            [0.85, 0.78, 1.0, 1.0], // Soft violet text
            glyphs,
        );

        // RAM & zram text
        let ram_text = format!("RAM: {}M / {}M (+{}M zram)", self.ram_used_mb, self.ram_total_mb, self.zram_saved_mb);
        let ram_x = self.bounds.width as i32 - 200;
        atlas.layout_text(
            &ram_text,
            Point::new(ram_x, 22),
            11,
            FontWeight::Regular,
            [0.55, 0.85, 0.65, 1.0], // Soft green for healthy ultralow RAM
            glyphs,
        );
    }
}

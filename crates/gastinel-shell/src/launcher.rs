//! Spotlight-style fast application search and command launcher with Inter DA.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect, Size};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub comment: String,
}

pub struct Launcher {
    pub visible: bool,
    pub bounds: Rect,
    pub query: String,
    pub entries: Vec<AppEntry>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
}

impl Launcher {
    pub const WIDTH: u32 = 640;
    pub const HEIGHT: u32 = 420;

    pub fn new(screen_size: Size) -> Self {
        let x = (screen_size.width.saturating_sub(Self::WIDTH) / 2) as i32;
        let y = (screen_size.height.saturating_sub(Self::HEIGHT) / 3) as i32;

        Self {
            visible: false,
            bounds: Rect::new(x, y, Self::WIDTH, Self::HEIGHT),
            query: String::with_capacity(64),
            entries: Vec::new(),
            filtered_indices: Vec::with_capacity(32),
            selected_index: 0,
        }
    }

    pub fn recalculate_position(&mut self, screen_size: Size) {
        self.bounds.x = (screen_size.width.saturating_sub(Self::WIDTH) / 2) as i32;
        self.bounds.y = (screen_size.height.saturating_sub(Self::HEIGHT) / 3) as i32;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.query.clear();
            self.refilter();
            self.selected_index = 0;
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.refilter();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
    }

    pub fn add_app(&mut self, name: &str, exec: &str, comment: &str) {
        self.entries.push(AppEntry {
            name: name.to_string(),
            exec: exec.to_string(),
            comment: comment.to_string(),
        });
        self.refilter();
    }

    pub fn insert_char(&mut self, c: char) {
        self.query.push(c);
        self.refilter();
        self.selected_index = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.refilter();
        self.selected_index = 0;
    }

    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_indices.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_indices.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&AppEntry> {
        let idx = *self.filtered_indices.get(self.selected_index)?;
        self.entries.get(idx)
    }

    fn refilter(&mut self) {
        self.filtered_indices.clear();
        if self.query.is_empty() {
            for i in 0..self.entries.len() {
                self.filtered_indices.push(i);
            }
            return;
        }

        let query_lower = self.query.to_lowercase();
        for (i, entry) in self.entries.iter().enumerate() {
            let name_lower = entry.name.to_lowercase();
            if fuzzy_match(&name_lower, &query_lower) {
                self.filtered_indices.push(i);
            }
        }
    }

    /// Generates draw primitives (quads & Inter text) into the frame render queue.
    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        if !self.visible {
            return;
        }

        // 1. Launcher Panel Backdrop (acrylic rounded box with soft drop shadow)
        let panel_bg = QuadInstance::rounded_box(
            self.bounds,
            14.0,
            [0.10, 0.11, 0.14, 0.94], // Deep acrylic dark
            [0.24, 0.26, 0.32, 0.70], // Crisp border
            1.5,
            0.8,
        )
        .with_shadow(
            [0.0, 0.0, 0.0, 0.50],
            24.0,
            [0.0, 8.0],
        );
        queue.push(panel_bg);

        // 2. Search Input Box
        let input_rect = Rect::new(
            self.bounds.x + 18,
            self.bounds.y + 18,
            self.bounds.width - 36,
            44,
        );
        queue.push(QuadInstance::rounded_box(
            input_rect,
            8.0,
            [0.15, 0.16, 0.20, 0.80],
            [0.28, 0.32, 0.40, 0.80],
            1.0,
            0.85,
        ));

        // Query text or placeholder
        if self.query.is_empty() {
            atlas.layout_text(
                "Type to search applications...",
                Point::new(input_rect.x + 16, input_rect.y + 28),
                15,
                FontWeight::Regular,
                [0.45, 0.48, 0.55, 1.0], // Muted placeholder
                glyphs,
            );
        } else {
            atlas.layout_text(
                &self.query,
                Point::new(input_rect.x + 16, input_rect.y + 28),
                15,
                FontWeight::SemiBold,
                [0.98, 0.98, 1.0, 1.0],
                glyphs,
            );
        }

        // 3. Results List
        let item_start_y = input_rect.bottom() + 16;
        let item_height = 46;

        for (slot, &entry_idx) in self.filtered_indices.iter().take(6).enumerate() {
            let item_y = item_start_y + (slot as i32) * (item_height + 4);
            let item_rect = Rect::new(
                self.bounds.x + 18,
                item_y,
                self.bounds.width - 36,
                item_height as u32,
            );

            let is_selected = slot == self.selected_index;
            if is_selected {
                // Electric blue highlight for selected item
                queue.push(QuadInstance::rounded_box(
                    item_rect,
                    8.0,
                    [0.22, 0.47, 0.94, 0.85],
                    [0.40, 0.65, 1.00, 0.90],
                    1.0,
                    0.86,
                ));
            } else {
                queue.push(QuadInstance::rounded_box(
                    item_rect,
                    8.0,
                    [0.13, 0.14, 0.17, 0.50],
                    [0.20, 0.22, 0.26, 0.30],
                    1.0,
                    0.85,
                ));
            }

            let entry = &self.entries[entry_idx];
            let name_color = if is_selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.90, 0.92, 0.95, 1.0]
            };
            let desc_color = if is_selected {
                [0.85, 0.90, 1.0, 0.9]
            } else {
                [0.55, 0.58, 0.65, 1.0]
            };

            // App Name (Inter SemiBold 14px)
            atlas.layout_text(
                &entry.name,
                Point::new(item_rect.x + 16, item_rect.y + 20),
                14,
                FontWeight::SemiBold,
                name_color,
                glyphs,
            );

            // App Description (Inter Regular 12px)
            atlas.layout_text(
                &entry.comment,
                Point::new(item_rect.x + 16, item_rect.y + 36),
                12,
                FontWeight::Regular,
                desc_color,
                glyphs,
            );
        }
    }
}

fn fuzzy_match(target: &str, query: &str) -> bool {
    let mut target_chars = target.chars();
    for qc in query.chars() {
        match target_chars.find(|&tc| tc == qc) {
            Some(_) => continue,
            None => return false,
        }
    }
    true
}

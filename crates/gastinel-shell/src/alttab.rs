//! Sélecteur de fenêtres Alt+Tab pour Gastinel Desktop.
//!
//! Affiche un aperçu modal centré des fenêtres ouvertes pour basculer
//! rapidement d'une application à l'autre au clavier ou à la souris.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Clone, Debug)]
pub struct WindowThumb {
    pub id: u32,
    pub title: String,
    pub app_icon: String,
}

pub struct AltTabSwitcher {
    pub is_open: bool,
    pub windows: Vec<WindowThumb>,
    pub selected_idx: usize,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl AltTabSwitcher {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            is_open: false,
            windows: Vec::with_capacity(8),
            selected_idx: 0,
            screen_width,
            screen_height,
        }
    }

    pub fn open(&mut self, open_windows: Vec<WindowThumb>) {
        if open_windows.is_empty() {
            return;
        }
        self.windows = open_windows;
        self.selected_idx = if self.windows.len() > 1 { 1 } else { 0 };
        self.is_open = true;
    }

    pub fn next(&mut self) {
        if !self.windows.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.windows.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.windows.is_empty() {
            self.selected_idx = (self.selected_idx + self.windows.len() - 1) % self.windows.len();
        }
    }

    pub fn close(&mut self) -> Option<u32> {
        self.is_open = false;
        self.windows.get(self.selected_idx).map(|w| w.id)
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        if !self.is_open || self.windows.is_empty() {
            return;
        }

        let item_w = 120u32;
        let item_h = 110u32;
        let spacing = 12i32;
        let total_w = (self.windows.len() as u32 * item_w) + ((self.windows.len() as u32 - 1) * spacing as u32) + 32;
        let total_h = item_h + 32;

        let modal_x = (self.screen_width as i32 - total_w as i32) / 2;
        let modal_y = (self.screen_height as i32 - total_h as i32) / 2;
        let modal_rect = Rect::new(modal_x, modal_y, total_w, total_h);

        // Fond modal acrylique sombre
        queue.push(
            QuadInstance::rounded_box(
                modal_rect,
                14.0,
                [0.10, 0.11, 0.16, 0.95],
                [0.25, 0.28, 0.42, 0.7],
                1.5,
                0.9,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.55], 32.0, [0.0, 10.0]),
        );

        // Cartes d'aperçu des fenêtres
        let mut cx = modal_x + 16;
        let cy = modal_y + 16;

        for (i, win) in self.windows.iter().enumerate() {
            let is_sel = i == self.selected_idx;
            let card_rect = Rect::new(cx, cy, item_w, item_h);

            let (bg_col, border_col, border_w) = if is_sel {
                ([0.18, 0.22, 0.35, 0.95], [0.45, 0.65, 1.0, 0.95], 2.0)
            } else {
                ([0.12, 0.13, 0.18, 0.8], [0.20, 0.22, 0.30, 0.4], 1.0)
            };

            queue.push(QuadInstance::rounded_box(
                card_rect,
                8.0,
                bg_col,
                border_col,
                border_w,
                0.91,
            ));

            // Icône
            atlas.layout_text(&win.app_icon, Point::new(cx + 46, cy + 44), 22, FontWeight::Bold, [0.95, 0.96, 1.0, 1.0], glyphs);

            // Titre de la fenêtre
            let short_title: String = win.title.chars().take(12).collect();
            atlas.layout_text(
                &short_title,
                Point::new(cx + 10, cy + 85),
                11,
                if is_sel { FontWeight::SemiBold } else { FontWeight::Regular },
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.70, 0.74, 0.85, 1.0] },
                glyphs,
            );

            cx += item_w as i32 + spacing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alt_tab_cycle() {
        let mut switcher = AltTabSwitcher::new(1920, 1080);
        let list = vec![
            WindowThumb { id: 1, title: String::from("Helix"), app_icon: String::from("📝") },
            WindowThumb { id: 2, title: String::from("Terminal"), app_icon: String::from("🖥️") },
            WindowThumb { id: 3, title: String::from("Chat Gemma"), app_icon: String::from("✨") },
        ];

        switcher.open(list);
        assert_eq!(switcher.selected_idx, 1); // Démarre sur la 2ème fenêtre

        switcher.next();
        assert_eq!(switcher.selected_idx, 2);

        switcher.next();
        assert_eq!(switcher.selected_idx, 0);

        switcher.prev();
        assert_eq!(switcher.selected_idx, 2);

        let chosen = switcher.close();
        assert_eq!(chosen, Some(3));
        assert!(!switcher.is_open);
    }
}

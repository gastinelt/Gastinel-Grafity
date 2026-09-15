//! Menu contextuel clic droit pour Gastinel Desktop.
//!
//! Affiche un menu surgissant élégant aux coordonnées exactes de la souris,
//! adapté au contexte (Bureau, Fenêtre, Fichier) avec raccourcis et séparateurs.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Clone, Debug)]
pub enum ContextMenuItem {
    Action {
        id: u32,
        icon: String,
        label: String,
        shortcut: Option<String>,
        enabled: bool,
    },
    Separator,
}

pub struct ContextMenu {
    pub is_visible: bool,
    pub bounds: Rect,
    pub items: Vec<ContextMenuItem>,
    pub hovered_idx: Option<usize>,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl ContextMenu {
    pub const MENU_WIDTH: u32 = 210;
    pub const ITEM_HEIGHT: u32 = 28;

    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            is_visible: false,
            bounds: Rect::ZERO,
            items: Vec::with_capacity(8),
            hovered_idx: None,
            screen_width,
            screen_height,
        }
    }

    pub fn open_at(&mut self, mut x: i32, mut y: i32, items: Vec<ContextMenuItem>) {
        let total_h = (items.len() as u32 * Self::ITEM_HEIGHT) + 12;

        // Contraindre dans l'écran
        if x + (Self::MENU_WIDTH as i32) > self.screen_width as i32 {
            x = (self.screen_width as i32) - (Self::MENU_WIDTH as i32) - 8;
        }
        if y + (total_h as i32) > self.screen_height as i32 {
            y = (self.screen_height as i32) - (total_h as i32) - 8;
        }

        self.bounds = Rect::new(x, y, Self::MENU_WIDTH, total_h);
        self.items = items;
        self.hovered_idx = None;
        self.is_visible = true;
    }

    pub fn open_desktop_menu(&mut self, x: i32, y: i32) {
        let items = vec![
            ContextMenuItem::Action { id: 1, icon: String::from("📁"), label: String::from("Nouveau dossier"), shortcut: Some(String::from("Ctrl+Shift+N")), enabled: true },
            ContextMenuItem::Action { id: 2, icon: String::from("📝"), label: String::from("Nouveau document Write"), shortcut: None, enabled: true },
            ContextMenuItem::Separator,
            ContextMenuItem::Action { id: 3, icon: String::from("🖥️"), label: String::from("Ouvrir dans le Terminal"), shortcut: None, enabled: true },
            ContextMenuItem::Action { id: 4, icon: String::from("🎨"), label: String::from("Changer de thème"), shortcut: None, enabled: true },
            ContextMenuItem::Separator,
            ContextMenuItem::Action { id: 5, icon: String::from("⚙️"), label: String::from("Paramètres d'affichage"), shortcut: None, enabled: true },
        ];
        self.open_at(x, y, items);
    }

    pub fn close(&mut self) {
        self.is_visible = false;
        self.items.clear();
        self.hovered_idx = None;
    }

    pub fn click(&mut self, pos: Point) -> Option<u32> {
        if !self.is_visible || !self.bounds.contains(pos) {
            self.close();
            return None;
        }

        let rel_y = (pos.y - self.bounds.y - 6).max(0) as u32;
        let idx = (rel_y / Self::ITEM_HEIGHT) as usize;

        if let Some(item) = self.items.get(idx) {
            if let ContextMenuItem::Action { id, enabled: true, .. } = item {
                let chosen_id = *id;
                self.close();
                return Some(chosen_id);
            }
        }

        self.close();
        None
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        if !self.is_visible || self.items.is_empty() {
            return;
        }

        // Fond acrylique du menu contextuel
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                8.0,
                [0.10, 0.11, 0.16, 0.98],
                [0.26, 0.28, 0.42, 0.8],
                1.0,
                0.92,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 20.0, [0.0, 6.0]),
        );

        let mut item_y = self.bounds.y + 6;

        for (idx, item) in self.items.iter().enumerate() {
            match item {
                ContextMenuItem::Action { icon, label, shortcut, enabled, .. } => {
                    let item_rect = Rect::new(self.bounds.x + 4, item_y, self.bounds.width - 8, Self::ITEM_HEIGHT);
                    let is_hover = self.hovered_idx == Some(idx);

                    if is_hover && *enabled {
                        queue.push(QuadInstance::rounded_box(
                            item_rect,
                            4.0,
                            [0.25, 0.35, 0.85, 0.9],
                            [0.4, 0.55, 1.0, 0.6],
                            1.0,
                            0.93,
                        ));
                    }

                    let (text_col, sc_col) = if *enabled {
                        ([0.95, 0.96, 1.0, 1.0], [0.60, 0.65, 0.78, 1.0])
                    } else {
                        ([0.45, 0.48, 0.55, 1.0], [0.35, 0.38, 0.45, 1.0])
                    };

                    let line_text = format!("{} {}", icon, label);
                    atlas.layout_text(
                        &line_text,
                        Point::new(item_rect.x + 8, item_rect.y + 19),
                        11,
                        FontWeight::Regular,
                        text_col,
                        glyphs,
                    );

                    if let Some(sc) = shortcut {
                        let sc_x = item_rect.right() - (sc.len() as i32 * 6) - 10;
                        atlas.layout_text(
                            sc,
                            Point::new(sc_x, item_rect.y + 19),
                            10,
                            FontWeight::Regular,
                            sc_col,
                            glyphs,
                        );
                    }

                    item_y += Self::ITEM_HEIGHT as i32;
                }
                ContextMenuItem::Separator => {
                    let sep_rect = Rect::new(self.bounds.x + 10, item_y + 3, self.bounds.width - 20, 1);
                    queue.push(QuadInstance::solid(sep_rect, [0.22, 0.24, 0.32, 0.6], 0.93));
                    item_y += 8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_open_and_click() {
        let mut menu = ContextMenu::new(1920, 1080);
        menu.open_desktop_menu(100, 100);
        assert!(menu.is_visible);
        assert!(menu.bounds.width > 0);

        let clicked = menu.click(Point::new(120, 115));
        assert_eq!(clicked, Some(1)); // Premier élément
        assert!(!menu.is_visible);
    }

    #[test]
    fn test_context_menu_edge_clamping() {
        let mut menu = ContextMenu::new(1920, 1080);
        menu.open_desktop_menu(1900, 1050);
        assert!(menu.bounds.right() <= 1920);
        assert!(menu.bounds.bottom() <= 1080);
    }
}

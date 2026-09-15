//! Gestionnaire de paquets graphique pour Alpine Linux (APK) dans Gastinel Desktop.
//!
//! Permet de parcourir, rechercher, installer et mettre à jour les paquets
//! avec affichage des tailles, versions, dépendances et catégories.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PackageCategory {
    Tous,
    Developpement,
    Systeme,
    Bureautique,
}

#[derive(Clone, Debug)]
pub struct PackageItem {
    pub name: String,
    pub version: String,
    pub description: String,
    pub size_mb: f32,
    pub category: PackageCategory,
    pub is_installed: bool,
    pub has_update: bool,
}

pub struct PackageManagerApp {
    pub bounds: Rect,
    pub packages: Vec<PackageItem>,
    pub active_category: PackageCategory,
    pub search_query: String,
    pub selected_package: Option<usize>,
    pub pending_updates_count: usize,
}

impl PackageManagerApp {
    pub fn new(bounds: Rect) -> Self {
        let mut packages = Vec::with_capacity(16);

        packages.push(PackageItem {
            name: String::from("vulkan-loader"),
            version: String::from("1.3.283-r0"),
            description: String::from("Chargeur Vulkan officiel Khronos pour Alpine"),
            size_mb: 1.2,
            category: PackageCategory::Systeme,
            is_installed: true,
            has_update: false,
        });

        packages.push(PackageItem {
            name: String::from("seatd"),
            version: String::from("0.8.0-r1"),
            description: String::from("Gestionnaire de session utilisateur sans systemd"),
            size_mb: 0.4,
            category: PackageCategory::Systeme,
            is_installed: true,
            has_update: false,
        });

        packages.push(PackageItem {
            name: String::from("mold"),
            version: String::from("2.31.0-r0"),
            description: String::from("Linker ultra-rapide optimisé pour le développement Rust"),
            size_mb: 18.5,
            category: PackageCategory::Developpement,
            is_installed: true,
            has_update: true,
        });

        packages.push(PackageItem {
            name: String::from("helix"),
            version: String::from("24.03-r0"),
            description: String::from("Éditeur modal post-moderne écrit en Rust"),
            size_mb: 22.0,
            category: PackageCategory::Developpement,
            is_installed: true,
            has_update: false,
        });

        packages.push(PackageItem {
            name: String::from("foot"),
            version: String::from("1.17.2-r0"),
            description: String::from("Émulateur de terminal Wayland rapide et léger"),
            size_mb: 1.8,
            category: PackageCategory::Systeme,
            is_installed: true,
            has_update: false,
        });

        packages.push(PackageItem {
            name: String::from("firefox-esr"),
            version: String::from("128.1.0-r0"),
            description: String::from("Navigateur Web Mozilla Firefox avec profil basse mémoire"),
            size_mb: 85.0,
            category: PackageCategory::Bureautique,
            is_installed: false,
            has_update: false,
        });

        packages.push(PackageItem {
            name: String::from("alacritty"),
            version: String::from("0.13.2-r1"),
            description: String::from("Terminal accéléré par le GPU"),
            size_mb: 12.4,
            category: PackageCategory::Developpement,
            is_installed: false,
            has_update: false,
        });

        packages.push(PackageItem {
            name: String::from("zellij"),
            version: String::from("0.40.0-r0"),
            description: String::from("Multiplexeur de terminal moderne en Rust"),
            size_mb: 14.2,
            category: PackageCategory::Developpement,
            is_installed: false,
            has_update: false,
        });

        let pending_updates = packages.iter().filter(|p| p.has_update).count();

        Self {
            bounds,
            packages,
            active_category: PackageCategory::Tous,
            search_query: String::new(),
            selected_package: Some(0),
            pending_updates_count: pending_updates,
        }
    }

    pub fn set_category(&mut self, cat: PackageCategory) {
        self.active_category = cat;
        self.selected_package = None;
    }

    pub fn toggle_install(&mut self, idx: usize) {
        if let Some(pkg) = self.packages.get_mut(idx) {
            pkg.is_installed = !pkg.is_installed;
            if !pkg.is_installed {
                pkg.has_update = false;
            }
        }
    }

    pub fn filtered_packages(&self) -> Vec<(usize, &PackageItem)> {
        self.packages
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let cat_match = match self.active_category {
                    PackageCategory::Tous => true,
                    c => p.category == c,
                };
                let search_match = if self.search_query.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&self.search_query.to_lowercase())
                        || p.description.to_lowercase().contains(&self.search_query.to_lowercase())
                };
                cat_match && search_match
            })
            .collect()
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // Fond principal
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                14.0,
                [0.10, 0.11, 0.15, 0.98],
                [0.24, 0.28, 0.40, 0.7],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 24.0, [0.0, 6.0]),
        );

        // Barre supérieure avec titre & recherche
        let header_rect = Rect::new(self.bounds.x + 14, self.bounds.y + 14, self.bounds.width - 28, 44);
        queue.push(QuadInstance::rounded_box(
            header_rect,
            8.0,
            [0.14, 0.15, 0.20, 0.9],
            [0.22, 0.25, 0.35, 0.5],
            1.0,
            0.21,
        ));

        atlas.layout_text(
            "📦 Logiciels & Paquets APK",
            Point::new(header_rect.x + 12, header_rect.y + 28),
            14,
            FontWeight::Bold,
            [0.95, 0.96, 1.0, 1.0],
            glyphs,
        );

        // Bannière de mise à jour si paquets disponibles
        if self.pending_updates_count > 0 {
            let badge = Rect::new(header_rect.right() - 170, header_rect.y + 8, 155, 28);
            queue.push(QuadInstance::rounded_box(badge, 6.0, [0.35, 0.25, 0.65, 0.9], [0.55, 0.40, 0.95, 0.8], 1.0, 0.22));
            let badge_str = format!("✨ {} mise à jour", self.pending_updates_count);
            atlas.layout_text(&badge_str, Point::new(badge.x + 12, badge.y + 19), 11, FontWeight::SemiBold, [1.0, 1.0, 1.0, 1.0], glyphs);
        }

        // Catégories (Tous, Dév, Système, Bureautique)
        let cat_y = header_rect.bottom() + 10;
        let categories = [
            (PackageCategory::Tous, "Tous"),
            (PackageCategory::Developpement, "Développement"),
            (PackageCategory::Systeme, "Système"),
            (PackageCategory::Bureautique, "Bureautique"),
        ];

        let mut cx = self.bounds.x + 14;
        for (c, label) in categories.iter() {
            let is_sel = *c == self.active_category;
            let cat_w = 95u32;
            let c_rect = Rect::new(cx, cat_y, cat_w, 28);
            let (bg, border) = if is_sel {
                ([0.28, 0.35, 0.85, 0.9], [0.45, 0.55, 1.0, 0.8])
            } else {
                ([0.13, 0.14, 0.19, 0.7], [0.20, 0.22, 0.30, 0.4])
            };

            queue.push(QuadInstance::rounded_box(c_rect, 6.0, bg, border, 1.0, 0.21));
            atlas.layout_text(
                label,
                Point::new(c_rect.x + 12, c_rect.y + 19),
                11,
                if is_sel { FontWeight::SemiBold } else { FontWeight::Regular },
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.70, 0.75, 0.85, 1.0] },
                glyphs,
            );
            cx += cat_w as i32 + 8;
        }

        // Liste des paquets filtrés
        let list_y = cat_y + 36;
        let mut py = list_y;
        let list_w = self.bounds.width - 28;

        for (idx, pkg) in self.filtered_packages().iter().take(6) {
            let card_rect = Rect::new(self.bounds.x + 14, py, list_w, 48);
            let is_sel = self.selected_package == Some(*idx);

            let border_col = if is_sel {
                [0.40, 0.55, 0.95, 0.8]
            } else {
                [0.18, 0.20, 0.28, 0.4]
            };

            queue.push(QuadInstance::rounded_box(
                card_rect,
                8.0,
                [0.12, 0.13, 0.18, 0.9],
                border_col,
                1.0,
                0.21,
            ));

            let title_line = format!("{}  v{}", pkg.name, pkg.version);
            atlas.layout_text(&title_line, Point::new(card_rect.x + 12, card_rect.y + 20), 12, FontWeight::SemiBold, [0.95, 0.96, 1.0, 1.0], glyphs);
            atlas.layout_text(&pkg.description, Point::new(card_rect.x + 12, card_rect.y + 38), 11, FontWeight::Regular, [0.60, 0.65, 0.75, 1.0], glyphs);

            // Bouton installer / installé
            let btn_rect = Rect::new(card_rect.right() - 100, card_rect.y + 11, 88, 26);
            let (btn_bg, btn_txt, btn_col) = if pkg.is_installed {
                ([0.18, 0.35, 0.25, 0.8], "Installé", [0.45, 0.85, 0.55, 1.0])
            } else {
                ([0.25, 0.40, 0.85, 0.85], "Installer", [1.0, 1.0, 1.0, 1.0])
            };

            queue.push(QuadInstance::rounded_box(btn_rect, 5.0, btn_bg, [1.0, 1.0, 1.0, 0.15], 1.0, 0.22));
            atlas.layout_text(btn_txt, Point::new(btn_rect.x + 18, btn_rect.y + 18), 11, FontWeight::SemiBold, btn_col, glyphs);

            py += 54;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_filtering() {
        let mut app = PackageManagerApp::new(Rect::new(0, 0, 650, 480));
        assert!(app.filtered_packages().len() >= 5);

        app.set_category(PackageCategory::Bureautique);
        assert_eq!(app.filtered_packages().len(), 1);
        assert_eq!(app.filtered_packages()[0].1.name, "firefox-esr");
    }

    #[test]
    fn test_package_toggle_install() {
        let mut app = PackageManagerApp::new(Rect::new(0, 0, 650, 480));
        let prev = app.packages[0].is_installed;
        app.toggle_install(0);
        assert_ne!(app.packages[0].is_installed, prev);
    }
}

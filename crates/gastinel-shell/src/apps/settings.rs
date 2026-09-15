//! Panneau de Paramètres Système complet pour Gastinel Desktop.
//!
//! Configure l'affichage, le réseau, le son, l'IA locale Gemma 4, la batterie,
//! les comptes, le stockage NVMe/zram et les informations matérielles.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SettingsCategory {
    Systeme,
    Affichage,
    Son,
    Reseau,
    IA,
    Batterie,
    Stockage,
    APropos,
}

#[derive(Clone, Debug)]
pub struct SettingItem {
    pub key: String,
    pub title: String,
    pub description: String,
    pub value_str: String,
    pub is_toggle: bool,
    pub toggle_state: bool,
}

pub struct SettingsApp {
    pub bounds: Rect,
    pub active_category: SettingsCategory,
    pub categories: Vec<(SettingsCategory, &'static str, &'static str)>,
    pub items: Vec<SettingItem>,
}

impl SettingsApp {
    pub fn new(bounds: Rect) -> Self {
        let categories = vec![
            (SettingsCategory::Systeme, "⚙️", "Système"),
            (SettingsCategory::Affichage, "🖥️", "Affichage"),
            (SettingsCategory::Son, "🔊", "Son & Audio"),
            (SettingsCategory::Reseau, "📶", "Réseau & Wi-Fi"),
            (SettingsCategory::IA, "🧠", "IA Locale (Gemma 4)"),
            (SettingsCategory::Batterie, "🔋", "Batterie & Énergie"),
            (SettingsCategory::Stockage, "💾", "Stockage & zram"),
            (SettingsCategory::APropos, "ℹ️", "À Propos"),
        ];

        let mut app = Self {
            bounds,
            active_category: SettingsCategory::Systeme,
            categories,
            items: Vec::with_capacity(16),
        };

        app.load_category_items(SettingsCategory::Systeme);
        app
    }

    pub fn select_category(&mut self, cat: SettingsCategory) {
        self.active_category = cat;
        self.load_category_items(cat);
    }

    pub fn toggle_item(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            if item.is_toggle {
                item.toggle_state = !item.toggle_state;
            }
        }
    }

    fn load_category_items(&mut self, cat: SettingsCategory) {
        self.items.clear();
        match cat {
            SettingsCategory::Systeme => {
                self.items.push(SettingItem {
                    key: String::from("hostname"),
                    title: String::from("Nom de la machine"),
                    description: String::from("Identifiant réseau du système"),
                    value_str: String::from("gastinel-book5"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("os_version"),
                    title: String::from("Version Gastinel"),
                    description: String::from("Alpine Linux Low-Level Musl + Vulkan Direct"),
                    value_str: String::from("0.1.0-edge (Rolling)"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("auto_update"),
                    title: String::from("Mises à jour automatiques APK"),
                    description: String::from("Télécharge et vérifie les signatures des paquets"),
                    value_str: String::new(),
                    is_toggle: true,
                    toggle_state: true,
                });
            }
            SettingsCategory::Affichage => {
                self.items.push(SettingItem {
                    key: String::from("resolution"),
                    title: String::from("Résolution de l'écran"),
                    description: String::from("Dalle AMOLED Samsung Galaxy Book5"),
                    value_str: String::from("1920 × 1080 @ 60Hz"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("night_light"),
                    title: String::from("Éclairage nocturne"),
                    description: String::from("Réduit la lumière bleue pour le confort visuel"),
                    value_str: String::new(),
                    is_toggle: true,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("scale"),
                    title: String::from("Mise à l'échelle de l'interface"),
                    description: String::from("Rendu vectoriel Inter Typography DA"),
                    value_str: String::from("100% (Pixel-perfect)"),
                    is_toggle: false,
                    toggle_state: false,
                });
            }
            SettingsCategory::Son => {
                self.items.push(SettingItem {
                    key: String::from("volume"),
                    title: String::from("Volume principal"),
                    description: String::from("Haut-parleurs Dolby Atmos"),
                    value_str: String::from("72%"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("system_sounds"),
                    title: String::from("Sons d'interface"),
                    description: String::from("Retours sonores pour les actions du shell"),
                    value_str: String::new(),
                    is_toggle: true,
                    toggle_state: true,
                });
            }
            SettingsCategory::Reseau => {
                self.items.push(SettingItem {
                    key: String::from("wifi"),
                    title: String::from("Wi-Fi Intel Wi-Fi 6E"),
                    description: String::from("Connecté à 'Fibre_Theo_5G' (Signal excellent)"),
                    value_str: String::from("192.168.1.42"),
                    is_toggle: true,
                    toggle_state: true,
                });
                self.items.push(SettingItem {
                    key: String::from("airplane"),
                    title: String::from("Mode avion"),
                    description: String::from("Désactive toutes les communications sans fil"),
                    value_str: String::new(),
                    is_toggle: true,
                    toggle_state: false,
                });
            }
            SettingsCategory::IA => {
                self.items.push(SettingItem {
                    key: String::from("ai_model"),
                    title: String::from("Modèle Local"),
                    description: String::from("Gemma 4 E2B Uncensored (HauhauCS Aggressive Q2_K_P)"),
                    value_str: String::from("3.00 Go GGUF"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("ai_vram"),
                    title: String::from("Allocation VRAM UMA"),
                    description: String::from("Mémoire GPU partagée Intel Graphics"),
                    value_str: String::from("2980 Mo alloués"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("ai_bandwidth"),
                    title: String::from("Débit Mémoire"),
                    description: String::from("LPDDR5x @ 8733 MT/s (~74 tok/s en inférence)"),
                    value_str: String::from("69.8 Go/s"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("ai_daemon"),
                    title: String::from("Démon IA en arrière-plan"),
                    description: String::from("gastinel-ai-daemon préchargé au démarrage"),
                    value_str: String::new(),
                    is_toggle: true,
                    toggle_state: true,
                });
            }
            SettingsCategory::Batterie => {
                self.items.push(SettingItem {
                    key: String::from("battery_level"),
                    title: String::from("Niveau de charge"),
                    description: String::from("Batterie Samsung Li-ion 54Wh"),
                    value_str: String::from("84% (En charge rapide)"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("power_saver"),
                    title: String::from("Mode économie d'énergie"),
                    description: String::from("Limite la fréquence des P-cores et l'inactivité GPU"),
                    value_str: String::new(),
                    is_toggle: true,
                    toggle_state: false,
                });
            }
            SettingsCategory::Stockage => {
                self.items.push(SettingItem {
                    key: String::from("nvme"),
                    title: String::from("SSD NVMe PCIe Gen4"),
                    description: String::from("Partition racine Alpine Linux / ext4"),
                    value_str: String::from("24 Go utilisés / 128 Go libres"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("zram"),
                    title: String::from("Swap compressé zram"),
                    description: String::from("Algorithme lz4 avec swappiness=180"),
                    value_str: String::from("6.0 Go réservés"),
                    is_toggle: false,
                    toggle_state: false,
                });
            }
            SettingsCategory::APropos => {
                self.items.push(SettingItem {
                    key: String::from("cpu"),
                    title: String::from("Processeur"),
                    description: String::from("Intel Core Ultra 5 225U (10 cœurs / 12 threads)"),
                    value_str: String::from("Jusqu'à 4.8 GHz"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("ram"),
                    title: String::from("Mémoire Vive"),
                    description: String::from("8 Go LPDDR5x bicanal ultra-haute vitesse"),
                    value_str: String::from("8733 MT/s"),
                    is_toggle: false,
                    toggle_state: false,
                });
                self.items.push(SettingItem {
                    key: String::from("graphics"),
                    title: String::from("Processeur Graphique"),
                    description: String::from("Intel Graphics Xe-LPG (Vulkan 1.4)"),
                    value_str: String::from("Direct UMA"),
                    is_toggle: false,
                    toggle_state: false,
                });
            }
        }
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // Fond de fenêtre principal
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                14.0,
                [0.09, 0.10, 0.13, 0.98],
                [0.22, 0.25, 0.35, 0.7],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 24.0, [0.0, 6.0]),
        );

        // Colonne de navigation gauche (Sidebar 190px)
        let sidebar_w = 190u32;
        let sidebar_rect = Rect::new(self.bounds.x + 10, self.bounds.y + 10, sidebar_w, self.bounds.height - 20);
        queue.push(QuadInstance::rounded_box(
            sidebar_rect,
            10.0,
            [0.12, 0.13, 0.18, 0.85],
            [0.18, 0.20, 0.28, 0.5],
            1.0,
            0.21,
        ));

        atlas.layout_text(
            "Paramètres",
            Point::new(sidebar_rect.x + 14, sidebar_rect.y + 26),
            15,
            FontWeight::Bold,
            [0.95, 0.96, 1.0, 1.0],
            glyphs,
        );

        let mut nav_y = sidebar_rect.y + 44;
        for (cat, icon, label) in self.categories.iter() {
            let is_sel = *cat == self.active_category;
            let nav_btn = Rect::new(sidebar_rect.x + 8, nav_y, sidebar_rect.width - 16, 34);

            if is_sel {
                queue.push(QuadInstance::rounded_box(
                    nav_btn,
                    6.0,
                    [0.28, 0.35, 0.85, 0.9],
                    [0.45, 0.55, 1.0, 0.8],
                    1.0,
                    0.22,
                ));
            }

            let text_col = if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.75, 0.78, 0.88, 1.0] };
            let nav_text = format!("{}  {}", icon, label);
            atlas.layout_text(
                &nav_text,
                Point::new(nav_btn.x + 10, nav_btn.y + 22),
                11,
                if is_sel { FontWeight::SemiBold } else { FontWeight::Regular },
                text_col,
                glyphs,
            );

            nav_y += 38;
        }

        // Zone de contenu droite
        let content_x = sidebar_rect.right() + 14;
        let content_w = (self.bounds.right() - 14).saturating_sub(content_x) as u32;
        let mut item_y = self.bounds.y + 20;

        for item in self.items.iter() {
            let card_rect = Rect::new(content_x, item_y, content_w, 64);
            queue.push(QuadInstance::rounded_box(
                card_rect,
                8.0,
                [0.12, 0.13, 0.17, 0.9],
                [0.20, 0.22, 0.30, 0.5],
                1.0,
                0.21,
            ));

            atlas.layout_text(
                &item.title,
                Point::new(card_rect.x + 14, card_rect.y + 24),
                13,
                FontWeight::SemiBold,
                [0.95, 0.96, 1.0, 1.0],
                glyphs,
            );

            atlas.layout_text(
                &item.description,
                Point::new(card_rect.x + 14, card_rect.y + 46),
                11,
                FontWeight::Regular,
                [0.55, 0.60, 0.72, 1.0],
                glyphs,
            );

            if item.is_toggle {
                let switch_rect = Rect::new(card_rect.right() - 56, card_rect.y + 20, 42, 24);
                let (sw_bg, sw_border) = if item.toggle_state {
                    ([0.25, 0.70, 0.45, 0.95], [0.4, 0.85, 0.6, 0.9])
                } else {
                    ([0.22, 0.24, 0.30, 0.8], [0.32, 0.35, 0.45, 0.6])
                };
                queue.push(QuadInstance::rounded_box(switch_rect, 12.0, sw_bg, sw_border, 1.0, 0.22));

                let knob_x = if item.toggle_state { switch_rect.right() - 20 } else { switch_rect.x + 3 };
                let knob = Rect::new(knob_x, switch_rect.y + 3, 18, 18);
                queue.push(QuadInstance::rounded_box(knob, 9.0, [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0], 0.0, 0.23));
            } else if !item.value_str.is_empty() {
                let val_x = card_rect.right() - 170;
                atlas.layout_text(
                    &item.value_str,
                    Point::new(val_x, card_rect.y + 35),
                    12,
                    FontWeight::Medium,
                    [0.45, 0.80, 0.60, 1.0], // vert émeraude
                    glyphs,
                );
            }

            item_y += 72;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_categories_and_items() {
        let mut app = SettingsApp::new(Rect::new(0, 0, 680, 480));
        assert_eq!(app.categories.len(), 8);
        assert_eq!(app.active_category, SettingsCategory::Systeme);
        assert!(!app.items.is_empty());

        app.select_category(SettingsCategory::IA);
        assert_eq!(app.active_category, SettingsCategory::IA);
        assert!(app.items.iter().any(|it| it.key == "ai_model"));
    }

    #[test]
    fn test_settings_toggle() {
        let mut app = SettingsApp::new(Rect::new(0, 0, 680, 480));
        let toggle_idx = app.items.iter().position(|it| it.is_toggle).unwrap();
        let prev = app.items[toggle_idx].toggle_state;
        app.toggle_item(toggle_idx);
        assert_ne!(app.items[toggle_idx].toggle_state, prev);
    }
}

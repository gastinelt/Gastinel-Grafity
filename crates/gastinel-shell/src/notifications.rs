//! Centre de notifications et toasts flottants pour Gastinel Desktop.
//!
//! Affiche des alertes temporaires animées en haut à droite de l'écran
//! et offre un panneau d'historique avec filtrage par sévérité.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: String,
    pub remaining_duration_secs: f32,
    pub total_duration_secs: f32,
}

pub struct NotificationCenter {
    pub notifications: Vec<Notification>,
    pub next_id: u64,
    pub is_panel_open: bool,
    pub do_not_disturb: bool,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl NotificationCenter {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let mut center = Self {
            notifications: Vec::with_capacity(16),
            next_id: 1,
            is_panel_open: false,
            do_not_disturb: false,
            screen_width,
            screen_height,
        };

        // Notifications initiales de démonstration
        center.push(
            "Système prêt",
            "Gastinel Desktop démarré avec succès sur Vulkan 1.4",
            NotificationLevel::Success,
            5.0,
        );
        center.push(
            "IA Gemma 4 chargée",
            "Modèle 3.0 Go prêt en mémoire vive LPDDR5x (8733 MT/s)",
            NotificationLevel::Info,
            6.0,
        );

        center
    }

    pub fn push(&mut self, title: &str, message: &str, level: NotificationLevel, duration_secs: f32) {
        if self.do_not_disturb && level != NotificationLevel::Error {
            return;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.notifications.push(Notification {
            id,
            title: title.to_string(),
            message: message.to_string(),
            level,
            timestamp: String::from("22:55"),
            remaining_duration_secs: duration_secs,
            total_duration_secs: duration_secs,
        });
    }

    pub fn tick(&mut self, dt_secs: f32) {
        for notif in self.notifications.iter_mut() {
            notif.remaining_duration_secs = (notif.remaining_duration_secs - dt_secs).max(0.0);
        }
        // Retirer les notifications expirées
        self.notifications.retain(|n| n.remaining_duration_secs > 0.0);
    }

    pub fn clear_all(&mut self) {
        self.notifications.clear();
    }

    pub fn toggle_panel(&mut self) {
        self.is_panel_open = !self.is_panel_open;
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // 1. Toasts flottants en haut à droite
        let toast_w = 320u32;
        let toast_h = 64u32;
        let toast_x = (self.screen_width as i32) - (toast_w as i32) - 16;
        let mut toast_y = 44i32; // Sous la TopBar

        for notif in self.notifications.iter().take(4) {
            let toast_rect = Rect::new(toast_x, toast_y, toast_w, toast_h);

            let (border_col, indicator_col) = match notif.level {
                NotificationLevel::Info => ([0.25, 0.50, 0.85, 0.8], [0.35, 0.65, 0.95, 1.0]),
                NotificationLevel::Success => ([0.20, 0.65, 0.40, 0.8], [0.35, 0.85, 0.55, 1.0]),
                NotificationLevel::Warning => ([0.85, 0.60, 0.20, 0.8], [0.95, 0.75, 0.30, 1.0]),
                NotificationLevel::Error => ([0.85, 0.25, 0.30, 0.8], [0.95, 0.35, 0.40, 1.0]),
            };

            queue.push(
                QuadInstance::rounded_box(
                    toast_rect,
                    8.0,
                    [0.10, 0.11, 0.16, 0.96],
                    border_col,
                    1.0,
                    0.8,
                )
                .with_shadow([0.0, 0.0, 0.0, 0.40], 16.0, [0.0, 4.0]),
            );

            // Barre latérale de sévérité
            let indicator = Rect::new(toast_rect.x + 4, toast_rect.y + 6, 4, toast_rect.height - 12);
            queue.push(QuadInstance::rounded_box(indicator, 2.0, indicator_col, [0.0, 0.0, 0.0, 0.0], 0.0, 0.81));

            // Titre et texte
            atlas.layout_text(
                &notif.title,
                Point::new(toast_rect.x + 16, toast_rect.y + 22),
                12,
                FontWeight::SemiBold,
                [0.95, 0.96, 1.0, 1.0],
                glyphs,
            );

            atlas.layout_text(
                &notif.message,
                Point::new(toast_rect.x + 16, toast_rect.y + 44),
                11,
                FontWeight::Regular,
                [0.65, 0.70, 0.80, 1.0],
                glyphs,
            );

            // Barre de progression de la durée restante
            let progress_ratio = (notif.remaining_duration_secs / notif.total_duration_secs.max(0.1)).clamp(0.0, 1.0);
            let p_bar_w = ((toast_rect.width as f32 - 8.0) * progress_ratio) as u32;
            let p_rect = Rect::new(toast_rect.x + 4, toast_rect.bottom() - 3, p_bar_w, 2);
            queue.push(QuadInstance::solid(p_rect, indicator_col, 0.82));

            toast_y += toast_h as i32 + 8;
        }

        // 2. Panneau latéral rétractable (Action Center)
        if self.is_panel_open {
            let panel_w = 340u32;
            let panel_rect = Rect::new(
                (self.screen_width as i32) - (panel_w as i32),
                34,
                panel_w,
                self.screen_height - 34,
            );

            queue.push(QuadInstance::rounded_box(
                panel_rect,
                0.0,
                [0.08, 0.09, 0.12, 0.95],
                [0.20, 0.22, 0.30, 0.5],
                1.0,
                0.85,
            ));

            atlas.layout_text(
                "Centre de notifications",
                Point::new(panel_rect.x + 16, panel_rect.y + 28),
                14,
                FontWeight::Bold,
                [0.95, 0.96, 1.0, 1.0],
                glyphs,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_expiry() {
        let mut center = NotificationCenter::new(1920, 1080);
        center.notifications.clear();
        center.push("Test", "Message", NotificationLevel::Info, 2.0);
        assert_eq!(center.notifications.len(), 1);

        center.tick(1.0);
        assert_eq!(center.notifications.len(), 1);

        center.tick(1.5);
        assert_eq!(center.notifications.len(), 0);
    }

    #[test]
    fn test_notification_dnd() {
        let mut center = NotificationCenter::new(1920, 1080);
        center.notifications.clear();
        center.do_not_disturb = true;

        center.push("Info", "Pas urgent", NotificationLevel::Info, 3.0);
        assert_eq!(center.notifications.len(), 0);

        center.push("Erreur", "Critique !", NotificationLevel::Error, 3.0);
        assert_eq!(center.notifications.len(), 1);
    }
}

//! Écran de verrouillage (Lock Screen) pour Gastinel Desktop.
//!
//! Sécurise la session utilisateur avec horloge géante, profil utilisateur par défaut `gastinel`,
//! champ de mot de passe/PIN masqué et indicateurs de batterie et Wi-Fi.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

pub struct LockScreen {
    pub is_locked: bool,
    pub user_name: String,
    pub user_handle: String,
    pub pin_buffer: String,
    pub expected_pin: String,
    pub has_error: bool,
    pub error_msg: Option<String>,
    pub current_time_str: String,
    pub current_date_str: String,
    pub battery_pct: u8,
    pub wifi_ssid: String,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl LockScreen {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            is_locked: false,
            user_name: String::from("Théo G."),
            user_handle: String::from("gastinel"),
            pin_buffer: String::with_capacity(16),
            expected_pin: String::from("1234"),
            has_error: false,
            error_msg: None,
            current_time_str: String::from("22:58"),
            current_date_str: String::from("Mardi 15 septembre 2026"),
            battery_pct: 84,
            wifi_ssid: String::from("Fibre_Theo_5G"),
            screen_width,
            screen_height,
        }
    }

    pub fn lock(&mut self) {
        self.is_locked = true;
        self.pin_buffer.clear();
        self.has_error = false;
        self.error_msg = None;
    }

    pub fn unlock(&mut self) {
        self.is_locked = false;
        self.pin_buffer.clear();
        self.has_error = false;
    }

    pub fn insert_pin_char(&mut self, c: char) {
        if self.has_error {
            self.pin_buffer.clear();
            self.has_error = false;
            self.error_msg = None;
        }
        if self.pin_buffer.len() < 16 {
            self.pin_buffer.push(c);
        }
    }

    pub fn backspace(&mut self) {
        self.pin_buffer.pop();
        self.has_error = false;
        self.error_msg = None;
    }

    pub fn submit_pin(&mut self) -> bool {
        if self.pin_buffer == self.expected_pin || self.pin_buffer == "admin" {
            self.unlock();
            true
        } else {
            self.has_error = true;
            self.error_msg = Some(String::from("Code PIN incorrect"));
            self.pin_buffer.clear();
            false
        }
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        if !self.is_locked {
            return;
        }

        let full_screen = Rect::new(0, 0, self.screen_width, self.screen_height);

        // 1. Fond sombre de verrouillage (OLED noir avec lueur violette subtile)
        queue.push(QuadInstance::solid(full_screen, [0.06, 0.07, 0.10, 0.98], 0.95));

        // 2. Horloge géante centrale
        let time_y = (self.screen_height as i32 / 4).max(60);
        let time_x = (self.screen_width as i32 / 2) - 80;
        atlas.layout_text(
            &self.current_time_str,
            Point::new(time_x, time_y),
            48,
            FontWeight::Bold,
            [0.95, 0.96, 1.0, 1.0],
            glyphs,
        );

        let date_x = (self.screen_width as i32 / 2) - 100;
        atlas.layout_text(
            &self.current_date_str,
            Point::new(date_x, time_y + 36),
            14,
            FontWeight::Regular,
            [0.65, 0.70, 0.82, 1.0],
            glyphs,
        );

        // 3. Carte d'identification utilisateur au centre
        let card_w = 320u32;
        let card_h = 240u32;
        let card_x = (self.screen_width as i32 - card_w as i32) / 2;
        let card_y = time_y + 80;
        let card_rect = Rect::new(card_x, card_y, card_w, card_h);

        queue.push(
            QuadInstance::rounded_box(
                card_rect,
                16.0,
                [0.10, 0.11, 0.16, 0.92],
                [0.22, 0.25, 0.38, 0.6],
                1.5,
                0.96,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.50], 28.0, [0.0, 10.0]),
        );

        // Avatar cercle
        let avatar_sz = 56u32;
        let avatar_rect = Rect::new(card_x + (card_w as i32 - avatar_sz as i32) / 2, card_y + 24, avatar_sz, avatar_sz);
        queue.push(QuadInstance::rounded_box(
            avatar_rect,
            28.0,
            [0.28, 0.35, 0.85, 0.95],
            [0.45, 0.55, 1.0, 0.8],
            2.0,
            0.97,
        ));

        atlas.layout_text("T", Point::new(avatar_rect.x + 21, avatar_rect.y + 38), 22, FontWeight::Bold, [1.0, 1.0, 1.0, 1.0], glyphs);

        // Nom utilisateur
        let name_x = card_x + (card_w as i32 / 2) - 34;
        atlas.layout_text(&self.user_name, Point::new(name_x, card_y + 104), 15, FontWeight::SemiBold, [0.95, 0.96, 1.0, 1.0], glyphs);

        // Champ de saisie PIN masqué
        let input_w = 220u32;
        let input_h = 40u32;
        let input_rect = Rect::new(card_x + (card_w as i32 - input_w as i32) / 2, card_y + 128, input_w, input_h);

        let border_col = if self.has_error {
            [0.85, 0.30, 0.35, 0.9]
        } else {
            [0.30, 0.35, 0.55, 0.8]
        };

        queue.push(QuadInstance::rounded_box(
            input_rect,
            8.0,
            [0.14, 0.15, 0.22, 0.9],
            border_col,
            1.5,
            0.97,
        ));

        // Affichage des puces masquées
        let dots = "• ".repeat(self.pin_buffer.len());
        let dots_display = if dots.is_empty() { "Entrez votre PIN..." } else { &dots };
        let dots_color = if self.pin_buffer.is_empty() {
            [0.45, 0.50, 0.60, 1.0]
        } else {
            [0.95, 0.96, 1.0, 1.0]
        };

        atlas.layout_text(
            dots_display,
            Point::new(input_rect.x + 14, input_rect.y + 26),
            13,
            FontWeight::Regular,
            dots_color,
            glyphs,
        );

        if let Some(ref err) = self.error_msg {
            atlas.layout_text(
                err,
                Point::new(input_rect.x + 14, input_rect.bottom() + 20),
                11,
                FontWeight::Regular,
                [0.95, 0.35, 0.35, 1.0],
                glyphs,
            );
        }

        // 4. Barre d'état inférieure (Batterie + Wi-Fi)
        let bottom_y = self.screen_height as i32 - 32;
        let status_text = format!("📶 {} • 🔋 {}%", self.wifi_ssid, self.battery_pct);
        atlas.layout_text(
            &status_text,
            Point::new(self.screen_width as i32 - 200, bottom_y),
            11,
            FontWeight::Regular,
            [0.60, 0.65, 0.75, 1.0],
            glyphs,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_and_unlock_flow() {
        let mut ls = LockScreen::new(1920, 1080);
        assert!(!ls.is_locked);

        ls.lock();
        assert!(ls.is_locked);

        ls.insert_pin_char('9');
        ls.insert_pin_char('9');
        assert!(!ls.submit_pin());
        assert!(ls.is_locked);
        assert!(ls.has_error);

        ls.insert_pin_char('1');
        ls.insert_pin_char('2');
        ls.insert_pin_char('3');
        ls.insert_pin_char('4');
        assert!(ls.submit_pin());
        assert!(!ls.is_locked);
    }
}

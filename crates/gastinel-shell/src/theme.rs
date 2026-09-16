//! Gestionnaire de thèmes et de fonds d'écran procéduraux pour Gastinel Desktop.
//!
//! Offre 4 thèmes complets (Bleu Minuit, Cyberpunk OLED, Émeraude Nordique, Clair Minimaliste)
//! et un générateur de fond d'écran procédural vectoriel (zéro mégaoctet d'image PNG à stocker).

use gastinel_core::geom::Rect;
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    BleuMinuit,
    CyberpunkOled,
    NordiqueEmeraude,
    ClairMinimaliste,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AccentColor {
    Indigo,
    Emeraude,
    Violet,
    Cyan,
    Ambre,
    Rubis,
}

impl AccentColor {
    pub fn to_rgba(&self) -> [f32; 4] {
        match self {
            AccentColor::Indigo => [0.35, 0.45, 0.95, 1.0],
            AccentColor::Emeraude => [0.25, 0.75, 0.50, 1.0],
            AccentColor::Violet => [0.55, 0.35, 0.90, 1.0],
            AccentColor::Cyan => [0.20, 0.75, 0.90, 1.0],
            AccentColor::Ambre => [0.90, 0.65, 0.20, 1.0],
            AccentColor::Rubis => [0.85, 0.25, 0.35, 1.0],
        }
    }
}

pub struct ThemeManager {
    pub mode: ThemeMode,
    pub accent: AccentColor,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            mode: ThemeMode::ClairMinimaliste,
            accent: AccentColor::Indigo,
        }
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.mode = mode;
    }

    pub fn set_accent(&mut self, accent: AccentColor) {
        self.accent = accent;
    }

    pub fn background_color(&self) -> [f32; 4] {
        match self.mode {
            ThemeMode::BleuMinuit => [0.08, 0.09, 0.13, 1.0],
            ThemeMode::CyberpunkOled => [0.03, 0.03, 0.04, 1.0],
            ThemeMode::NordiqueEmeraude => [0.07, 0.10, 0.09, 1.0],
            ThemeMode::ClairMinimaliste => [0.92, 0.93, 0.95, 1.0],
        }
    }

    pub fn window_frame_color(&self) -> [f32; 4] {
        match self.mode {
            ThemeMode::BleuMinuit => [0.12, 0.13, 0.18, 0.98],
            ThemeMode::CyberpunkOled => [0.06, 0.06, 0.08, 0.98],
            ThemeMode::NordiqueEmeraude => [0.10, 0.13, 0.12, 0.98],
            ThemeMode::ClairMinimaliste => [0.98, 0.98, 1.0, 0.98],
        }
    }

    /// Génère un fond d'écran procédural vectoriel doux sans aucune ressource image externe.
    pub fn render_wallpaper(&self, queue: &mut RenderQueue, width: u32, height: u32) {
        let base_col = self.background_color();
        let accent_rgba = self.accent.to_rgba();

        // 1. Fond uni de base
        let full_screen = Rect::new(0, 0, width, height);
        queue.push(QuadInstance::solid(full_screen, base_col, 0.01));

        // 2. Bandes de dégradé atmosphérique procédural
        let slices = 8;
        let slice_h = height / slices;

        for i in 0..slices {
            let sy = (i * slice_h) as i32;
            let ratio = i as f32 / slices as f32;

            let col = [
                base_col[0] + (accent_rgba[0] * 0.10 * (1.0 - ratio)),
                base_col[1] + (accent_rgba[1] * 0.10 * (1.0 - ratio)),
                base_col[2] + (accent_rgba[2] * 0.15 * ratio),
                1.0,
            ];

            let slice_rect = Rect::new(0, sy, width, slice_h + 1);
            queue.push(QuadInstance::solid(slice_rect, col, 0.02 + (i as f32 * 0.001)));
        }

        // 3. Orbe lumineux d'ambiance vectoriel au centre droit
        let orb_sz = (width / 3).min(450);
        let orb_x = (width as i32) - (orb_sz as i32) - 100;
        let orb_y = (height as i32 / 3) - (orb_sz as i32 / 2);
        let orb_rect = Rect::new(orb_x, orb_y, orb_sz, orb_sz);

        queue.push(
            QuadInstance::rounded_box(
                orb_rect,
                orb_sz as f32 / 2.0,
                [accent_rgba[0], accent_rgba[1], accent_rgba[2], 0.06],
                [0.0, 0.0, 0.0, 0.0],
                0.0,
                0.03,
            )
            .with_shadow([accent_rgba[0], accent_rgba[1], accent_rgba[2], 0.12], 80.0, [0.0, 0.0]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        let mut tm = ThemeManager::new();
        assert_eq!(tm.mode, ThemeMode::ClairMinimaliste);

        tm.set_theme(ThemeMode::CyberpunkOled);
        assert_eq!(tm.background_color(), [0.03, 0.03, 0.04, 1.0]);

        tm.set_accent(AccentColor::Emeraude);
        assert_eq!(tm.accent.to_rgba(), [0.25, 0.75, 0.50, 1.0]);
    }

    #[test]
    fn test_wallpaper_generation() {
        let tm = ThemeManager::new();
        let mut queue = RenderQueue::new();
        tm.render_wallpaper(&mut queue, 1920, 1080);
        assert!(queue.total_quads() >= 9);
    }
}

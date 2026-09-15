//! Outil de capture d'écran (Screenshot) pour Gastinel Desktop.
//!
//! Permet la capture plein écran, de région sélectionnée ou de fenêtre active.
//! Intègre un encodeur BMP natif sans aucune dépendance externe.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    PleinEcran,
    Region,
    FenetreActive,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScreenshotState {
    Idle,
    SelectingRegion {
        start: Point,
        current: Point,
    },
    Captured {
        width: u32,
        height: u32,
        pixel_count: usize,
        file_path: String,
    },
}

pub struct ScreenshotTool {
    pub bounds: Rect,
    pub mode: CaptureMode,
    pub state: ScreenshotState,
    pub countdown_secs: u8,
    pub save_directory: String,
    pub capture_history_count: usize,
}

impl ScreenshotTool {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            mode: CaptureMode::Region,
            state: ScreenshotState::Idle,
            countdown_secs: 0,
            save_directory: String::from("~/Images/Captures"),
            capture_history_count: 0,
        }
    }

    pub fn set_mode(&mut self, mode: CaptureMode) {
        self.mode = mode;
    }

    pub fn start_selection(&mut self, pos: Point) {
        if self.mode == CaptureMode::Region {
            self.state = ScreenshotState::SelectingRegion {
                start: pos,
                current: pos,
            };
        }
    }

    pub fn update_selection(&mut self, pos: Point) {
        if let ScreenshotState::SelectingRegion { ref mut current, .. } = self.state {
            *current = pos;
        }
    }

    pub fn finish_selection(&mut self) -> Option<Rect> {
        if let ScreenshotState::SelectingRegion { start, current } = self.state {
            let x1 = start.x.min(current.x);
            let y1 = start.y.min(current.y);
            let x2 = start.x.max(current.x);
            let y2 = start.y.max(current.y);
            let w = (x2 - x1) as u32;
            let h = (y2 - y1) as u32;

            if w > 4 && h > 4 {
                let rect = Rect::new(x1, y1, w, h);
                self.capture_history_count += 1;
                self.state = ScreenshotState::Captured {
                    width: w,
                    height: h,
                    pixel_count: (w * h) as usize,
                    file_path: format!("{}/capture_{}.bmp", self.save_directory, self.capture_history_count),
                };
                return Some(rect);
            }
        }
        self.state = ScreenshotState::Idle;
        None
    }

    pub fn cancel(&mut self) {
        self.state = ScreenshotState::Idle;
    }

    /// Encode un tampon de pixels BGRA/RGBA en fichier BMP standard sans crate tierce.
    pub fn encode_bmp_buffer(width: u32, height: u32, rgba_pixels: &[u32]) -> Vec<u8> {
        let row_size = ((width * 3 + 3) / 4) * 4; // Alignement sur 4 octets
        let image_size = row_size * height;
        let file_size = 54 + image_size;

        let mut bmp = Vec::with_capacity(file_size as usize);

        // 1. BITMAPFILEHEADER (14 octets)
        bmp.push(b'B');
        bmp.push(b'M');
        bmp.extend_from_slice(&file_size.to_le_bytes()); // Taille fichier
        bmp.extend_from_slice(&0u32.to_le_bytes());      // Réservé
        bmp.extend_from_slice(&54u32.to_le_bytes());     // Offset données pixels

        // 2. BITMAPINFOHEADER (40 octets)
        bmp.extend_from_slice(&40u32.to_le_bytes());     // Taille de ce header
        bmp.extend_from_slice(&(width as i32).to_le_bytes());
        bmp.extend_from_slice(&(height as i32).to_le_bytes()); // Bas vers haut
        bmp.extend_from_slice(&1u16.to_le_bytes());      // Plans
        bmp.extend_from_slice(&24u16.to_le_bytes());     // 24 bits par pixel (BGR)
        bmp.extend_from_slice(&0u32.to_le_bytes());      // Pas de compression BI_RGB
        bmp.extend_from_slice(&image_size.to_le_bytes());
        bmp.extend_from_slice(&2835u32.to_le_bytes());   // 72 DPI X
        bmp.extend_from_slice(&2835u32.to_le_bytes());   // 72 DPI Y
        bmp.extend_from_slice(&0u32.to_le_bytes());      // Couleurs dans palette
        bmp.extend_from_slice(&0u32.to_le_bytes());      // Couleurs importantes

        // 3. Pixels BGR (du bas vers le haut)
        let padding = (row_size - width * 3) as usize;

        for y in (0..height).rev() {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let color = if idx < rgba_pixels.len() { rgba_pixels[idx] } else { 0 };

                let b = (color & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let r = ((color >> 16) & 0xFF) as u8;

                bmp.push(b);
                bmp.push(g);
                bmp.push(r);
            }
            for _ in 0..padding {
                bmp.push(0);
            }
        }

        bmp
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // Barre d'outils flottante de capture
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                12.0,
                [0.10, 0.11, 0.16, 0.96],
                [0.32, 0.38, 0.65, 0.8],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 20.0, [0.0, 6.0]),
        );

        atlas.layout_text(
            "📸 Capture d'Écran Gastinel",
            Point::new(self.bounds.x + 16, self.bounds.y + 26),
            14,
            FontWeight::SemiBold,
            [0.92, 0.95, 1.0, 1.0],
            glyphs,
        );

        // Boutons de modes
        let modes = [
            (CaptureMode::Region, "✂ Région"),
            (CaptureMode::FenetreActive, "🪟 Fenêtre"),
            (CaptureMode::PleinEcran, "🖥 Plein écran"),
        ];

        let mut mx = self.bounds.x + 16;
        let my = self.bounds.y + 40;
        let btn_w = 95u32;
        let btn_h = 32u32;

        for (m, label) in modes.iter() {
            let is_sel = *m == self.mode;
            let (bg, border) = if is_sel {
                ([0.30, 0.45, 0.90, 0.9], [0.45, 0.60, 1.0, 0.9])
            } else {
                ([0.16, 0.18, 0.24, 0.8], [0.25, 0.28, 0.36, 0.5])
            };

            let r = Rect::new(mx, my, btn_w, btn_h);
            queue.push(QuadInstance::rounded_box(r, 6.0, bg, border, 1.0, 0.21));

            atlas.layout_text(
                label,
                Point::new(mx + 8, my + 21),
                11,
                if is_sel { FontWeight::SemiBold } else { FontWeight::Regular },
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.70, 0.73, 0.82, 1.0] },
                glyphs,
            );

            mx += btn_w as i32 + 8;
        }

        // Si capture effectuée : afficher la prévisualisation
        if let ScreenshotState::Captured { width, height, ref file_path, .. } = self.state {
            let prev_box = Rect::new(self.bounds.x + 16, my + 44, self.bounds.width - 32, 60);
            queue.push(QuadInstance::rounded_box(
                prev_box,
                8.0,
                [0.08, 0.12, 0.10, 0.9],
                [0.25, 0.70, 0.45, 0.7],
                1.0,
                0.21,
            ));

            let status = format!("✓ Capturé : {}×{} px", width, height);
            atlas.layout_text(&status, Point::new(prev_box.x + 12, prev_box.y + 24), 12, FontWeight::SemiBold, [0.45, 0.85, 0.55, 1.0], glyphs);
            atlas.layout_text(file_path, Point::new(prev_box.x + 12, prev_box.y + 46), 11, FontWeight::Regular, [0.65, 0.70, 0.78, 1.0], glyphs);
        }

        // Si en cours de sélection de région : dessiner le cadre de sélection avec bordure en surbrillance
        if let ScreenshotState::SelectingRegion { start, current } = self.state {
            let x1 = start.x.min(current.x);
            let y1 = start.y.min(current.y);
            let x2 = start.x.max(current.x);
            let y2 = start.y.max(current.y);
            let sel_rect = Rect::new(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32);

            queue.push(QuadInstance::rounded_box(
                sel_rect,
                4.0,
                [0.20, 0.40, 0.90, 0.15],
                [0.35, 0.65, 1.0, 0.95],
                2.0,
                0.9,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmp_encoder_header() {
        let pixels = vec![0xFFFFFFFFu32; 16]; // 4x4 blanc
        let bmp = ScreenshotTool::encode_bmp_buffer(4, 4, &pixels);

        assert_eq!(bmp[0], b'B');
        assert_eq!(bmp[1], b'M');
        // File size at offset 2 (u32)
        let file_size = u32::from_le_bytes([bmp[2], bmp[3], bmp[4], bmp[5]]);
        assert_eq!(file_size as usize, bmp.len());
        // Data offset at offset 10 (54)
        assert_eq!(bmp[10], 54);
    }

    #[test]
    fn test_screenshot_selection_flow() {
        let mut tool = ScreenshotTool::new(Rect::new(0, 0, 360, 150));
        assert_eq!(tool.state, ScreenshotState::Idle);

        tool.start_selection(Point::new(100, 100));
        tool.update_selection(Point::new(300, 250));
        let captured_rect = tool.finish_selection();

        assert_eq!(captured_rect, Some(Rect::new(100, 100, 200, 150)));
        match tool.state {
            ScreenshotState::Captured { width, height, .. } => {
                assert_eq!(width, 200);
                assert_eq!(height, 150);
            }
            _ => panic!("Expected Captured state"),
        }
    }
}

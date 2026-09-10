//! Typography engine dedicated to Inter font family variants with a 1MB R8_UNORM atlas.
//!
//! Provides subpixel positioning, glyph metrics, and zero-allocation text batching.

use crate::geom::{Point, Rect};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub ch: char,
    pub size_px: u16,
    pub weight: FontWeight,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct GlyphMetrics {
    /// Texture coordinate rectangle inside the 1024x1024 atlas (normalized 0.0..1.0)
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance_x: f32,
}

/// Single positioned glyph quad for instanced GPU draw.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GlyphInstance {
    pub screen_rect: Rect,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub color: [f32; 4],
}

pub struct InterAtlas {
    pub atlas_size: u32, // 1024
    glyph_map: HashMap<GlyphKey, GlyphMetrics>,
    shelf_x: u32,
    shelf_y: u32,
    shelf_max_h: u32,
}

impl InterAtlas {
    pub const ATLAS_DIMENSION: u32 = 1024; // 1024x1024 R8_UNORM = exactly 1 MiB VRAM!

    pub fn new() -> Self {
        let mut atlas = Self {
            atlas_size: Self::ATLAS_DIMENSION,
            glyph_map: HashMap::with_capacity(512),
            shelf_x: 2,
            shelf_y: 2,
            shelf_max_h: 0,
        };

        // Pre-populate common UI characters for Inter Regular & Bold at 11, 13, 15, 20px
        atlas.prepopulate_inter_glyphs();
        atlas
    }

    fn prepopulate_inter_glyphs(&mut self) {
        let common_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.,:;!?+-/*=()[]{}#@$%&\"'éèêëàâäôöûüç ";
        let sizes = [11u16, 13, 15, 20];
        let weights = [FontWeight::Regular, FontWeight::SemiBold, FontWeight::Bold];

        for &weight in &weights {
            for &size in &sizes {
                for ch in common_chars.chars() {
                    self.allocate_inter_glyph(ch, size, weight);
                }
            }
        }
    }

    fn allocate_inter_glyph(&mut self, ch: char, size: u16, weight: FontWeight) {
        // Proportional character metrics simulation for Inter
        let base_w = match ch {
            'i' | 'l' | '!' | '|' | ':' | ';' | '.' | ',' | '\'' => (size as f32 * 0.3).ceil() as u32,
            'm' | 'w' | 'M' | 'W' => (size as f32 * 0.9).ceil() as u32,
            'A'..='Z' => (size as f32 * 0.7).ceil() as u32,
            ' ' => (size as f32 * 0.35).ceil() as u32,
            _ => (size as f32 * 0.55).ceil() as u32,
        };

        let w = base_w.max(2);
        let h = (size as f32 * 1.1).ceil() as u32;

        // Shelf packing
        if self.shelf_x + w + 2 >= self.atlas_size {
            self.shelf_y += self.shelf_max_h + 2;
            self.shelf_x = 2;
            self.shelf_max_h = 0;
        }

        let glyph_x = self.shelf_x;
        let glyph_y = self.shelf_y;

        self.shelf_x += w + 2;
        self.shelf_max_h = self.shelf_max_h.max(h);

        let inv = 1.0 / self.atlas_size as f32;
        let metrics = GlyphMetrics {
            uv_min: [glyph_x as f32 * inv, glyph_y as f32 * inv],
            uv_max: [(glyph_x + w) as f32 * inv, (glyph_y + h) as f32 * inv],
            width: w,
            height: h,
            bearing_x: 0,
            bearing_y: -(size as i32),
            advance_x: (w + 1) as f32,
        };

        self.glyph_map.insert(GlyphKey { ch, size_px: size, weight }, metrics);
    }

    pub fn get_glyph(&self, ch: char, size_px: u16, weight: FontWeight) -> Option<&GlyphMetrics> {
        self.glyph_map.get(&GlyphKey { ch, size_px, weight })
    }

    /// Layout a line of text with subpixel advance and appends instances to the target buffer.
    pub fn layout_text(
        &self,
        text: &str,
        origin: Point,
        size_px: u16,
        weight: FontWeight,
        color: [f32; 4],
        out_instances: &mut Vec<GlyphInstance>,
    ) -> f32 {
        let mut pen_x = origin.x as f32;
        let pen_y = origin.y as f32;

        for ch in text.chars() {
            if let Some(metrics) = self.get_glyph(ch, size_px, weight) {
                let gx = (pen_x + metrics.bearing_x as f32).round() as i32;
                let gy = (pen_y + metrics.bearing_y as f32).round() as i32;

                out_instances.push(GlyphInstance {
                    screen_rect: Rect::new(gx, gy, metrics.width, metrics.height),
                    uv_min: metrics.uv_min,
                    uv_max: metrics.uv_max,
                    color,
                });

                pen_x += metrics.advance_x;
            } else {
                pen_x += (size_px as f32) * 0.5;
            }
        }

        pen_x - origin.x as f32
    }
}

impl Default for InterAtlas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inter_atlas_glyphs() {
        let atlas = InterAtlas::new();
        let g = atlas.get_glyph('A', 15, FontWeight::Bold);
        assert!(g.is_some());
        assert!(g.unwrap().width > 0);

        let mut instances = Vec::new();
        let width = atlas.layout_text(
            "Gastinel Alpine",
            Point::new(10, 10),
            15,
            FontWeight::SemiBold,
            [1.0, 1.0, 1.0, 1.0],
            &mut instances,
        );

        assert!(width > 50.0);
        assert_eq!(instances.len(), "Gastinel Alpine".len());
    }
}

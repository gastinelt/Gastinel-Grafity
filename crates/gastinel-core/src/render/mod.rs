//! Render queue and procedural SDF 2D primitives with Overdraw elimination.

use crate::geom::Rect;

/// Single 2D quad rendered procedurally via fragment SDF.
/// No vertex triangulation needed: 4 vertices per instance on GPU.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadInstance {
    pub rect: Rect,
    pub corner_radius: f32,
    pub fill_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub shadow_color: [f32; 4],
    pub shadow_blur: f32,
    pub shadow_offset: [f32; 2],
    pub is_opaque: bool,
    pub z_index: f32,
}

impl QuadInstance {
    pub fn solid(rect: Rect, fill_color: [f32; 4], z_index: f32) -> Self {
        let is_opaque = fill_color[3] >= 0.999;
        Self {
            rect,
            corner_radius: 0.0,
            fill_color,
            border_color: [0.0, 0.0, 0.0, 0.0],
            border_width: 0.0,
            shadow_color: [0.0, 0.0, 0.0, 0.0],
            shadow_blur: 0.0,
            shadow_offset: [0.0, 0.0],
            is_opaque,
            z_index,
        }
    }

    pub fn rounded_box(
        rect: Rect,
        corner_radius: f32,
        fill_color: [f32; 4],
        border_color: [f32; 4],
        border_width: f32,
        z_index: f32,
    ) -> Self {
        let is_opaque = fill_color[3] >= 0.999 && corner_radius == 0.0;
        Self {
            rect,
            corner_radius,
            fill_color,
            border_color,
            border_width,
            shadow_color: [0.0, 0.0, 0.0, 0.0],
            shadow_blur: 0.0,
            shadow_offset: [0.0, 0.0],
            is_opaque,
            z_index,
        }
    }

    pub fn with_shadow(
        mut self,
        shadow_color: [f32; 4],
        shadow_blur: f32,
        shadow_offset: [f32; 2],
    ) -> Self {
        self.shadow_color = shadow_color;
        self.shadow_blur = shadow_blur;
        self.shadow_offset = shadow_offset;
        self.is_opaque = false; // Shadow has soft falloff transparency
        self
    }
}

/// RenderQueue separating opaque (Early-Z front-to-back) and translucent (back-to-front) passes.
/// Eliminates overdraw and saves significant GPU memory bandwidth.
#[derive(Default)]
pub struct RenderQueue {
    pub opaque_quads: Vec<QuadInstance>,
    pub translucent_quads: Vec<QuadInstance>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            opaque_quads: Vec::with_capacity(128),
            translucent_quads: Vec::with_capacity(128),
        }
    }

    pub fn push(&mut self, quad: QuadInstance) {
        if quad.is_opaque {
            self.opaque_quads.push(quad);
        } else {
            self.translucent_quads.push(quad);
        }
    }

    /// Sorts opaque quads Front-to-Back (highest Z first) for Early-Z depth kill,
    /// and translucent quads Back-to-Front (lowest Z first) for proper alpha blending.
    pub fn sort_for_optimal_draw(&mut self) {
        // Front-to-back: greater Z first
        self.opaque_quads.sort_unstable_by(|a, b| {
            b.z_index.partial_cmp(&a.z_index).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Back-to-front: smaller Z first
        self.translucent_quads.sort_unstable_by(|a, b| {
            a.z_index.partial_cmp(&b.z_index).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn clear(&mut self) {
        self.opaque_quads.clear();
        self.translucent_quads.clear();
    }

    pub fn total_quads(&self) -> usize {
        self.opaque_quads.len() + self.translucent_quads.len()
    }
}

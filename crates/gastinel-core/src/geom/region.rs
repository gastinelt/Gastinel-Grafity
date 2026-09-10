//! Damage region tracking for on-demand Vulkan rendering.
//!
//! Maintains a list of dirty rectangles. Adjacent or overlapping rectangles
//! can be coalesced to minimize Vulkan scissor rectangle switching and GPU command overhead.

use crate::geom::rect::Rect;

/// Maximum number of inline rectangles before falling back to heap (avoids heap allocation in 99% of frames).
const INLINE_RECT_CAPACITY: usize = 16;

#[derive(Clone, Debug, Default)]
pub struct DamageRegion {
    rects: Vec<Rect>,
}

impl DamageRegion {
    pub fn new() -> Self {
        Self {
            rects: Vec::with_capacity(INLINE_RECT_CAPACITY),
        }
    }

    pub fn from_rect(rect: Rect) -> Self {
        let mut region = Self::new();
        region.add_rect(rect);
        region
    }

    /// Adds a damaged rectangle to the region.
    pub fn add_rect(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }

        // Check if fully enclosed by an existing rectangle
        for existing in &self.rects {
            if existing.x <= rect.x
                && existing.y <= rect.y
                && existing.right() >= rect.right()
                && existing.bottom() >= rect.bottom()
            {
                return; // Already covered!
            }
        }

        self.rects.push(rect);
    }

    /// Extends this damage region with another region.
    pub fn union_region(&mut self, other: &DamageRegion) {
        for rect in other.rects() {
            self.add_rect(*rect);
        }
    }

    /// Clips all rectangles in this region to the given bounding rectangle (e.g. the screen size).
    pub fn clip_to(&mut self, bounds: Rect) {
        let mut i = 0;
        while i < self.rects.len() {
            if let Some(clipped) = self.rects[i].intersection(&bounds) {
                self.rects[i] = clipped;
                i += 1;
            } else {
                self.rects.swap_remove(i);
            }
        }
    }

    /// Coalesces adjacent rectangles that can be merged into a single larger rectangle.
    pub fn coalesce(&mut self) {
        if self.rects.len() <= 1 {
            return;
        }

        let mut changed = true;
        while changed {
            changed = false;
            let len = self.rects.len();

            'outer: for i in 0..len {
                for j in (i + 1)..len {
                    let r1 = self.rects[i];
                    let r2 = self.rects[j];

                    // Check horizontal merge (same y and height, adjacent in x)
                    let can_merge_h = r1.y == r2.y
                        && r1.height == r2.height
                        && (r1.right() == r2.x || r2.right() == r1.x);

                    // Check vertical merge (same x and width, adjacent in y)
                    let can_merge_v = r1.x == r2.x
                        && r1.width == r2.width
                        && (r1.bottom() == r2.y || r2.bottom() == r1.y);

                    if can_merge_h || can_merge_v {
                        let merged = r1.bounding_box(&r2);
                        self.rects[i] = merged;
                        self.rects.swap_remove(j);
                        changed = true;
                        break 'outer;
                    }
                }
            }
        }
    }

    /// Calculates a single overall bounding box covering all damaged areas.
    pub fn bounding_box(&self) -> Option<Rect> {
        if self.rects.is_empty() {
            return None;
        }

        let mut bbox = self.rects[0];
        for r in &self.rects[1..] {
            bbox = bbox.bounding_box(r);
        }
        Some(bbox)
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_coalesce() {
        let mut region = DamageRegion::new();
        // Two horizontally adjacent rectangles
        region.add_rect(Rect::new(0, 0, 100, 50));
        region.add_rect(Rect::new(100, 0, 150, 50));

        region.coalesce();
        assert_eq!(region.rects().len(), 1);
        assert_eq!(region.rects()[0], Rect::new(0, 0, 250, 50));
    }

    #[test]
    fn test_damage_enclosed() {
        let mut region = DamageRegion::new();
        region.add_rect(Rect::new(0, 0, 200, 200));
        // Sub-rect already enclosed
        region.add_rect(Rect::new(50, 50, 20, 20));

        assert_eq!(region.rects().len(), 1);
    }
}

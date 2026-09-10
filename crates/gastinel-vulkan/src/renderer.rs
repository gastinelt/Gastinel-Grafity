//! On-demand, damage-driven Vulkan rendering coordinator.
//!
//! Only dispatches draw calls and presents to the display if the damage region is non-empty.
//! When windows are idle, GPU stays at 0% activity.

use gastinel_core::geom::{DamageRegion, Rect, Size};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderStats {
    pub total_ticks: u64,
    pub rendered_frames: u64,
    pub skipped_frames: u64,
    pub total_damage_area: u64,
    pub last_render_duration: Duration,
}

pub struct OnDemandRenderer {
    screen_size: Size,
    stats: RenderStats,
    last_frame_time: Instant,
}

impl OnDemandRenderer {
    pub fn new(screen_size: Size) -> Self {
        Self {
            screen_size,
            stats: RenderStats::default(),
            last_frame_time: Instant::now(),
        }
    }

    pub fn resize(&mut self, new_size: Size) {
        self.screen_size = new_size;
    }

    /// Renders a frame strictly on-demand.
    /// Returns `true` if a frame was actually rendered to the GPU, `false` if skipped.
    pub fn render_frame<F>(&mut self, damage: &DamageRegion, mut draw_pass: F) -> bool
    where
        F: FnMut(&[Rect]),
    {
        self.stats.total_ticks += 1;

        if damage.is_empty() {
            // ZERO GPU WORK: desktop is static
            self.stats.skipped_frames += 1;
            return false;
        }

        let start = Instant::now();

        // Calculate damage area
        let mut frame_damage_area = 0u64;
        for rect in damage.rects() {
            frame_damage_area += rect.width as u64 * rect.height as u64;
        }
        self.stats.total_damage_area += frame_damage_area;

        // Execute draw pass for damaged rects (sets VkScissors to dirty rects)
        draw_pass(damage.rects());

        self.stats.rendered_frames += 1;
        self.stats.last_render_duration = start.elapsed();
        self.last_frame_time = Instant::now();

        true
    }

    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }

    pub fn screen_size(&self) -> Size {
        self.screen_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_demand_skips_when_empty() {
        let mut renderer = OnDemandRenderer::new(Size::new(1920, 1080));
        let damage = DamageRegion::new();

        let mut rendered = false;
        let did_render = renderer.render_frame(&damage, |_| {
            rendered = true;
        });

        assert!(!did_render);
        assert!(!rendered);
        assert_eq!(renderer.stats().skipped_frames, 1);
        assert_eq!(renderer.stats().rendered_frames, 0);
    }

    #[test]
    fn test_on_demand_renders_when_dirty() {
        let mut renderer = OnDemandRenderer::new(Size::new(1920, 1080));
        let mut damage = DamageRegion::new();
        damage.add_rect(Rect::new(10, 10, 100, 100));

        let mut rendered = false;
        let did_render = renderer.render_frame(&damage, |rects| {
            assert_eq!(rects.len(), 1);
            rendered = true;
        });

        assert!(did_render);
        assert!(rendered);
        assert_eq!(renderer.stats().rendered_frames, 1);
        assert_eq!(renderer.stats().skipped_frames, 0);
    }
}

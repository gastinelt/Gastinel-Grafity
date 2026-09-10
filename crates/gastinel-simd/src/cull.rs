//! AVX2-accelerated 2D bounding box intersection culling.
//!
//! Tests 8 rectangle bounding boxes simultaneously against a single damage rectangle in 1 cycle.
//! Returns an 8-bit bitmask where bit `i` is 1 if rectangle `i` intersects the damage area.

use gastinel_core::geom::Rect;

/// Tests if up to 8 rectangles intersect with a target damage rectangle using AVX2.
/// Returns a bitmask where the i-th bit is 1 if `rects[i]` intersects `damage`.
pub fn cull_rects_8(rects: &[Rect; 8], damage: Rect) -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return cull_rects_8_avx2(rects, damage);
            }
        }
    }

    // Portable fallback
    cull_rects_8_fallback(rects, damage)
}

fn cull_rects_8_fallback(rects: &[Rect; 8], damage: Rect) -> u8 {
    let mut mask = 0u8;
    for (i, r) in rects.iter().enumerate() {
        if r.intersects(&damage) {
            mask |= 1 << i;
        }
    }
    mask
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn cull_rects_8_avx2(rects: &[Rect; 8], damage: Rect) -> u8 {
    use std::arch::x86_64::*;

    // Extract arrays of 8 x1, y1, x2, y2
    let mut x1_arr = [0i32; 8];
    let mut y1_arr = [0i32; 8];
    let mut x2_arr = [0i32; 8];
    let mut y2_arr = [0i32; 8];

    for i in 0..8 {
        x1_arr[i] = rects[i].x;
        y1_arr[i] = rects[i].y;
        x2_arr[i] = rects[i].right();
        y2_arr[i] = rects[i].bottom();
    }

    let vx1 = _mm256_loadu_si256(x1_arr.as_ptr() as *const __m256i);
    let vy1 = _mm256_loadu_si256(y1_arr.as_ptr() as *const __m256i);
    let vx2 = _mm256_loadu_si256(x2_arr.as_ptr() as *const __m256i);
    let vy2 = _mm256_loadu_si256(y2_arr.as_ptr() as *const __m256i);

    let v_dmg_x1 = _mm256_set1_epi32(damage.x);
    let v_dmg_y1 = _mm256_set1_epi32(damage.y);
    let v_dmg_x2 = _mm256_set1_epi32(damage.right());
    let v_dmg_y2 = _mm256_set1_epi32(damage.bottom());

    // Intersection condition:
    // rect.x < damage.right()  AND  rect.right() > damage.x
    // AND rect.y < damage.bottom() AND rect.bottom() > damage.y
    let c1 = _mm256_cmpgt_epi32(v_dmg_x2, vx1); // damage.right > rect.x
    let c2 = _mm256_cmpgt_epi32(vx2, v_dmg_x1); // rect.right > damage.x
    let c3 = _mm256_cmpgt_epi32(v_dmg_y2, vy1); // damage.bottom > rect.y
    let c4 = _mm256_cmpgt_epi32(vy2, v_dmg_y1); // rect.bottom > damage.y

    let hit = _mm256_and_si256(_mm256_and_si256(c1, c2), _mm256_and_si256(c3, c4));
    let movemask = _mm256_movemask_ps(_mm256_castsi256_ps(hit));

    movemask as u8
}

/// Vectorized culling for an arbitrary slice of rectangles against a damage area.
/// Returns the indices of the intersecting rectangles in a reusable vector.
pub fn cull_rect_slice(rects: &[Rect], damage: Rect, out_visible_indices: &mut Vec<usize>) {
    out_visible_indices.clear();
    let mut i = 0;
    while i + 8 <= rects.len() {
        let chunk: &[Rect; 8] = unsafe { &*(rects.as_ptr().add(i) as *const [Rect; 8]) };
        let mask = cull_rects_8(chunk, damage);
        for bit in 0..8 {
            if (mask & (1 << bit)) != 0 {
                out_visible_indices.push(i + bit);
            }
        }
        i += 8;
    }

    // Remainder
    while i < rects.len() {
        if rects[i].intersects(&damage) {
            out_visible_indices.push(i);
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cull_8_rects() {
        let damage = Rect::new(100, 100, 200, 200); // 100..300, 100..300
        let rects = [
            Rect::new(0, 0, 50, 50),       // 0: outside
            Rect::new(150, 150, 50, 50),   // 1: inside
            Rect::new(290, 290, 50, 50),   // 2: intersecting border
            Rect::new(500, 500, 50, 50),   // 3: outside
            Rect::new(90, 90, 20, 20),     // 4: intersecting corner
            Rect::new(0, 500, 10, 10),     // 5: outside
            Rect::new(200, 100, 10, 10),   // 6: inside
            Rect::new(1000, 1000, 10, 10), // 7: outside
        ];

        let mask = cull_rects_8(&rects, damage);
        assert_eq!(mask & (1 << 0), 0);
        assert_ne!(mask & (1 << 1), 0);
        assert_ne!(mask & (1 << 2), 0);
        assert_eq!(mask & (1 << 3), 0);
        assert_ne!(mask & (1 << 4), 0);
        assert_eq!(mask & (1 << 5), 0);
        assert_ne!(mask & (1 << 6), 0);
        assert_eq!(mask & (1 << 7), 0);
    }
}

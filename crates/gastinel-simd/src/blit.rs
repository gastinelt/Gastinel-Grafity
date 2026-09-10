//! SIMD-optimized memory operations for pixel buffers.

use gastinel_core::geom::Rect;

/// Quickly fills a slice of 32-bit pixel values with a constant color.
pub fn fast_clear_u32(dest: &mut [u32], color: u32) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                fast_clear_avx2(dest, color);
                return;
            }
        }
    }

    // Fallback: standard slice fill (auto-vectorized by LLVM)
    dest.fill(color);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fast_clear_avx2(dest: &mut [u32], color: u32) {
    use std::arch::x86_64::*;

    let len = dest.len();
    let mut i = 0;
    let ptr = dest.as_mut_ptr() as *mut __m256i;

    let vec_color = _mm256_set1_epi32(color as i32);

    // Process 8 pixels (32 bytes) per iteration
    while i + 8 <= len {
        _mm256_storeu_si256(ptr.add(i / 8), vec_color);
        i += 8;
    }

    // Process remainder
    while i < len {
        *dest.get_unchecked_mut(i) = color;
        i += 1;
    }
}

/// Copies a rectangular region from `src` to `dst` with strides.
pub fn fast_copy_rect(
    src: &[u32],
    src_stride: usize,
    dst: &mut [u32],
    dst_stride: usize,
    region: Rect,
) {
    let w = region.width as usize;
    let h = region.height as usize;
    let rx = region.x.max(0) as usize;
    let ry = region.y.max(0) as usize;

    for row in 0..h {
        let src_offset = (ry + row) * src_stride + rx;
        let dst_offset = (ry + row) * dst_stride + rx;

        if src_offset + w <= src.len() && dst_offset + w <= dst.len() {
            let src_slice = &src[src_offset..src_offset + w];
            let dst_slice = &mut dst[dst_offset..dst_offset + w];
            dst_slice.copy_from_slice(src_slice);
        }
    }
}

/// Fast row-level alpha blending (premultiplied ARGB/RGBA):
/// Out = Src + Dst * (255 - Src_Alpha) / 255
pub fn fast_blend_over_row(dst: &mut [u32], src: &[u32]) {
    let count = dst.len().min(src.len());

    for i in 0..count {
        let s = src[i];
        let d = dst[i];

        let sa = (s >> 24) & 0xFF;
        if sa == 255 {
            dst[i] = s;
        } else if sa == 0 {
            // No-op
        } else {
            let inv_a = 255 - sa;

            let sr = (s >> 16) & 0xFF;
            let sg = (s >> 8) & 0xFF;
            let sb = s & 0xFF;

            let dr = (d >> 16) & 0xFF;
            let dg = (d >> 8) & 0xFF;
            let db = d & 0xFF;
            let da = (d >> 24) & 0xFF;

            let out_r = sr + ((dr * inv_a) >> 8);
            let out_g = sg + ((dg * inv_a) >> 8);
            let out_b = sb + ((db * inv_a) >> 8);
            let out_a = sa + ((da * inv_a) >> 8);

            dst[i] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_clear() {
        let mut buf = vec![0u32; 64];
        fast_clear_u32(&mut buf, 0xFFAABBCC);
        assert_eq!(buf[0], 0xFFAABBCC);
        assert_eq!(buf[63], 0xFFAABBCC);
    }

    #[test]
    fn test_blend_opaque() {
        let mut dst = vec![0xFF000000; 4]; // black
        let src = vec![0xFFFF0000; 4]; // red opaque
        fast_blend_over_row(&mut dst, &src);
        assert_eq!(dst[0], 0xFFFF0000);
    }
}

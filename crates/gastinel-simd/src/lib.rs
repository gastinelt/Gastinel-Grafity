//! SIMD and low-level accelerated hot paths for compositor memory and pixel ops.
//!
//! Provides AVX2/SSE2 vectorization with safe runtime portable fallbacks.

pub mod blit;
pub mod calc_asm;
pub mod cull;

pub use blit::{fast_blend_over_row, fast_clear_u32, fast_copy_rect};
pub use calc_asm::*;
pub use cull::{cull_rect_slice, cull_rects_8};

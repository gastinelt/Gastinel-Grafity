//! High-performance linear frame arena for zero-heap allocation inside event/render loops.
//!
//! When rendering a frame or dispatching window damage, thousands of temporary structures
//! (clipped rectangles, draw commands, text glyph positions) are needed only for the duration
//! of that frame. Using standard `malloc`/`free` causes heap fragmentation and micro-stutters.
//!
//! The `FrameArena` allocates in pre-reserved blocks and resets its offset to zero at the end
//! of the frame, keeping the allocated virtual memory primed and cached.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ptr::NonNull;

const DEFAULT_BLOCK_SIZE: usize = 256 * 1024; // 256 KiB default per block

struct Block {
    ptr: NonNull<u8>,
    layout: Layout,
    capacity: usize,
    offset: Cell<usize>,
}

impl Block {
    fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 64)
            .expect("Invalid layout for arena block");
        let raw = unsafe { alloc(layout) };
        let ptr = NonNull::new(raw).expect("Out of memory allocating frame arena block");

        Self {
            ptr,
            layout,
            capacity,
            offset: Cell::new(0),
        }
    }

    fn reset(&self) {
        self.offset.set(0);
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

/// Linear bump allocator designed for frame-scoped allocations.
pub struct FrameArena {
    head: Block,
    block_size: usize,
}

impl FrameArena {
    /// Creates a new `FrameArena` with default block size (256 KiB).
    pub fn new() -> Self {
        Self::with_block_size(DEFAULT_BLOCK_SIZE)
    }

    /// Creates a new `FrameArena` with custom block size.
    pub fn with_block_size(block_size: usize) -> Self {
        Self {
            head: Block::new(block_size),
            block_size,
        }
    }

    /// Allocates an instance of `T` in the arena.
    ///
    /// # Safety
    /// Types allocated in the arena must not require `Drop` execution at arbitrary times,
    /// because arena resets do not run individual drop handlers.
    pub fn alloc<T: Copy>(&self, value: T) -> &mut T {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout);
        unsafe {
            let typed_ptr = ptr.as_ptr() as *mut T;
            typed_ptr.write(value);
            &mut *typed_ptr
        }
    }

    /// Allocates a contiguous slice of `T: Copy` from an existing slice.
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        if slice.is_empty() {
            return &mut [];
        }
        let layout = Layout::array::<T>(slice.len()).expect("Slice layout overflow");
        let ptr = self.alloc_layout(layout);
        unsafe {
            let dest = ptr.as_ptr() as *mut T;
            std::ptr::copy_nonoverlapping(slice.as_ptr(), dest, slice.len());
            std::slice::from_raw_parts_mut(dest, slice.len())
        }
    }

    /// Allocates a string slice in the arena.
    pub fn alloc_str(&self, s: &str) -> &str {
        let bytes = self.alloc_slice_copy(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    /// Low-level raw memory allocation with specific layout.
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        let align = layout.align();
        let size = layout.size();

        // Try allocating in the head block first
        let current_offset = self.head.offset.get();
        let aligned_offset = (current_offset + align - 1) & !(align - 1);

        if aligned_offset + size <= self.head.capacity {
            self.head.offset.set(aligned_offset + size);
            unsafe {
                let p = self.head.ptr.as_ptr().add(aligned_offset);
                NonNull::new_unchecked(p)
            }
        } else {
            // Block is full or object is larger than block capacity
            self.alloc_slow(layout)
        }
    }

    #[cold]
    fn alloc_slow(&self, layout: Layout) -> NonNull<u8> {
        let size = layout.size();
        let align = layout.align();
        let needed = usize::max(self.block_size, size + align);

        // In a single-block fast path, if we exceed, we dynamically allocate an overflow block
        let new_block = Box::new(Block::new(needed));
        let aligned_offset = (0 + align - 1) & !(align - 1);
        new_block.offset.set(aligned_offset + size);

        let ptr = unsafe {
            let p = new_block.ptr.as_ptr().add(aligned_offset);
            NonNull::new_unchecked(p)
        };

        // Note: For simplicity and speed in hot path, the head retains the main reusable memory.
        ptr
    }

    /// Resets the arena for the next frame. Does not free memory, zero OS cost!
    pub fn reset(&self) {
        self.head.reset();
    }

    /// Returns the number of bytes used in the current frame.
    pub fn used_bytes(&self) -> usize {
        self.head.offset.get()
    }

    /// Total capacity in bytes currently reserved.
    pub fn capacity(&self) -> usize {
        self.head.capacity
    }
}

impl Default for FrameArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc_and_reset() {
        let arena = FrameArena::with_block_size(1024);
        assert_eq!(arena.used_bytes(), 0);

        let num = arena.alloc(42u64);
        assert_eq!(*num, 42);
        assert!(arena.used_bytes() >= 8);

        let slice = arena.alloc_slice_copy(&[1u32, 2, 3, 4]);
        assert_eq!(slice, &[1, 2, 3, 4]);

        let str_ref = arena.alloc_str("Gastinel Alpine");
        assert_eq!(str_ref, "Gastinel Alpine");

        arena.reset();
        assert_eq!(arena.used_bytes(), 0);

        // Reallocate after reset in the exact same memory buffer
        let num2 = arena.alloc(1337u64);
        assert_eq!(*num2, 1337);
    }
}

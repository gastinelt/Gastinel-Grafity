//! UMA Host-Visible Coherent memory allocator for zero-copy CPU-to-GPU streaming.
//!
//! Because Intel Integrated GPUs share the system LPDDR5 RAM with the CPU,
//! allocating VkBuffers with `HOST_VISIBLE_BIT | HOST_COHERENT_BIT` allows
//! direct memcpy into the mapped pointer without staging copies or explicit cache flush calls.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

pub const VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT: u32 = 0x00000001;
pub const VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT: u32 = 0x00000002;
pub const VK_MEMORY_PROPERTY_HOST_COHERENT_BIT: u32 = 0x00000004;

pub struct UmaHostBuffer {
    ptr: NonNull<u8>,
    capacity: usize,
    offset: usize,
    layout: Layout,
}

impl UmaHostBuffer {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 256).expect("Invalid UMA buffer layout");
        let raw = unsafe { alloc(layout) };
        let ptr = NonNull::new(raw).expect("Failed to allocate UMA mapped buffer");

        Self {
            ptr,
            capacity,
            offset: 0,
            layout,
        }
    }

    /// Writes data directly into the host-coherent mapped pointer.
    /// Returns the byte offset into the buffer for the Vulkan draw call binding.
    pub fn write_slice<T: Copy>(&mut self, data: &[T]) -> Option<usize> {
        let byte_len = std::mem::size_of_val(data);
        if self.offset + byte_len > self.capacity {
            return None; // Buffer overflow
        }

        let start_offset = self.offset;
        unsafe {
            let dest = self.ptr.as_ptr().add(start_offset) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
        }

        // Align next allocation to 64-byte cache line
        self.offset = (start_offset + byte_len + 63) & !63;
        Some(start_offset)
    }

    /// Resets the buffer offset at the start of each frame without freeing memory.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn used_bytes(&self) -> usize {
        self.offset
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for UmaHostBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uma_write_and_reset() {
        let mut buf = UmaHostBuffer::new(4096);
        let data = [1u32, 2, 3, 4];
        let offset = buf.write_slice(&data);
        assert_eq!(offset, Some(0));
        assert!(buf.used_bytes() >= 16);

        buf.reset();
        assert_eq!(buf.used_bytes(), 0);
    }
}

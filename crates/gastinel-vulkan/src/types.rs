//! Pure Rust definitions for Vulkan 1.0/1.3 core types and structures.
//!
//! Eliminates heavy C bindings or external crates, providing exact C-ABI compatible types.

use std::os::raw::{c_char, c_void};

pub type VkFlags = u32;
pub type VkBool32 = u32;
pub type VkDeviceSize = u64;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VkResult(pub i32);

pub const VK_SUCCESS: VkResult = VkResult(0);
pub const VK_NOT_READY: VkResult = VkResult(1);
pub const VK_TIMEOUT: VkResult = VkResult(2);
pub const VK_ERROR_OUT_OF_HOST_MEMORY: VkResult = VkResult(-1);
pub const VK_ERROR_OUT_OF_DEVICE_MEMORY: VkResult = VkResult(-2);
pub const VK_ERROR_INITIALIZATION_FAILED: VkResult = VkResult(-3);
pub const VK_ERROR_DEVICE_LOST: VkResult = VkResult(-4);

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VkStructureType(pub i32);

pub const VK_STRUCTURE_TYPE_APPLICATION_INFO: VkStructureType = VkStructureType(0);
pub const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: VkStructureType = VkStructureType(1);
pub const VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO: VkStructureType = VkStructureType(2);
pub const VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO: VkStructureType = VkStructureType(3);
pub const VK_STRUCTURE_TYPE_SUBMIT_INFO: VkStructureType = VkStructureType(4);
pub const VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO: VkStructureType = VkStructureType(39);
pub const VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO: VkStructureType = VkStructureType(40);
pub const VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO: VkStructureType = VkStructureType(42);
pub const VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO: VkStructureType = VkStructureType(43);

pub const VK_TRUE: VkBool32 = 1;
pub const VK_FALSE: VkBool32 = 0;

pub const VK_NULL_HANDLE: usize = 0;

macro_rules! define_handle {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub struct $name(pub usize);

        impl $name {
            pub const NULL: Self = Self(VK_NULL_HANDLE);
            pub fn is_null(&self) -> bool {
                self.0 == VK_NULL_HANDLE
            }
        }
    };
}

define_handle!(VkInstance);
define_handle!(VkPhysicalDevice);
define_handle!(VkDevice);
define_handle!(VkQueue);
define_handle!(VkCommandBuffer);
define_handle!(VkCommandPool);
define_handle!(VkSurfaceKHR);
define_handle!(VkSwapchainKHR);
define_handle!(VkRenderPass);
define_handle!(VkFramebuffer);
define_handle!(VkFence);
define_handle!(VkSemaphore);

#[repr(C)]
pub struct VkApplicationInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub p_application_name: *const c_char,
    pub application_version: u32,
    pub p_engine_name: *const c_char,
    pub engine_version: u32,
    pub api_version: u32,
}

#[repr(C)]
pub struct VkInstanceCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: VkFlags,
    pub p_application_info: *const VkApplicationInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const c_char,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const c_char,
}

#[repr(C)]
pub struct VkPhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: u32,
    pub device_name: [c_char; 256],
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: [u8; 504], // VkPhysicalDeviceLimits padded
    pub sparse_properties: [u8; 20],
}

#[repr(C)]
pub struct VkQueueFamilyProperties {
    pub queue_flags: VkFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: [u32; 3],
}

pub const VK_QUEUE_GRAPHICS_BIT: VkFlags = 0x00000001;
pub const VK_QUEUE_COMPUTE_BIT: VkFlags = 0x00000002;
pub const VK_QUEUE_TRANSFER_BIT: VkFlags = 0x00000004;

pub const VK_MAKE_API_VERSION: fn(u32, u32, u32, u32) -> u32 = |variant, major, minor, patch| {
    (variant << 29) | (major << 22) | (minor << 12) | patch
};

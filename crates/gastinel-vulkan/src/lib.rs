//! Low-level Vulkan backend and on-demand render coordinator.

pub mod loader;
pub mod renderer;
pub mod types;

pub use loader::{PhysicalDeviceInfo, VulkanEntry, VulkanInstance};
pub use renderer::{OnDemandRenderer, RenderStats};
pub use types::*;

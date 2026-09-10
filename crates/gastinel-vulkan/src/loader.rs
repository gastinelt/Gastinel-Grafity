//! Dynamic Vulkan 1.0+ runtime loader.
#![allow(non_camel_case_types)]
//!
//! Loads Vulkan driver libraries dynamically without linking against a static SDK or C runtime:
//! - Windows: `vulkan-1.dll`
//! - Linux (Alpine / musl): `libvulkan.so.1` or `libvulkan.so`

use std::ffi::{c_void, CStr, CString};
use crate::types::*;

#[allow(non_camel_case_types)]
pub type PFN_vkGetInstanceProcAddr = unsafe extern "system" fn(
    instance: VkInstance,
    p_name: *const std::os::raw::c_char,
) -> Option<unsafe extern "system" fn()>;

pub type PFN_vkCreateInstance = unsafe extern "system" fn(
    p_create_info: *const VkInstanceCreateInfo,
    p_allocator: *const c_void,
    p_instance: *mut VkInstance,
) -> VkResult;

pub type PFN_vkDestroyInstance = unsafe extern "system" fn(
    instance: VkInstance,
    p_allocator: *const c_void,
);

pub type PFN_vkEnumeratePhysicalDevices = unsafe extern "system" fn(
    instance: VkInstance,
    p_physical_device_count: *mut u32,
    p_physical_devices: *mut VkPhysicalDevice,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceProperties = unsafe extern "system" fn(
    physical_device: VkPhysicalDevice,
    p_properties: *mut VkPhysicalDeviceProperties,
);

pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties = unsafe extern "system" fn(
    physical_device: VkPhysicalDevice,
    p_queue_family_property_count: *mut u32,
    p_queue_family_properties: *mut VkQueueFamilyProperties,
);

/// Dynamic function pointers loaded from the Vulkan runtime.
pub struct VulkanEntry {
    _lib: LibraryHandle,
    pub get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    pub create_instance: PFN_vkCreateInstance,
}

struct LibraryHandle {
    #[cfg(windows)]
    handle: *mut c_void,
    #[cfg(unix)]
    handle: *mut c_void,
}

impl LibraryHandle {
    #[cfg(windows)]
    fn load() -> Option<Self> {
        extern "system" {
            fn LoadLibraryA(lpLibFileName: *const std::os::raw::c_char) -> *mut c_void;
        }
        let name = CString::new("vulkan-1.dll").ok()?;
        let handle = unsafe { LoadLibraryA(name.as_ptr()) };
        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
        }
    }

    #[cfg(unix)]
    fn load() -> Option<Self> {
        extern "C" {
            fn dlopen(filename: *const std::os::raw::c_char, flag: std::os::raw::c_int) -> *mut c_void;
        }
        let names = ["libvulkan.so.1", "libvulkan.so"];
        for name in &names {
            if let Ok(c_name) = CString::new(*name) {
                let handle = unsafe { dlopen(c_name.as_ptr(), 1 /* RTLD_LAZY */) };
                if !handle.is_null() {
                    return Some(Self { handle });
                }
            }
        }
        None
    }

    #[cfg(windows)]
    unsafe fn get_sym(&self, name: &str) -> Option<*mut c_void> {
        extern "system" {
            fn GetProcAddress(hModule: *mut c_void, lpProcName: *const std::os::raw::c_char) -> *mut c_void;
        }
        let c_name = CString::new(name).ok()?;
        let sym = GetProcAddress(self.handle, c_name.as_ptr());
        if sym.is_null() {
            None
        } else {
            Some(sym)
        }
    }

    #[cfg(unix)]
    unsafe fn get_sym(&self, name: &str) -> Option<*mut c_void> {
        extern "C" {
            fn dlsym(handle: *mut c_void, symbol: *const std::os::raw::c_char) -> *mut c_void;
        }
        let c_name = CString::new(name).ok()?;
        let sym = dlsym(self.handle, c_name.as_ptr());
        if sym.is_null() {
            None
        } else {
            Some(sym)
        }
    }
}

impl Drop for LibraryHandle {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            extern "system" {
                fn FreeLibrary(hLibModule: *mut c_void) -> i32;
            }
            FreeLibrary(self.handle);
        }
        #[cfg(unix)]
        unsafe {
            extern "C" {
                fn dlclose(handle: *mut c_void) -> std::os::raw::c_int;
            }
            dlclose(self.handle);
        }
    }
}

impl VulkanEntry {
    /// Dynamically loads the Vulkan driver from the operating system.
    pub fn load() -> Result<Self, &'static str> {
        let lib = LibraryHandle::load().ok_or("Vulkan runtime driver not found")?;

        let get_proc = unsafe {
            let sym = lib.get_sym("vkGetInstanceProcAddr")
                .ok_or("Could not locate vkGetInstanceProcAddr")?;
            std::mem::transmute::<*mut c_void, PFN_vkGetInstanceProcAddr>(sym)
        };

        let create_instance_name = CString::new("vkCreateInstance").unwrap();
        let create_instance = unsafe {
            let sym = get_proc(VkInstance::NULL, create_instance_name.as_ptr())
                .ok_or("Failed to query vkCreateInstance")?;
            std::mem::transmute::<unsafe extern "system" fn(), PFN_vkCreateInstance>(sym)
        };

        Ok(Self {
            _lib: lib,
            get_instance_proc_addr: get_proc,
            create_instance,
        })
    }

    /// Initializes a Gastinel Vulkan Instance.
    pub fn create_instance(&self, app_name: &str) -> Result<VulkanInstance, VkResult> {
        let app_cname = CString::new(app_name).unwrap();
        let engine_cname = CString::new("GastinelEngine").unwrap();

        let app_info = VkApplicationInfo {
            s_type: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: app_cname.as_ptr(),
            application_version: VK_MAKE_API_VERSION(0, 1, 0, 0),
            p_engine_name: engine_cname.as_ptr(),
            engine_version: VK_MAKE_API_VERSION(0, 1, 0, 0),
            api_version: VK_MAKE_API_VERSION(0, 1, 3, 0),
        };

        let create_info = VkInstanceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: 0,
            p_application_info: &app_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: std::ptr::null(),
        };

        let mut instance = VkInstance::NULL;
        let res = unsafe { (self.create_instance)(&create_info, std::ptr::null(), &mut instance) };
        if res != VK_SUCCESS {
            return Err(res);
        }

        // Load instance functions
        unsafe {
            let destroy_name = CString::new("vkDestroyInstance").unwrap();
            let destroy_sym = (self.get_instance_proc_addr)(instance, destroy_name.as_ptr()).unwrap();
            let destroy_instance = std::mem::transmute::<_, PFN_vkDestroyInstance>(destroy_sym);

            let enum_name = CString::new("vkEnumeratePhysicalDevices").unwrap();
            let enum_sym = (self.get_instance_proc_addr)(instance, enum_name.as_ptr()).unwrap();
            let enumerate_physical_devices = std::mem::transmute::<_, PFN_vkEnumeratePhysicalDevices>(enum_sym);

            let props_name = CString::new("vkGetPhysicalDeviceProperties").unwrap();
            let props_sym = (self.get_instance_proc_addr)(instance, props_name.as_ptr()).unwrap();
            let get_physical_device_properties = std::mem::transmute::<_, PFN_vkGetPhysicalDeviceProperties>(props_sym);

            Ok(VulkanInstance {
                raw: instance,
                destroy_instance,
                enumerate_physical_devices,
                get_physical_device_properties,
            })
        }
    }
}

pub struct VulkanInstance {
    pub raw: VkInstance,
    destroy_instance: PFN_vkDestroyInstance,
    enumerate_physical_devices: PFN_vkEnumeratePhysicalDevices,
    get_physical_device_properties: PFN_vkGetPhysicalDeviceProperties,
}

impl VulkanInstance {
    pub fn list_physical_devices(&self) -> Vec<PhysicalDeviceInfo> {
        let mut count = 0u32;
        unsafe {
            let res = (self.enumerate_physical_devices)(self.raw, &mut count, std::ptr::null_mut());
            if res != VK_SUCCESS || count == 0 {
                return Vec::new();
            }

            let mut devices = vec![VkPhysicalDevice::NULL; count as usize];
            (self.enumerate_physical_devices)(self.raw, &mut count, devices.as_mut_ptr());

            devices
                .into_iter()
                .map(|dev| {
                    let mut props: VkPhysicalDeviceProperties = std::mem::zeroed();
                    (self.get_physical_device_properties)(dev, &mut props);

                    let name_c = CStr::from_ptr(props.device_name.as_ptr());
                    let name = name_c.to_string_lossy().into_owned();

                    PhysicalDeviceInfo {
                        handle: dev,
                        name,
                        device_type: props.device_type,
                        api_version: props.api_version,
                    }
                })
                .collect()
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                (self.destroy_instance)(self.raw, std::ptr::null());
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalDeviceInfo {
    pub handle: VkPhysicalDevice,
    pub name: String,
    pub device_type: u32,
    pub api_version: u32,
}

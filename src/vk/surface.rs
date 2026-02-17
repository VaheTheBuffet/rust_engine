use super::*;
use ash::vk::{self, Handle};

pub(super) struct Surface {
    instance: Arc<vulkan::Instance>,
    pub(super) handle: vk::SurfaceKHR
}

impl Drop for Surface {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.instance.surface.destroy_surface(self.handle, None);
        }
    }
}

impl Surface 
{
    pub fn new(window: &glfw::PWindow, instance: Arc<vulkan::Instance>) -> Surface
    {
        unsafe 
        {
            let mut surface: glfw::ffi::VkSurfaceKHR = std::ptr::null_mut();
            if window.create_window_surface(
                std::mem::transmute(instance.instance.handle()), 
                std::ptr::null(), 
                &mut surface
            ) == glfw::ffi::VkResult_VK_SUCCESS {
                Surface{instance, handle: vk::SurfaceKHR::from_raw(surface as _)}
            } else {
                panic!("failed to create surface")
            }
        }
    }
}
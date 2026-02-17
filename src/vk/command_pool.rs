use ash::vk;
use super::*;

pub(super) struct CommandPool {
    device: Arc<device::Device>,
    pub(super) handle: vk::CommandPool,
}

impl Drop for CommandPool {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.destroy_command_pool(self.handle, None);
        }
    }
}

pub(super) fn create_command_pools(
    instance: &vulkan::Instance, 
    device: Arc<device::Device>,
    physical_device: vk::PhysicalDevice, 
    surface: vk::SurfaceKHR,
) -> (CommandPool, CommandPool)
{
    let indices = physical_device::get_queue_families(instance, physical_device, surface)
        .expect("failed to get queue family indices");

    let graphics_pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(indices.graphics);
    
    let transfer_pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(indices.transfer);

    let (graphics_pool, transfer_pool) = unsafe {
        let graphics_pool = device.device.create_command_pool(&graphics_pool_info, None)
            .expect("failed to create graphics queue");
        let transfer_pool = device.device.create_command_pool(&transfer_pool_info, None)
            .expect("failed to create transfer queue");

        (graphics_pool, transfer_pool)
    };

    (
        CommandPool{device: device.clone(), handle: graphics_pool}, 
        CommandPool{device: device, handle: transfer_pool}
    )
}
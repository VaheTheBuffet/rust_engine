use ash::vk;
use super::*;

pub(super) const DEVICE_EXTENSIONS: [*const i8; 1] = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];

pub(super) struct Device {
    pub device: ash::Device,
    pub swapchain: ash::khr::swapchain::Device,
}

impl Drop for Device 
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.destroy_device(None);
        }
    }
}

impl Device 
{
    pub(super) fn new(
        instance: &vulkan::Instance, 
        device: vk::PhysicalDevice, 
        indices: physical_device::QueueFamilyIndices
    ) -> (Device, Queues) {

        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        let unique_familes = std::collections::HashSet::from(indices.iter());

        for family in unique_familes
        {
            let queue_create_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(&[1.0]);

            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures::default()
            .sampler_anisotropy(true)
            .sample_rate_shading(true);
        
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&super::device::DEVICE_EXTENSIONS);


        unsafe 
        {
            let device = instance.instance.create_device(device, &device_create_info, None)
                .expect("failed to create logical device");

            let graphics = device.get_device_queue(indices.graphics, 0);
            let present = device.get_device_queue(indices.present, 0);
            let transfer = device.get_device_queue(indices.transfer, 0);

            let families = Queues{graphics, present, transfer};

            let device_swapchain = ash::khr::swapchain::Device::new(&instance.instance, &device);
            let device = Device{device, swapchain: device_swapchain};

            (device, families)
        }
    }
}

pub(super) struct Queues {
    pub(super) graphics: vk::Queue,
    pub(super) present: vk::Queue,
    pub(super) transfer: vk::Queue,
}
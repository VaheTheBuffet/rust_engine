use ash::vk;
use super::*;


pub(super) struct ImageView {
    pub(super) handle: vk::ImageView,
    device: Arc<device::Device>,
}

impl Drop for ImageView {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.destroy_image_view(self.handle, None);
        }
    }
}

impl ImageView {
    pub(super) fn new(
        device: Arc<device::Device>,
        image: vk::Image, 
        format: vk::Format, 
        aspect_flags: vk::ImageAspectFlags, 
        mip_levels: u32,
    ) -> ImageView
    {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect_flags)
                    .base_mip_level(0)
                    .level_count(mip_levels)
                    .base_array_layer(0)
                    .layer_count(1));

        let handle = unsafe{
            device.device.create_image_view(&view_info, None)
                .expect("failed to create image view")
        };

        ImageView {handle, device}
    }
}


pub struct Image {
    device: Arc<device::Device>,
    pub(super) handle: vk::Image,
    pub(super) memory: vk::DeviceMemory
}

impl Drop for Image {
    fn drop(&mut self) 
    {
        unsafe  
        {
            self.device.device.free_memory(self.memory, None);
            self.device.device.destroy_image(self.handle, None);
        }
    }
}


impl Image {
    pub(super) fn new(
        instance: &vulkan::Instance,
        device: Arc<device::Device>,
        extent: vk::Extent3D, 
        mip_levels: u32, 
        samples: vk::SampleCountFlags, 
        format: vk::Format, 
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
        physical_device: vk::PhysicalDevice,
    ) -> Image 
    {
        let image_create_info = vk::ImageCreateInfo::default()
            .format(format)
            .array_layers(1)
            .extent(extent)
            .mip_levels(mip_levels)
            .samples(samples)
            .tiling(tiling)
            .array_layers(1)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        

        let image = unsafe{device.device.create_image(&image_create_info, None)}
            .expect("failed to create image");

        let mem_requirements = unsafe{device.device.get_image_memory_requirements(image)};

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(find_memory_type(instance, physical_device, mem_requirements.memory_type_bits, properties)
                .expect("failed to find suitable memory type"));

        let image_memory = unsafe{device.device.allocate_memory(&alloc_info, None)}
            .expect("failed to allocate image memory");

        unsafe{device.device.bind_image_memory(image, image_memory, 0)}
            .expect("failed to bind image memory");

        Image{device, handle: image, memory: image_memory}
    }
}


fn find_memory_type(
    instance: &vulkan::Instance, 
    physical_device: vk::PhysicalDevice, 
    memory_type: u32, 
    properties: vk::MemoryPropertyFlags
) -> Result<u32, ()>
{
    let mem_properties = unsafe{
        instance.instance.get_physical_device_memory_properties(physical_device)
    };

    for i in 0..mem_properties.memory_type_count 
    {
        if  memory_type & (1 << i) != 0  
            && mem_properties.memory_types[i as usize].property_flags 
                & properties 
                == properties
        {
            return Ok(i)
        }
    }

    Err(())
}

pub(super) fn find_supported_image_format(
    instance: &vulkan::Instance, 
    physical_device: vk::PhysicalDevice, 
    candidates: &[vk::Format], 
    tiling: vk::ImageTiling, 
    features: vk::FormatFeatureFlags
) -> Option<vk::Format>
{
    for &format in candidates 
    {
        let properties = unsafe{
            instance.instance.get_physical_device_format_properties(physical_device, format)
        };

        if  tiling == vk::ImageTiling::LINEAR 
            && properties.linear_tiling_features.contains(features) 
        {
            return Some(format)
        } 
        else if tiling == vk::ImageTiling::OPTIMAL 
                && properties.optimal_tiling_features.contains(features) 
        {
            return Some(format)
        }
    }

    None
}

pub(super) fn find_depth_format(
    instance: &vulkan::Instance, 
    physical_device: vk::PhysicalDevice, 
) -> vk::Format
{
    find_supported_image_format(
        instance, 
        physical_device, 
        &[vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT],
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    ).expect("failed to find depth format")
}

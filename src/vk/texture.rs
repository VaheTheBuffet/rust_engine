use crate::renderer;
use super::*;
use ash::vk;

pub(super) struct Texture {
    image: image::Image
}

impl renderer::Texture for Texture {
    fn as_any(&self) -> &dyn std::any::Any 
    {
        self
    }

    fn texture_data(&mut self, data: &[u8]) 
    {
        panic!("don't fucking dynamically allocate image")
    }
}

impl Texture {
    pub(super) fn new(api: &vulkan::VKInner, info: renderer::TextureCreateInfo<'_>) -> Texture 
    {
        let image_size = (info.height * info.width * 4) as vk::DeviceSize;
        let staging_buffer = buffer::Buffer::new(
            api, image_size,
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);


        unsafe 
        {
            let mem_ptr = api.device.device.map_memory(
                staging_buffer.memory, 0, image_size, 
                vk::MemoryMapFlags::empty()
            ).expect("failed to map image memory");

            std::ptr::copy_nonoverlapping(
                info.pixels.as_ptr(), 
                mem_ptr as *mut u8, 
                info.pixels.len());

            api.device.device.unmap_memory(staging_buffer.memory);
        }

        let image = image::Image::new(
            &api.instance,
            api.device.clone(),
            api.physical_device,
            vk::Extent3D::default()
                .depth(1)
                .width(info.width as u32)
                .height(info.height as u32), 
            1, 
            vk::SampleCountFlags::TYPE_1, 
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL);

        let temp = api.graphics_pool.create_temp_command_buffer(api.queues.graphics);
        temp.transition_image_layout(
            &image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            info.layers as _,
            0);

        let temp = api.graphics_pool.create_temp_command_buffer(api.queues.graphics);
        temp.copy_buffer_to_image(
            &staging_buffer, 
            &image, 
            info.width as u32, 
            info.height as u32, 
            info.layers as u32);

        Texture{image}
    }
}

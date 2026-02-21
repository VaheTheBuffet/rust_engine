use crate::renderer;
use super::*;
use ash::vk;

pub(super) struct Texture {
}

impl Drop for Texture {
    fn drop(&mut self) 
    {
        
    }
}

impl renderer::Texture for Texture {
    fn as_any(&self) -> &dyn std::any::Any 
    {
        self
    }

    fn texture_data(&mut self, data: &[u8]) 
    {
        
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
            api,
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
        
        image.transition_layout(

        );
        

        todo!()
    }
}
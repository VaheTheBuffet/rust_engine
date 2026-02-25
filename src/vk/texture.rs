use crate::renderer;
use super::*;
use ash::vk;

pub(super) struct Texture {
    pub(super) image: image::Image,
    pub(super) image_view: image::ImageView,
    pub(super) sampler: Sampler,
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
        let image_size = (info.height * info.width * info.layers * 4) as vk::DeviceSize;

        let mut staging_buffer = buffer::Buffer::new(
            api, image_size,
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        staging_buffer.map_memory();

        unsafe 
        {
            std::ptr::copy_nonoverlapping(
                info.pixels.as_ptr() as *const std::ffi::c_void, 
                staging_buffer.memory_mapped as *mut std::ffi::c_void, 
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
            info.layers as u32,
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
            1);

        let temp = api.graphics_pool.create_temp_command_buffer(api.queues.graphics);
        temp.copy_buffer_to_image(
            &staging_buffer, 
            &image, 
            info.width as u32, 
            info.height as u32, 
            info.layers as u32);

        temp.transition_image_layout(
            &image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            info.layers as _,
            1);

        let image_view = image::ImageView::new(
            api.device.clone(), 
            image.handle, 
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageAspectFlags::COLOR, 
            1,
            info.layers as u32);

        let sampler = Sampler::new(api);

        Texture{image, image_view, sampler}
    }
}

pub(super) struct Sampler {
    pub(super) handle: vk::Sampler,
    device: Arc<device::Device>
}

impl Drop for Sampler {
    fn drop(&mut self)
    {
        unsafe 
        {
            self.device.device.destroy_sampler(self.handle, None);
        }
    }
}

impl Sampler {
    fn new(api: &vulkan::VKInner) -> Sampler 
    {
        let properties = unsafe{api.instance.instance.get_physical_device_properties(api.physical_device)};

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .max_anisotropy(properties.limits.max_sampler_anisotropy)
            .unnormalized_coordinates(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE)
            .mip_lod_bias(0.0);

        let sampler = unsafe{api.device.device.create_sampler(&sampler_info, None)}
            .expect("failed to create sampler");

        Sampler{handle: sampler, device: api.device.clone()}
    }
}

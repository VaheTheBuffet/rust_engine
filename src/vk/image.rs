use ash::vk;
use super::*;

pub(super) fn create_image_view(
    device: &device::Device,
    image: vk::Image, 
    format: vk::Format, 
    aspect_flags: vk::ImageAspectFlags, 
    mip_levels: u32,
) -> vk::ImageView 
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

    unsafe{
        device.device.create_image_view(&view_info, None)
            .expect("failed to create image view")
    }
}

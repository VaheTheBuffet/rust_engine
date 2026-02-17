use ash::vk;
use super::*;
use std::sync::Arc;

pub(super) struct Swapchain {
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    device: Arc<super::device::Device>
}

impl Drop for Swapchain 
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            for i in 0..self.images.len() 
            {
                self.device.device.destroy_image_view(self.image_views[i], None);
            }
            self.device.swapchain.destroy_swapchain(self.swapchain, None);
        }
    }
}

pub(super) fn create(
    instance: &vulkan::Instance, 
    device: crate::Arc<device::Device>, 
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    window: &glfw::PWindow
) -> Swapchain
{
    let support_details = SwapchainSupportDetails::query_device(
        instance,
        physical_device,
        surface
    );

    let surface_format = support_details.choose_format();
    let present_mode = support_details.choose_present_mode();
    let extent = support_details.choose_extent(window);

    let image_count = std::cmp::min(
        support_details.capabilities.max_image_count, 
        support_details.capabilities.min_image_count + 1);
    
    let mut create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_format(surface_format.format)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

    let unique_queue_families: Vec<_> = std::collections::HashSet::from(
        physical_device::get_queue_families(
            instance, 
            physical_device, 
            surface
        ).unwrap().iter()
    ).into_iter().collect();

    create_info = if unique_queue_families.len() > 1 {
            create_info.image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&unique_queue_families)
    } else {
        create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    };

    create_info = create_info
        .pre_transform(support_details.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain = unsafe{device.swapchain.create_swapchain(&create_info, None)}
        .expect("failed to create swapchain");

    let images = unsafe{device.swapchain.get_swapchain_images(swapchain)}
        .expect("failed to create swapchain images");

    let image_views = create_swapchain_views(&device, surface_format.format, images.as_slice());

    Swapchain {
        swapchain, 
        format: surface_format.format, 
        images, 
        image_views, 
        device
    }
}

pub(super) fn create_swapchain_views(
    device: &device::Device, 
    format: vk::Format, 
    images: &[vk::Image]
) -> Vec<vk::ImageView>
{
    let mut image_views: Vec<vk::ImageView> = Vec::new();

    for i in 0..images.len() 
    {
        image_views.push(
            image::create_image_view(device, images[i], format, vk::ImageAspectFlags::COLOR, 1)
        );
    }

    image_views
}

pub(super) struct SwapchainSupportDetails {
    pub(super) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(super) formats: Vec<vk::SurfaceFormatKHR>,
    pub(super) present_modes: Vec<vk::PresentModeKHR>
}

impl SwapchainSupportDetails 
{
    pub(super) fn query_device(
        instance: &vulkan::Instance, 
        device: vk::PhysicalDevice, 
        surface: vk::SurfaceKHR
    ) -> SwapchainSupportDetails 
    {
        let capabilities = unsafe{instance.surface.get_physical_device_surface_capabilities(device, surface)}
            .expect("failed to query swapchain capabilities");
        let formats = unsafe{instance.surface.get_physical_device_surface_formats(device, surface)}
            .expect("failed to query swapchain formats");
        let present_modes = unsafe{instance.surface.get_physical_device_surface_present_modes(device, surface)}
            .expect("failed to query swapchain present modes");

        Self{capabilities, formats, present_modes}
    }

    pub(super) fn choose_format(&self) -> vk::SurfaceFormatKHR
    {
        for &format in &self.formats 
        {
            if format.format == vk::Format::B8G8R8A8_SRGB 
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR 
            {
                return format
            }
        }

        self.formats[0]
    }

    pub(super) fn choose_present_mode(&self) -> vk::PresentModeKHR 
    {
        for &present_mode in &self.present_modes 
        {
            if present_mode == vk::PresentModeKHR::MAILBOX {
                return present_mode
            }
        }

        self.present_modes[0]
    }

    pub(super) fn choose_extent(&self, window: &glfw::PWindow) -> vk::Extent2D 
    {
        if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            let (width, height) = window.get_framebuffer_size();
            vk::Extent2D::default()
                .width(width as u32)
                .height(height as u32)
        }
    }
}

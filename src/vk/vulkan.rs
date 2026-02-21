use ash::{self, Entry, vk};
use crate::{renderer::*, vk::image::Image};
use super::*;


pub(super) struct Instance {
    pub instance: ash::Instance,
    pub surface: ash::khr::surface::Instance,
}

impl Instance 
{
    fn new(entry: &ash::Entry, glfw: &glfw::Glfw) -> Instance
    {
        let application_info = vk::ApplicationInfo::default()
            .application_name(c"Minecraft Clone")
            .application_version(1)
            .api_version(vk::API_VERSION_1_3)
            .engine_name(c"Voxel Engine")
            .engine_version(1);

        let mut debug_create_info = debug::debug_utils_messenger_create_info();
        let mut instance_create_info = if debug::ENABLE_VALIDATION_LAYERS {
            vk::InstanceCreateInfo::default()
                .application_info(&application_info)
                .enabled_layer_names(unsafe{std::mem::transmute(debug::VALIDATION_LAYERS.as_slice())})
                .push_next(&mut debug_create_info)
        } else {
            vk::InstanceCreateInfo::default()
        };

        let extensions = VKInner::get_required_extensions(glfw);
        let extensions_raw: Vec<*const std::ffi::c_char> = extensions.iter().map(
            |name| name.as_c_str() as *const _ as _).collect();

        instance_create_info = instance_create_info.enabled_extension_names(&extensions_raw);

        let instance = unsafe {
            entry.create_instance(&instance_create_info, None)
                .expect("failed to create instance")
        };
        let surface = ash::khr::surface::Instance::new(entry, &instance);
        let swapchain = ash::khr::swapchain::Instance::new(entry, &instance);

        Instance {instance, surface}
    }
}

impl Drop for Instance
{
    fn drop(&mut self) 
    {
        unsafe
        {
            self.instance.destroy_instance(None);
        }
    }
}

pub struct VKInner {
    //device level
    pub(super)color_image: image::Image,
    pub(super)color_image_view: image::ImageView,
    pub(super)depth_image: image::Image,
    pub(super)depth_image_view: image::ImageView,
    pub(super)queues: device::Queues,
    pub(super)swapchain: swapchain::Swapchain,
    pub(super)graphics_pool: command_pool::CommandPool,
    pub(super)transfer_pool: command_pool::CommandPool,
    pub(super)physical_device: vk::PhysicalDevice,
    pub(super)device: Arc<device::Device>,
    //instance level
    pub(super)debug_utils_messenger: debug::DebugUtilsMessenger,
    pub(super)surface: surface::Surface,
    pub(super)instance: Arc<Instance>,


    //data
    pub(super) depth_format: vk::Format,
}

impl VKInner {
    pub(super) const FRAMES_IN_FLIGHT:u32 = 2;

    pub fn new(window: &glfw::PWindow, glfw: &glfw::Glfw) -> VKInner 
    {
        let entry = unsafe {
            Entry::load().expect("failed to initialize vulkan loader")
        };

        let instance = Arc::new(Instance::new(&entry, glfw));
        let debug_utils_messenger = debug::DebugUtilsMessenger::new(&entry, &instance.instance);
        let surface = surface::Surface::new(window, instance.clone());
        let physical_device = physical_device::create(&instance);
        let queue_family_indices = physical_device::get_queue_families(&instance, physical_device, surface.handle)
            .expect("failed to find adequate queue families");
        let (device, queues) = device::Device::new(&instance, physical_device, queue_family_indices);
        let device = Arc::new(device);
        let swapchain = swapchain::create(&instance, device.clone(), physical_device, surface.handle, window);
        let (graphics_pool, transfer_pool) = command_pool::create_command_pools(&instance, device.clone(), physical_device, surface.handle);

        let screen_extent = vk::Extent3D::default()
            .depth(0)
            .width(swapchain.extent.width)
            .height(swapchain.extent.height);

        let color_image = Image::new(
            &instance, 
            device.clone(), 
            screen_extent, 
            1, 
            vk::SampleCountFlags::TYPE_1, 
            swapchain.format, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL, 
            physical_device);

        let color_image_view = image::ImageView::new(
            device.clone(), 
            color_image.handle, 
            swapchain.format, 
            vk::ImageAspectFlags::COLOR, 
            1);
        
        let depth_format = image::find_depth_format(&instance, physical_device);
        let depth_image = Image::new(
            &instance, 
            device.clone(), 
            screen_extent, 
            1, 
            vk::SampleCountFlags::TYPE_1, 
            depth_format, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL, 
            physical_device);

        let depth_image_view = image::ImageView::new(
            device.clone(), 
            depth_image.handle, 
            depth_format, 
            vk::ImageAspectFlags::DEPTH, 
            1);

        VKInner { 
            color_image,
            color_image_view,
            depth_image,
            depth_image_view,
            swapchain,
            device,
            instance,
            debug_utils_messenger,
            surface,
            physical_device,
            queues,
            graphics_pool,
            transfer_pool,
            depth_format
        }
    }


    fn get_required_extensions(glfw: &glfw::Glfw) -> Vec<std::ffi::CString>
    {
        let mut extensions = glfw.get_required_instance_extensions()
            .expect("failed to get glfw extensions");

        if debug::ENABLE_VALIDATION_LAYERS
        {
            extensions.push(vk::EXT_DEBUG_UTILS_NAME.to_string_lossy().into_owned())
        }

        extensions.iter().map(|name| std::ffi::CString::new(name.as_str()).unwrap()).collect()
    }
}


impl Api for VKInner {
    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<Box<dyn Pipeline>, ()> 
    {
        let pipeline = pipeline::Pipeline::new(
            self, 
            self.color_image_view.handle, 
            self.depth_image_view.handle, 
            pipeline_info, 
            &self.swapchain);

        Ok(Box::new(pipeline))
    }

    fn create_command_buffer<'a>(&self) -> Result<Box<dyn CommandBuffer<'a> + 'a>, ()>
    {
        todo!()
    }

    fn create_buffer(&self, info: BufferCreateInfo) -> Result<Box<dyn Buffer>, ()> 
    {
        todo!()
    }

    fn create_texture(&mut self, texture_info: TextureCreateInfo) -> Result<Box<dyn Texture>, ()> 
    {
        todo!()
    }
}
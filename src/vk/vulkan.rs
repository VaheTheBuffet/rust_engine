use ash::{self, Entry, vk};
use crate::renderer::*;
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
    queues: device::Queues,
    swapchain: swapchain::Swapchain,
    graphics_pool: command_pool::CommandPool,
    transfer_pool: command_pool::CommandPool,
    physical_device: vk::PhysicalDevice,
    device: Arc<device::Device>,
    //instance level
    debug_utils_messenger: debug::DebugUtilsMessenger,
    surface: surface::Surface,
    instance: Arc<Instance>,
}

impl VKInner {
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

        VKInner { 
            swapchain,
            device,
            instance,
            debug_utils_messenger,
            surface,
            physical_device,
            queues,
            graphics_pool,
            transfer_pool,
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
        todo!()
    }

    fn create_command_buffer<'a>(&self) -> Result<Box<dyn CommandBuffer<'a> + 'a>, ()>
    {
        todo!()
    }

    fn create_buffer(&self, buffer_info: BufferMemory) -> Result<Box<dyn Buffer>, ()> 
    {
        todo!()
    }

    fn create_texture(&mut self, texture_info: TextureCreateInfo) -> Result<Box<dyn Texture>, ()> 
    {
        todo!()
    }
}
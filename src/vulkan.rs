use ash::{self, Entry, vk::{self, Handle}};
use crate::renderer::*;

pub struct VKInner {
    instance: ash::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR
}

impl VKInner {
    pub fn new(window: &glfw::PWindow, glfw: &glfw::Glfw) -> VKInner 
    {
        let entry = unsafe {
            Entry::load().expect("failed to initialize vulkan loader")
        };

        let instance = Self::create_instance(&entry, glfw);
        let debug_utils_messenger = debug::create_debug_utils_messenger(&entry, &instance);
        let surface = Self::create_surface(window, &instance);
        let physical_device = physical_device::create(&instance);
        let queue_family_indices = physical_device::get_queue_families(&entry, &instance, physical_device, surface)
            .expect("failed to find adequate queue families");
        let (device, queues) = device::create(&instance, physical_device, queue_family_indices);
        let (swapchain, swapchain_images) = swapchain::create(&entry, &instance, &device, physical_device, surface, window);
        let (graphics_queue, transfer_queue) = command::create_command_pools(&entry, &instance, &device, physical_device, surface);

        VKInner { 
            instance: instance,
            debug_utils_messenger,
            surface,
        }
    }

    fn create_instance(entry: &ash::Entry, glfw: &glfw::Glfw) -> ash::Instance
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

        unsafe {
            entry.create_instance(&instance_create_info, None)
                .expect("failed to create instance")
        }
    }

    fn create_surface(window: &glfw::PWindow, instance: &ash::Instance) -> vk::SurfaceKHR
    {
        unsafe 
        {
            let mut surface: glfw::ffi::VkSurfaceKHR = std::ptr::null_mut();
            if window.create_window_surface(
                std::mem::transmute(instance.handle()), 
                std::ptr::null(), 
                &mut surface
            ) == glfw::ffi::VkResult_VK_SUCCESS {
                vk::SurfaceKHR::from_raw(surface as _)
            } else {
                panic!("failed to create surface")
            }
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


mod device 
{
    pub(super) const DEVICE_EXTENSIONS: [*const i8; 1] = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];

    use ash::vk;

    pub(super) fn create(
        instance: &ash::Instance, 
        device: vk::PhysicalDevice, 
        indices: super::physical_device::QueueFamilyIndices
    ) -> (ash::Device, Queues) {

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
            let device = instance.create_device(device, &device_create_info, None)
                .expect("failed to create logical device");

            let graphics = device.get_device_queue(indices.graphics, 0);
            let present = device.get_device_queue(indices.present, 0);
            let transfer = device.get_device_queue(indices.transfer, 0);

            let families = Queues{graphics, present, transfer};

            (device, families)
        }
    }
    pub(super) struct Queues {
        pub(super) graphics: vk::Queue,
        pub(super) present: vk::Queue,
        pub(super) transfer: vk::Queue,
    }
}


mod physical_device 
{
    use ash::vk;

    pub(super) fn physical_device_score(properties: &vk::PhysicalDeviceProperties) -> usize
    {
        let mut score = 0;
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
        {
            score += 1000;
        }

        score
    }

    pub(super) fn create(instance: &ash::Instance) -> vk::PhysicalDevice 
    {

        use std::collections::BTreeMap;

        let mut device_scores: BTreeMap<usize, usize> = BTreeMap::new();

        let physical_devices = unsafe {
            instance.enumerate_physical_devices()
                .expect("failed to query physical devices")
        };

        unsafe 
        {
            for (n, &device) in physical_devices.iter().enumerate()
            {
                let score = physical_device_score(
                    &instance.get_physical_device_properties(device)
                );
                device_scores.insert(score, n);
            }
        }

        physical_devices[device_scores.last_entry().unwrap().remove()]
    }

    pub(super) fn get_queue_families(
        entry: &ash::Entry,
        instance: &ash::Instance, 
        device: vk::PhysicalDevice, 
        surface: vk::SurfaceKHR
    ) -> Result<QueueFamilyIndices, ()>
    {
        let mut present_queue: Option<u32> = None;
        let mut graphics_queue: Option<u32> = None;
        let mut transfer_queue: Option<u32> = None;

        let khr_instance = ash::khr::surface::Instance::new(entry, instance);

        let properties = unsafe{instance.get_physical_device_queue_family_properties(device)};

        for (i, family) in properties.iter().enumerate() 
        {
            let graphics_support = family.queue_flags & vk::QueueFlags::GRAPHICS != vk::QueueFlags::empty();
            let transfer_support = family.queue_flags & vk::QueueFlags::TRANSFER != vk::QueueFlags::empty();
            let present_support = unsafe{khr_instance.get_physical_device_surface_support(device, i as _, surface)}.unwrap();

            if transfer_support {
                transfer_queue = Some(i as u32);
            }

            if graphics_support && Some(i as u32) != transfer_queue {
                graphics_queue = Some(i as u32);
            }

            if present_support && Some(i as u32) != transfer_queue {
                transfer_queue = Some(i as u32);
            }
        }

        if graphics_queue.is_none() && transfer_queue.is_some() {
            graphics_queue = transfer_queue;
        }

        if present_queue.is_none() && transfer_queue.is_some() {
            present_queue = transfer_queue;
        }

        if let Some(gq) = graphics_queue && let Some(pq) = present_queue && let Some(tq) = transfer_queue {
            Ok(QueueFamilyIndices {graphics: gq, present: pq, transfer: tq})
        } else {
            Err(())
        }
    }
    pub(super) struct QueueFamilyIndices {
        pub(super) graphics: u32,
        pub(super) present: u32,
        pub(super) transfer: u32
    }

    impl QueueFamilyIndices 
    {
        pub(super) fn iter(&self) -> [u32; 3]
        {
            [self.graphics, self.present, self.transfer]
        }
    }

}


mod debug 
{
    use ash::vk;

    pub(super) const VALIDATION_LAYERS: [&std::ffi::CStr; 1] = [c"VK_LAYER_KHRONOS_validation"];
    #[cfg(debug_assertions)]
    pub(super) const ENABLE_VALIDATION_LAYERS: bool = true;
    #[cfg(not(debug_assertions))]
    pub(super) const ENABLE_VALIDATION_LAYERS: bool = false;

    pub(super) fn create_debug_utils_messenger(entry: &ash::Entry, instance: &ash::Instance) -> vk::DebugUtilsMessengerEXT 
    {
        let debug_utils_instance = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_create_info = debug_utils_messenger_create_info();
        unsafe {
            debug_utils_instance.create_debug_utils_messenger(&debug_create_info, None)
                .expect("failed to create debug utils messenger")
        }
    }

    pub(super) fn debug_utils_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static>
    {
        vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE 
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING 
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL 
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE 
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION)
            .user_data(crate::ptr::null_mut())
            .pfn_user_callback(Some(debug_callback))
    }

    #[allow(unused_variables)]
    pub(super) unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        p_user_data: *mut core::ffi::c_void,
    ) -> vk::Bool32 
    {
        eprintln!(
            "validation layer {}", 
            unsafe{(*p_callback_data).message_as_c_str().unwrap().to_str().unwrap()}
        );

        vk::FALSE
    }
}


mod swapchain 
{
    use ash::vk;

    use crate::vulkan::physical_device;

    pub(super) fn create(
        entry: &ash::Entry, 
        instance: &ash::Instance, 
        device: &ash::Device, 
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        window: &glfw::PWindow
    ) -> (vk::SwapchainKHR, Vec<vk::Image>)
    {
        let khr_loader = ash::khr::swapchain::Device::new(instance, device);

        let support_details = SwapchainSupportDetails::query_device(
            entry,
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
                entry, 
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

        let swapchain = unsafe{khr_loader.create_swapchain(&create_info, None)}
            .expect("failed to create swapchain");

        let images = unsafe{khr_loader.get_swapchain_images(swapchain)}
            .expect("failed to create swapchain images");

        (swapchain, images)
    }

    pub(super) struct SwapchainSupportDetails {
        pub(super) capabilities: vk::SurfaceCapabilitiesKHR,
        pub(super) formats: Vec<vk::SurfaceFormatKHR>,
        pub(super) present_modes: Vec<vk::PresentModeKHR>
    }

    impl SwapchainSupportDetails 
    {
        pub(super) fn query_device(
            entry: &ash::Entry, 
            instance: &ash::Instance, 
            device: vk::PhysicalDevice, 
            surface: vk::SurfaceKHR
        ) -> SwapchainSupportDetails 
        {
            let khr_instance = ash::khr::surface::Instance::new(entry, instance);

            let capabilities = unsafe{khr_instance.get_physical_device_surface_capabilities(device, surface)}
                .expect("failed to query swapchain capabilities");
            let formats = unsafe{khr_instance.get_physical_device_surface_formats(device, surface)}
                .expect("failed to query swapchain formats");
            let present_modes = unsafe{khr_instance.get_physical_device_surface_present_modes(device, surface)}
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
}

mod command 
{
    use ash::vk;
    use super::physical_device;

    pub(super) fn create_command_pools(
        entry: &ash::Entry, 
        instance: &ash::Instance, 
        device: &ash::Device,
        physical_device: vk::PhysicalDevice, 
        surface: vk::SurfaceKHR
    ) -> (vk::CommandPool, vk::CommandPool)
    {
        let indices = physical_device::get_queue_families(entry, instance, physical_device, surface)
            .expect("failed to get queue family indices");

        let graphics_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.graphics);
        
        let transfer_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.transfer);

        let (graphics_pool, transfer_pool) = unsafe {
            let graphics_pool = device.create_command_pool(&graphics_pool_info, None)
                .expect("failed to create graphics queue");
            let transfer_pool = device.create_command_pool(&transfer_pool_info, None)
                .expect("failed to create transfer queue");

            (graphics_pool, transfer_pool)
        };

        (graphics_pool, transfer_pool)
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
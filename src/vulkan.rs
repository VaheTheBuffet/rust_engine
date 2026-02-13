use ash::{vk, Entry, self};
use crate::renderer::*;

#[derive(Default)]
pub struct VKinner {
    instance: Option<ash::Instance>,
}

impl VKinner {
    const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KRHONOS_validation"];
    #[cfg(debug_assertions)]
    const ENABLE_VALIDATION_LAYERS: bool = true;
    #[cfg(not(debug_assertions))]
    const ENABLE_VALIDATION_LAYERS: bool = false;

    pub fn new(window: &glfw::PWindow, glfw: &glfw::Glfw) -> VKinner {
        let application_info = vk::ApplicationInfo::default()
            .application_name(c"Minecraft Clone")
            .application_version(1)
            .api_version(vk::API_VERSION_1_3)
            .engine_name(c"Voxel Engine")
            .engine_version(1);

        let mut debug_create_info = VKinner::create_debug_messenger_create_info();
        let instance_create_info = if VKinner::ENABLE_VALIDATION_LAYERS {
            vk::InstanceCreateInfo::default()
                .application_info(&application_info)
                .enabled_layer_names(unsafe{std::mem::transmute(VKinner::VALIDATION_LAYERS.as_slice())})
                .push_next(&mut debug_create_info)
        } else {
            vk::InstanceCreateInfo::default()
        };
        
        let instance = unsafe {
            let entry = Entry::load().expect("failed to initialize vulkan loader");
            entry.create_instance(&instance_create_info, Some(&vk::AllocationCallbacks::default()))
                .expect("failed to create instance")
        };

        VKinner { 
            instance: Some(instance)
        }
    }

    fn create_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static>
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
            .pfn_user_callback(Some(VKinner::debug_callback))
    }

    fn get_required_extensions(glfw: glfw::Glfw) -> Vec<String>
    {
        let mut extensions = glfw.get_required_instance_extensions()
            .expect("failed to get glfw extensions");

        if VKinner::ENABLE_VALIDATION_LAYERS
        {
            extensions.push(vk::EXT_DEBUG_UTILS_NAME.to_string_lossy().into_owned())
        }

        extensions
    }

    #[allow(unused_variables)]
    unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        p_user_data: *mut core::ffi::c_void,
    ) -> vk::Bool32 {

        eprintln!(
            "validation layer {}", 
            unsafe{(*p_callback_data).message_as_c_str().unwrap().to_str().unwrap()}
        );

        vk::FALSE
    }
}

impl Api for VKinner {
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
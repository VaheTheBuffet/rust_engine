use ash::vk;

pub(super) const VALIDATION_LAYERS: [&std::ffi::CStr; 1] = [c"VK_LAYER_KHRONOS_validation"];
#[cfg(debug_assertions)]
pub(super) const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub(super) const ENABLE_VALIDATION_LAYERS: bool = false;


pub(super) struct DebugUtilsMessenger {
    pub(super) handle: vk::DebugUtilsMessengerEXT,
    instance: ash::ext::debug_utils::Instance,
}

impl Drop for DebugUtilsMessenger{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.instance.destroy_debug_utils_messenger(self.handle, None);
        }
    }
}

impl DebugUtilsMessenger 
{
    pub(super) fn new(entry: &ash::Entry, instance: &ash::Instance) -> DebugUtilsMessenger
    {
        let debug_utils_instance = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_create_info = debug_utils_messenger_create_info();
        unsafe {
            DebugUtilsMessenger {
                handle:debug_utils_instance.create_debug_utils_messenger(&debug_create_info, None)
                    .expect("failed to create debug utils messenger"),
                instance: debug_utils_instance,
            }
        }
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
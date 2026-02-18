use super::*;
use ash::vk;
use crate::renderer;

pub(super) struct Buffer{
    pub(super) handle: vk::Buffer,
    pub(super) memory: vk::DeviceMemory,
    pub(super) memory_mapped: *const std::ffi::c_void,
    device: Arc<device::Device>,
}

impl Drop for Buffer
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.free_memory(self.memory, None);
            self.device.device.destroy_buffer(self.handle, None);
        }
    }
}

impl Buffer {
    fn new(device: Arc<device::Device>, size: vk::DeviceSize, usage: vk::BufferUsageFlags) -> Buffer {

        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe{device.device.create_buffer(&buffer_create_info, None)}
            .expect("failed to create buffer");

        let memory_requirements = unsafe{device.device.get_buffer_memory_requirements(buffer)};

        todo!()
    }
}

impl crate::renderer::Buffer for Buffer
{
    fn allocate(&self, size: i32) {
        todo!()
    }

    fn buffer_data(&self, data: &[u8]) {
        todo!()
    }

    fn buffer_sub_data(&self, data: &[u8], offset:i32) {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
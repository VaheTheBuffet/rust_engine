use super::*;
use ash::vk;
use crate::renderer;

pub(super) struct Buffer{
    pub(super) handle: vk::Buffer,
    device: Arc<device::Device>,
}

impl Drop for Buffer
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.destroy_buffer(self.handle, None);
        }
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
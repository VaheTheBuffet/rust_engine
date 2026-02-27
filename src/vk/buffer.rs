use super::*;
use ash::vk;

pub(super) struct Buffer{
    pub(super) handle: vk::Buffer,
    pub(super) memory: vk::DeviceMemory,
    pub(super) memory_mapped: *const std::ffi::c_void,
    pub(super) size: vk::DeviceSize,
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
    pub(super) fn new(
        api: &vulkan::VKInner, 
        size: vk::DeviceSize, 
        usage: vk::BufferUsageFlags, 
        properties: vk::MemoryPropertyFlags
    ) -> Buffer 
    {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe{api.device.device.create_buffer(&buffer_create_info, None)}
            .expect("failed to create buffer");

        let memory_requirements = unsafe{api.device.device.get_buffer_memory_requirements(buffer)};

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(size)
            .memory_type_index(
                image::find_memory_type(
                    &api.instance, 
                    api.physical_device, 
                    memory_requirements.memory_type_bits, 
                    properties
                ).expect("failed to find suitable memory type")
            );
        
        let buffer_memory = unsafe{api.device.device.allocate_memory(&alloc_info, None)}
            .expect("failed to allocate buffer memory");

        unsafe{api.device.device.bind_buffer_memory(buffer, buffer_memory, 0)}
            .expect("failed to bind buffer memory");


        Buffer{
            handle: buffer, 
            memory: buffer_memory, 
            memory_mapped: std::ptr::null(), 
            size,
            device: api.device.clone()
        }
    }

    pub(super) fn map_memory(&mut self) 
    {
        self.memory_mapped = unsafe{
            self.device.device.map_memory(
                self.memory, 
                0, 
                self.size, 
                vk::MemoryMapFlags::empty()
            )
        }.expect("failed to map buffer memory");
    }

    pub(super) fn device_local(
        api: &vulkan::VKInner, 
        data: &[u8],
        cmd: command_pool::TempBuffer
    ) -> Buffer 
    {
        let mut staging_buffer = Buffer::new(
            api, 
            data.len() as vk::DeviceSize, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        staging_buffer.map_memory();
        <Buffer as crate::renderer::Buffer>::buffer_sub_data(&staging_buffer, data, 0);

        let final_buffer = Buffer::new(
            api, 
            data.len() as vk::DeviceSize, 
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL);

        cmd.copy_buffer_to_buffer(&staging_buffer, &final_buffer, data.len() as vk::DeviceSize);
        cmd.submit();

        final_buffer
    }
}


impl crate::renderer::Buffer for Buffer
{
    fn allocate(&self, size: i32) {
        println!("buffer already allocated at creation time");
    }

    fn buffer_data(&self, data: &[u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.memory_mapped as _, data.len());
        }
    }

    fn buffer_sub_data(&self, data: &[u8], offset:i32) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(), 
                (self.memory_mapped as *mut u8).add(offset as _), 
                data.len()
            );
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

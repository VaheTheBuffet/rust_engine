use ash::vk;
use crate::renderer;
use super::*;

pub(super) struct CommandBuffer {
    pub(super) handle: vk::CommandBuffer,
}

impl CommandBuffer {
    fn new(device: &device::Device, command_pool: vk::CommandPool) -> CommandBuffer
    {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe{device.device.allocate_command_buffers(&alloc_info)}
            .expect("failed to allocate command buffer")[0];

        CommandBuffer {handle: command_buffer}
    }
}

impl<'a> renderer::CommandBuffer<'a> for CommandBuffer {
    fn bind_buffer(&self, buf: &dyn renderer::Buffer, source_binding: usize) 
    {

    }

    fn bind_pipeline(&mut self, pipeline: &'a dyn renderer::Pipeline) 
    {
        
    }

    fn bind_texture(&self, tex: &dyn renderer::Texture, source_binding: usize) 
    {
        
    }

    fn bind_vertex_buffer(&self, buf: &dyn renderer::Buffer) 
    {
        
    }

    fn draw(&self, start:i32, end:i32) 
    {
        
    }

    fn draw_indexed(&self, start:i32, end:i32) 
    {
        
    }

    fn submit(&self) 
    {
        
    }
}
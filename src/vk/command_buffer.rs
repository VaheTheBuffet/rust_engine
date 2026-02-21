use ash::vk;
use crate::renderer;
use super::*;

//THIS IS JUST FOR DRAWING FOR NOW
pub(super) struct CommandBuffer<'a> {
    pub(super) handle: vk::CommandBuffer,
    pub(super) descriptor_sets: Vec<vk::DescriptorSet>,
    pipeline: Option<&'a pipeline::Pipeline>,
    device: Arc<device::Device>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl<'a> CommandBuffer<'_> {
    fn new(api: &vulkan::VKInner, command_pool: vk::CommandPool) -> CommandBuffer<'a>
    {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe{api.device.device.allocate_command_buffers(&alloc_info)}
            .expect("failed to allocate command buffer")[0];

        let graphics_queue = api.queues.graphics;
        let present_queue = api.queues.present;

        CommandBuffer {
            handle: command_buffer, 
            descriptor_sets: Vec::new(), 
            pipeline: None, 
            device: api.device.clone(),
            graphics_queue,
            present_queue
        }
    }
}

impl<'a> renderer::CommandBuffer<'a> for CommandBuffer<'a> {
    fn bind_pipeline(&mut self, pipeline: &'a dyn renderer::Pipeline) 
    {
        let pipeline = pipeline.as_any().downcast_ref::<pipeline::Pipeline>()
            .expect("must bind a pipeline created by vulkan api to vulkan command buffer");

        self.pipeline = Some(pipeline);
    }

    fn bind_descriptors(&self, descriptors: &[renderer::DescriptorWriteInfo]) 
    {
        let pipeline = self.pipeline
            .expect("bind pipeline before binding descriptors");

        let layouts = vec![pipeline.descriptor_set_layout; vulkan::VKInner::FRAMES_IN_FLIGHT as usize];

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pipeline.descriptor_pool)
            .set_layouts(&layouts);
    }

    fn bind_vertex_buffer(&self, buf: &dyn renderer::Buffer) 
    {
        let buf = buf.as_any().downcast_ref::<buffer::Buffer>()
            .expect("vulkan buffer must be bound to vulkan command buffer");

        unsafe 
        {
            self.device.device.cmd_bind_vertex_buffers(self.handle, 0, &[buf.handle], &[0]);
        }
    }

    fn draw(&self, start:i32, end:i32) 
    {
        unsafe 
        {
            self.device.device.cmd_draw(self.handle, (end-start) as u32, 1, 0, 0);
        }
    }

    fn draw_indexed(&self, start:i32, end:i32) 
    {
        unsafe
        {
            self.device.device.cmd_draw_indexed(
                self.handle, (end - start) as u32,
                 1, 0, 0, 0);
        }
    }

    fn submit(&self) 
    {
        unsafe 
        {
            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&[self.handle])
                .signal_semaphores(todo!())
                .wait_semaphores(todo!());

            self.device.device.queue_submit(self.graphics_queue, &[submit_info], None);
        }
    }

    fn begin(&self)
    {
        
    }
}
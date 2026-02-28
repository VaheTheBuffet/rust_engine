use ash::vk;
use crate::renderer;
use super::*;

//THIS IS JUST FOR DRAWING FOR NOW
//TODO: should combine this with CommandPool::TempBuffer
//TODO: should add compute passes
pub(super) struct CommandBuffer<'a> {
    pub(super) handles: Vec<vk::CommandBuffer>,
    pub(super) descriptor_set: vk::DescriptorSet,

    pipeline: Option<&'a pipeline::Pipeline>,
    device: Arc<device::Device>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    cur_frame: usize,
    cur_draw: Vec<usize>,
    
    image_available: Vec<vk::Semaphore>,
    render_finished: Vec<vk::Semaphore>,
    frame_in_flight: Vec<vk::Fence>,

    dynamic_ranges: Vec<u32>,

    image_idx: usize,
    vbo: vk::Buffer,

    swapchain: vk::SwapchainKHR,
}

impl Drop for CommandBuffer<'_> {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.queue_wait_idle(self.graphics_queue)
                .expect("failed command buffer cleanup");
            self.device.device.queue_wait_idle(self.present_queue)
                .expect("failed command buffer cleanup");


            for ia in self.image_available.iter()
            {
                self.device.device.destroy_semaphore(*ia, None);
            }

            for rf in self.render_finished.iter()
            {
                self.device.device.destroy_semaphore(*rf, None);
            }

            for fif in self.frame_in_flight.iter()
            {
                self.device.device.destroy_fence(*fif, None);
            }
        }
    }
}

impl<'a> CommandBuffer<'_> {
    pub(super) fn new(api: &vulkan::VKInner, command_pool: vk::CommandPool) -> CommandBuffer<'a>
    {

        let frames_in_flight = 3;

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(frames_in_flight);

        let handles = unsafe{api.device.device.allocate_command_buffers(&alloc_info)}
            .expect("failed to allocate command buffer");

        let graphics_queue = api.queues.graphics;
        let present_queue = api.queues.present;

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let fence_create_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available: Vec<vk::Semaphore> = Vec::new();
        let mut render_finished: Vec<vk::Semaphore> = Vec::new();
        let mut frame_in_flight: Vec<vk::Fence> = Vec::new();
        for _ in 0..frames_in_flight
        {
            image_available.push(unsafe{api.device.device.create_semaphore(&semaphore_create_info, None)}
                .expect("failed to create semaphore"));

            frame_in_flight.push(unsafe{api.device.device.create_fence(&fence_create_info, None)}
            .expect("failed to create fence"));
        }

        for _ in 0..api.swapchain.images.len()
        {
            render_finished.push(unsafe{api.device.device.create_semaphore(&semaphore_create_info, None)}
                .expect("failed to create semaphore"));
        }



        CommandBuffer {
            handles, 
            descriptor_set: vk::DescriptorSet::null(), 
            pipeline: None, 
            device: api.device.clone(),
            graphics_queue,
            present_queue,
            image_idx: 0,
            cur_draw: vec![0; frames_in_flight as usize],
            cur_frame: 0,
            vbo: vk::Buffer::null(),

            dynamic_ranges: Vec::new(),

            image_available,
            render_finished,
            frame_in_flight,

            swapchain: api.swapchain.swapchain
        }
    }
}

impl<'a> renderer::CommandBuffer<'a> for CommandBuffer<'a> {
    //this only needs to run once we can think of a better architecture
    fn bind_pipeline(&mut self, pipeline: &'a dyn renderer::Pipeline) 
    {
        let pipeline = pipeline.as_any().downcast_ref::<pipeline::Pipeline>()
            .expect("must bind a pipeline created by vulkan api to vulkan command buffer");

        self.pipeline = Some(pipeline);
    }

    // this only needs to run once we can think of a better architecture
    fn bind_descriptors(&mut self, descriptors: &[renderer::DescriptorWriteInfo]) 
    {
        let pipeline = self.pipeline
            .expect("bind pipeline before binding descriptors");

        let layouts = [pipeline.descriptor_set_layout];

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pipeline.descriptor_pool)
            .set_layouts(&layouts);

        self.descriptor_set = unsafe{self.device.device.allocate_descriptor_sets(&alloc_info)}
            .expect("failed to allocate descriptor sets")[0];

        let mut descriptor_write: Vec<vk::WriteDescriptorSet> = Vec::new();
        let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = Vec::new();
        let mut image_infos: Vec<vk::DescriptorImageInfo> = Vec::new();
        self.dynamic_ranges = Vec::new();

        for descriptor in descriptors.iter()
        {
            match descriptor {
                renderer::DescriptorWriteInfo::Uniform {handle} => {
                    let buffer = handle.as_any().downcast_ref::<buffer::Buffer>()
                        .expect("must use buffer created with vulkan api in vulkan descriptor");

                    self.dynamic_ranges.push(buffer.range as u32);

                    buffer_infos.push(
                        vk::DescriptorBufferInfo::default()
                            .buffer(buffer.handle)
                            .range(buffer.range));

                }

                renderer::DescriptorWriteInfo::Texture {handle} => {
                    let texture = handle.as_any().downcast_ref::<texture::Texture>()
                        .expect("must use texture created with vulkan api in vulkan descriptor");

                    image_infos.push(
                        vk::DescriptorImageInfo::default()
                            .image_view(texture.image_view.handle)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .sampler(texture.sampler.handle)
                    );
                }
            }
        }

        let (mut buffer_idx, mut image_idx) = (0, 0);
        for (i, descriptor) in descriptors.iter().enumerate()
        {
            match descriptor {
                renderer::DescriptorWriteInfo::Uniform {handle: _} => {
                    descriptor_write.push(
                        vk::WriteDescriptorSet::default()
                            .dst_binding(i as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                            .buffer_info(&buffer_infos[buffer_idx..buffer_idx+1])
                            .dst_set(self.descriptor_set)
                    );

                    buffer_idx += 1;
                }

                renderer::DescriptorWriteInfo::Texture {handle: _} => {
                    descriptor_write.push(
                        vk::WriteDescriptorSet::default()
                            .dst_binding(i as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(&image_infos[image_idx..image_idx+1])
                            .dst_set(self.descriptor_set)
                    );

                    image_idx += 1;
                }
            }
        }

        unsafe 
        {
            self.device.device.update_descriptor_sets(descriptor_write.as_slice(), &[]);
        }
    }

    // This can run once per draw call so it belongs in the command buffer
    fn bind_vertex_buffer(&mut self, buf: &dyn renderer::Buffer) 
    {
        let buf = buf.as_any().downcast_ref::<buffer::Buffer>()
            .expect("attempted to bind non vulkan vertex buffer to vulkan command buffer");

        self.vbo = buf.handle;
    }

    fn draw(&mut self, start:i32, end:i32) 
    {
        let dynamic_offsets: Vec<u32> = self.dynamic_ranges.iter().map(
            |&r| (
                self.cur_draw[self.cur_frame] as vk::DeviceSize as u32
                * r + self.cur_frame as vk::DeviceSize as u32 * 1_000_000
            ) as u32
        ).collect();

        unsafe
        {
            self.device.device.cmd_bind_descriptor_sets(
                self.handles[self.cur_frame],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.unwrap().layout,
                0,
                std::slice::from_ref(&self.descriptor_set),
                dynamic_offsets.as_slice(),
            );

            self.device.device.cmd_bind_vertex_buffers(
                self.handles[self.cur_frame], 
                0, 
                &[self.vbo], 
                &[0]);

            self.device.device.cmd_draw(self.handles[self.cur_frame], (end-start) as u32, 1, 0, 0);
        }

        self.cur_draw[self.cur_frame] += 1;
    }

    fn draw_indexed(&mut self, start:i32, end:i32) 
    {
        unsafe 
        {
            self.device.device.cmd_draw_indexed(
                self.handles[self.cur_frame], (end - start) as u32,
                 1, 0, 0, 0);
        }
    }

    fn begin(&mut self)
    {
        self.cur_draw[self.cur_frame] = 0;
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe 
        {
            //This makes sure previous frame has been rendered but not necessarily presented
            self.device.device.wait_for_fences(&[self.frame_in_flight[self.cur_frame]], true, u64::MAX)
                .expect("failed to wait for fences");
            self.device.device.reset_fences(&[self.frame_in_flight[self.cur_frame]])
                .expect("failed to reset fences");

            let (img_idx, _) = self.device.swapchain.acquire_next_image(
                self.swapchain, 
                u64::MAX, 
                self.image_available[self.cur_frame], 
                vk::Fence::null())
                .expect("failed to get swapchain image");

            self.image_idx = img_idx as usize;

            self.device.device.reset_command_buffer(self.handles[self.cur_frame], vk::CommandBufferResetFlags::empty())
                .expect("failed to reset command buffer");
            self.device.device.begin_command_buffer(self.handles[self.cur_frame], &begin_info)
                .expect("failed to begin command buffer");
        }

        let pipeline = self.pipeline.expect("pipeline not bound before drawing");

        let clear_values = vec![
                vk::ClearValue{color: vk::ClearColorValue{float32: [0.6, 0.8, 0.99, 1.0]}},
                vk::ClearValue{depth_stencil: vk::ClearDepthStencilValue::default().depth(1.0)}
        ];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.pipeline.unwrap().render_pass.handle)
            .framebuffer(pipeline.framebuffer.handles[self.image_idx])
            .clear_values(clear_values.as_slice())
            .render_area(
                vk::Rect2D::default()
                    .offset(vk::Offset2D::default().x(0).y(0))
                    .extent(vk::Extent2D::default().width(1280).height(720)));

        unsafe 
        {
            self.device.device.cmd_begin_render_pass(
                self.handles[self.cur_frame], 
                &render_pass_info, 
                vk::SubpassContents::INLINE);

            self.device.device.cmd_bind_pipeline(
                self.handles[self.cur_frame], 
                vk::PipelineBindPoint::GRAPHICS, 
                pipeline.handle);

        }
    }

    fn submit(&mut self)
    {
        let wait_semaphores = [self.render_finished[self.image_idx]];
        let image_indices = [self.image_idx as u32];
        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe 
        {

            self.device.device.cmd_end_render_pass(self.handles[self.cur_frame]);
            self.device.device.end_command_buffer(self.handles[self.cur_frame])
                .expect("failed to end command buffer");

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(std::slice::from_ref(&self.handles[self.cur_frame]))
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .signal_semaphores(std::slice::from_ref(&self.render_finished[self.image_idx]))
                .wait_semaphores(std::slice::from_ref(&self.image_available[self.cur_frame]));

            self.device.device.queue_submit(self.graphics_queue, &[submit_info], self.frame_in_flight[self.cur_frame])
                .expect("failed to submit to queue");

            self.device.swapchain.queue_present(self.present_queue, &present_info)
                .expect("failed to present swapchain image");
        }

        self.cur_frame += 1;
        self.cur_frame %= self.frame_in_flight.len();
    }

    fn update_buffer(
        &mut self, 
        buffer: &dyn renderer::Buffer, 
        data: &[u8], 
        offset: i32
    ) {
        let buffer = buffer.as_any().downcast_ref::<buffer::Buffer>()
            .expect("attempted to use non vulkan buffer with vulkan command buffer");

        <buffer::Buffer as renderer::Buffer>::sub_data( buffer, data, 
            offset + (self.cur_draw[self.cur_frame] * buffer.range as usize + self.cur_frame*1_000_000) as i32);
    }
}

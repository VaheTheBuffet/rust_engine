use ash::vk;
use crate::renderer;
use super::*;

//THIS IS JUST FOR DRAWING FOR NOW
pub(super) struct CommandBuffer<'a> {
    pub(super) handles: Vec<vk::CommandBuffer>,
    pub(super) descriptor_sets: Vec<vk::DescriptorSet>,

    pipeline: Option<&'a pipeline::Pipeline>,
    device: Arc<device::Device>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    image_available: Vec<vk::Semaphore>,
    cur_frame: usize,
    
    render_finished: Vec<vk::Semaphore>,
    frame_in_progress: vk::Fence,

    image_idx: usize,

    swapchain: vk::SwapchainKHR,
}

impl Drop for CommandBuffer<'_> {
    fn drop(&mut self) 
    {
        unsafe 
        {
            for (ia, rf) in self.image_available.iter().zip(self.render_finished.iter())
            {
                self.device.device.destroy_semaphore(*ia, None);
                self.device.device.destroy_semaphore(*rf, None);
            }
            self.device.device.destroy_fence(self.frame_in_progress, None);
        }
    }
}

impl<'a> CommandBuffer<'_> {
    pub(super) fn new(api: &vulkan::VKInner, command_pool: vk::CommandPool) -> CommandBuffer<'a>
    {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(vulkan::VKInner::FRAMES_IN_FLIGHT);

        let handles = unsafe{api.device.device.allocate_command_buffers(&alloc_info)}
            .expect("failed to allocate command buffer");

        let graphics_queue = api.queues.graphics;
        let present_queue = api.queues.present;

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let fence_create_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available: Vec<vk::Semaphore> = Vec::new();
        let mut render_finished: Vec<vk::Semaphore> = Vec::new();
        for _ in 0..api.swapchain.images.len() 
        {
            image_available.push(unsafe{api.device.device.create_semaphore(&semaphore_create_info, None)}
                .expect("failed to create semaphore"));

            render_finished.push(unsafe{api.device.device.create_semaphore(&semaphore_create_info, None)}
                .expect("failed to create semaphore"));

        }
        let frame_in_progress = unsafe{api.device.device.create_fence(&fence_create_info, None)}
            .expect("failed to create fence");


        CommandBuffer {
            handles, 
            descriptor_sets: Vec::new(), 
            pipeline: None, 
            device: api.device.clone(),
            graphics_queue,
            present_queue,
            image_idx: 0,
            cur_frame: 0,

            image_available,
            render_finished,
            frame_in_progress,

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

        let layouts = [pipeline.descriptor_set_layout; vulkan::VKInner::FRAMES_IN_FLIGHT as usize];

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pipeline.descriptor_pool)
            .set_layouts(&layouts);

        self.descriptor_sets = unsafe{self.device.device.allocate_descriptor_sets(&alloc_info)}
            .expect("failed to allocate descriptor sets");

        for set_n in 0..vulkan::VKInner::FRAMES_IN_FLIGHT as usize
        {
            let mut descriptor_write: Vec<vk::WriteDescriptorSet> = Vec::new();

            let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = Vec::new();
            let mut image_infos: Vec<vk::DescriptorImageInfo> = Vec::new();

            for descriptor in descriptors.iter()
            {
                match descriptor {
                    renderer::DescriptorWriteInfo::Uniform {handle} => {
                        let buffer = handle.as_any().downcast_ref::<buffer::Buffer>()
                            .expect("must use buffer created with vulkan api in vulkan descriptor");

                        buffer_infos.push(
                            vk::DescriptorBufferInfo::default()
                                .buffer(buffer.handle)
                                .range(buffer.size));

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
                                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                                .buffer_info(&buffer_infos[buffer_idx..buffer_idx+1])
                                .dst_set(self.descriptor_sets[set_n])
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
                                .dst_set(self.descriptor_sets[set_n])
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
    }

    // This can run once per draw call so it belongs in the command buffer
    fn bind_vertex_buffer(&mut self, buf: &dyn renderer::Buffer) 
    {
        let buf = buf.as_any().downcast_ref::<buffer::Buffer>()
            .expect("vulkan buffer must be bound to vulkan command buffer");

        unsafe 
        {
            self.device.device.cmd_bind_vertex_buffers(
                self.handles[0], 
                0, 
                &[buf.handle], 
                &[0]);
        }
    }

    fn draw(&self, start:i32, end:i32) 
    {

        unsafe 
        {
            self.device.device.cmd_draw(self.handles[0], (end-start) as u32, 1, 0, 0);
        }
    }

    fn draw_indexed(&self, start:i32, end:i32) 
    {
        unsafe 
        {
            self.device.device.cmd_draw_indexed(
                self.handles[0], (end - start) as u32,
                 1, 0, 0, 0);
        }
    }

    fn submit(&mut self) 
    {
        unsafe 
        {
            self.device.device.cmd_end_render_pass(self.handles[0]);
            self.device.device.end_command_buffer(self.handles[0])
                .expect("failed to end command buffer");

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&self.handles[0..1])
                .signal_semaphores(std::slice::from_ref(&self.render_finished[self.image_idx]))
                .wait_semaphores(std::slice::from_ref(&self.image_available[0]))
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]);

            self.device.device.queue_submit(self.graphics_queue, &[submit_info], self.frame_in_progress)
                .expect("failed to submit to queue");

            let image_idx = self.image_idx as u32;
            let present_info = vk::PresentInfoKHR::default()
                .swapchains(std::slice::from_ref(&self.swapchain))
                .wait_semaphores(std::slice::from_ref(&self.render_finished[self.image_idx]))
                .image_indices(std::slice::from_ref(&image_idx));

            self.device.swapchain.queue_present(self.present_queue, &present_info)
                .expect("failed to present swapchain image");

            self.cur_frame = self.image_idx;
        }
    }

    fn begin(&mut self)
    {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe 
        {
            //This makes sure previous frame has been rendered but not necessarily presented
            self.device.device.wait_for_fences(&[self.frame_in_progress], true, u64::MAX)
                .expect("failed to wait for fences");
            self.device.device.reset_fences(&[self.frame_in_progress])
                .expect("failed to reset fences");

            //This will mark the next available image and signal the sempahore once it becomes
            //usable
            let (img_idx, _) = self.device.swapchain.acquire_next_image(
                self.swapchain, 
                u64::MAX, 
                self.image_available[0], 
                vk::Fence::null()
            ).expect("failed to get swapchain image");
            self.image_idx = img_idx as usize;

            self.device.device.reset_command_buffer(self.handles[0], vk::CommandBufferResetFlags::empty())
                .expect("failed to reset command buffer");
            self.device.device.begin_command_buffer(self.handles[0], &begin_info)
                .expect("failed to begin command buffer");
        }


        //Per batch-draw command buffer commands
        let pipeline = self.pipeline.expect("pipeline not bound before drawing");

        let clear_values = [
            vk::ClearValue{color: vk::ClearColorValue{float32: [0.0, 0.0, 0.5, 1.0]}},
            vk::ClearValue{depth_stencil: vk::ClearDepthStencilValue::default().depth(1.0).stencil(0)}
        ];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(pipeline.render_pass.handle)
            .framebuffer(pipeline.framebuffer.handles[self.image_idx])
            .render_area(
                vk::Rect2D::default()
                    .offset(vk::Offset2D::default().x(0).y(0))
                    .extent(vk::Extent2D::default().width(1280).height(720)))
            .clear_values(&clear_values);

        unsafe 
        {
            self.device.device.cmd_begin_render_pass(
                self.handles[0], 
                &render_pass_info, 
                vk::SubpassContents::INLINE);

            self.device.device.cmd_bind_pipeline(
                self.handles[0], 
                vk::PipelineBindPoint::GRAPHICS, 
                pipeline.handle);

            self.device.device.cmd_bind_descriptor_sets(
                self.handles[0],
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &self.descriptor_sets[0..1],
                &[]
            );
        }
    }
}

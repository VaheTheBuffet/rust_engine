use ash::vk;
use super::*;

pub(super) struct CommandPool {
    device: Arc<device::Device>,
    pub(super) handle: vk::CommandPool,
}

impl Drop for CommandPool {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.destroy_command_pool(self.handle, None);
        }
    }
}

impl CommandPool{
    pub(super) fn create_temp_command_buffer(&self, queue: vk::Queue) -> TempBuffer {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.handle)
            .command_buffer_count(1);

        let buffers = unsafe{self.device.device.allocate_command_buffers(&alloc_info)}
            .expect("failed to allocate command buffers");

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe{self.device.device.begin_command_buffer(buffers[0], &begin_info)}
            .expect("failed to start command buffer");

        TempBuffer{handle: buffers[0], queue, pool: self.handle, device: self.device.clone()}
    }
}

pub(super) fn create_command_pools(
    instance: &vulkan::Instance, 
    device: Arc<device::Device>,
    physical_device: vk::PhysicalDevice, 
    surface: vk::SurfaceKHR,
) -> (CommandPool, CommandPool)
{
    let indices = physical_device::get_queue_families(instance, physical_device, surface)
        .expect("failed to get queue family indices");

    let graphics_pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(indices.graphics);
    
    let transfer_pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(indices.transfer);

    let (graphics_pool, transfer_pool) = unsafe {
        let graphics_pool = device.device.create_command_pool(&graphics_pool_info, None)
            .expect("failed to create graphics queue");
        let transfer_pool = device.device.create_command_pool(&transfer_pool_info, None)
            .expect("failed to create transfer queue");

        (graphics_pool, transfer_pool)
    };

    (
        CommandPool{device: device.clone(), handle: graphics_pool}, 
        CommandPool{device: device, handle: transfer_pool}
    )
}

pub(super) struct TempBuffer {
    handle: vk::CommandBuffer,
    queue: vk::Queue,
    pool: vk::CommandPool,
    device: Arc<device::Device>
        
}

impl Drop for TempBuffer {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.free_command_buffers(
                self.pool, 
                std::slice::from_ref(&self.handle));
        }
    }
}

impl TempBuffer {
    pub(super) fn transition_image_layout(
        &self,
        image: &image::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        layers: u32,
        mip_levels: u32,
    ) {
        let mut barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image.handle)
            .src_access_mask(vk::AccessFlags::NONE)
            .dst_access_mask(vk::AccessFlags::NONE)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .base_mip_level(0)
                    .level_count(mip_levels)
                    .base_array_layer(0)
                    .layer_count(layers));

        let src_stage;
        let dst_stage;
        
        if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL 
        {
            barrier.subresource_range.aspect_mask |= vk::ImageAspectFlags::DEPTH;
        } 
        else 
        {
            barrier.subresource_range.aspect_mask |= vk::ImageAspectFlags::COLOR;
        }

        if  old_layout == vk::ImageLayout::UNDEFINED 
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            barrier.src_access_mask = vk::AccessFlags::NONE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

            src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            dst_stage = vk::PipelineStageFlags::TRANSFER;
        }
        else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL 
                && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            src_stage = vk::PipelineStageFlags::TRANSFER;
            dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        }
        else if old_layout == vk::ImageLayout::UNDEFINED 
                && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL 
        {
            barrier.src_access_mask = vk::AccessFlags::NONE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

            src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            dst_stage = vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
        }
        else 
        {
            panic!("unsupported layout transition");
        }

        unsafe 
        {
            self.device.device.cmd_pipeline_barrier(
                self.handle,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        self.submit();
    }

    pub(super) fn copy_buffer_to_image(
        &self, 
        buffer: &buffer::Buffer, 
        image: &image::Image, 
        width: u32, 
        height: u32, 
        layers: u32
    )
    {
        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(layers))
            .image_offset(vk::Offset3D::default())
            .image_extent(
                vk::Extent3D::default()
                    .depth(1)
                    .width(width)
                    .height(height));

            unsafe 
            {
                self.device.device.cmd_copy_buffer_to_image(
                    self.handle,
                    buffer.handle,
                    image.handle,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(&region)
                );
            }
    }

    pub(super) fn copy_buffer_to_buffer(
        &self, 
        src: &buffer::Buffer, 
        dst: &buffer::Buffer, 
        size: vk::DeviceSize
    ) 
    {
        let copy_region = vk::BufferCopy::default()
            .size(size);

        unsafe 
        {
            self.device.device.cmd_copy_buffer(
                self.handle, 
                src.handle, 
                dst.handle, 
                std::slice::from_ref(&copy_region));
        }
    }

    pub(super) fn submit(&self)
    {
        let submit_info = vk::SubmitInfo::default()
            .command_buffers(std::slice::from_ref(&self.handle));

        unsafe 
        {
            self.device.device.end_command_buffer(self.handle)
                .expect("failed to end command buffer"); 
            self.device.device.queue_submit(self.queue, std::slice::from_ref(&submit_info), vk::Fence::null())
                .expect("failed to submit to queue");
            self.device.device.queue_wait_idle(self.queue)
                .expect("failed to wait for queue operation");
        }
    }
}

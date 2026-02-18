use super::*;
use crate::renderer;
use ash::vk;

pub(super) struct Pipeline {
    pub(super) framebuffer: Framebuffer,
    pub(super) render_pass: RenderPass,
    pub(super) handle: vk::Pipeline,
    device: Arc<device::Device>
}

impl Drop for Pipeline {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.destroy_pipeline(self.handle, None);
        }     
    }
}

impl Pipeline {
    pub(super) fn new(
        device: Arc<device::Device>, 
        color_image_view: vk::ImageView,
        depth_image_view: vk::ImageView,
        info: renderer::PipelineInfo, 
        swapchain: &swapchain::Swapchain
    ) -> Pipeline 
    {
        let renderer::ShaderInfo::SpirV(vert, frag) = info.shader_info else {
            panic!("spir-v shaders required for vulkan")
        };

        let vert_module = create_shader_module(&device, vert);
        let frag_module = create_shader_module(&device, frag);

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_module)
            .name(c"main");

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(c"main");

        let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

        let vertex_binding_description = vk::VertexInputBindingDescription::default()
            .binding(info.vbo_layout.bind_point as u32)
            .stride(info.vbo_layout.size(None) as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let mut vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::new();

        for (i, attribute) in info.vbo_layout.elements.iter().enumerate()
        {
            let format = match (&attribute.element_type, attribute.quantity) {
                (renderer::BufferElementType::F32, 1) => vk::Format::R32_SFLOAT,
                (renderer::BufferElementType::F32, 2) => vk::Format::R32G32_SFLOAT,
                (renderer::BufferElementType::F32, 3) => vk::Format::R32G32B32_SFLOAT,
                (renderer::BufferElementType::I32, 1) => vk::Format::R32_SINT,
                (renderer::BufferElementType::I32, 2) => vk::Format::R32G32_SINT,
                (renderer::BufferElementType::I32, 3) => vk::Format::R32G32B32_SINT,
                (renderer::BufferElementType::U32, 1) => vk::Format::R32_UINT,
                (renderer::BufferElementType::U32, 2) => vk::Format::R32G32_UINT,
                (renderer::BufferElementType::U32, 3) => vk::Format::R32G32B32_UINT,
                _ => panic!("vertex attribute format not supported {:?}, {:?}", attribute.element_type, attribute.quantity)
            };

            vertex_attribute_descriptions.push(
                vk::VertexInputAttributeDescription::default()
                    .binding(info.vbo_layout.bind_point as u32)
                    .format(format)
                    .location(i as u32)
                    .offset(info.vbo_layout.size(Some(i)) as u32));
        }

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding_description))
            .vertex_attribute_descriptions(vertex_attribute_descriptions.as_slice());

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(swapchain.extent);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default();

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::A | vk::ColorComponentFlags::R 
                | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B)
            .blend_enable(true)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_attachment = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .min_depth_bounds(0.0)
                .max_depth_bounds(1.0);

        let mut layout_bindings: Vec<vk::DescriptorSetLayoutBinding> = Vec::new();
        for descriptor in info.descriptor_layouts {
            layout_bindings.push(
                match descriptor {
                    renderer::DescriptorInfo::Vertex {stride: _, bind_point} => {
                        vk::DescriptorSetLayoutBinding::default()
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .descriptor_count(1)
                            .stage_flags(vk::ShaderStageFlags::VERTEX)
                            .binding(bind_point as u32)
                    }
                    renderer::DescriptorInfo::Texture {bind_point} => {
                        vk::DescriptorSetLayoutBinding::default()
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .descriptor_count(1)
                            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                            .binding(bind_point as u32)
                    }
                    _ => panic!("unsupported descriptor")
                });
        }

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(layout_bindings.as_slice());

        let descriptor_set_layout = unsafe{device.device.create_descriptor_set_layout(&layout_info, None)}
            .expect("failed to create descriptor set layout");

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let pipeline_layout = unsafe{device.device.create_pipeline_layout(&pipeline_layout_create_info, None)}
            .expect("failed to create pipeline layout");

        let render_pass = RenderPass::new(device.clone(), swapchain.format, swapchain.format);

        let framebuffer = Framebuffer::new(device.clone(), swapchain, color_image_view, depth_image_view, &render_pass);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(shader_stages.as_slice())
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend_attachment)
            //.dynamic_state(dynamic_state) //WE SHOULD CONSIDER USING DYNAMIC STATE FOR VIEWPORT AND SCISSOR
            .layout(pipeline_layout)
            .render_pass(render_pass.handle)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1);

        let pipeline = unsafe{
            device.device.create_graphics_pipelines(
                vk::PipelineCache::null(), 
                std::slice::from_ref(&pipeline_create_info), 
                None)
        }.expect("failed to create graphics pipeline")[0];

        Pipeline {handle: pipeline, device, render_pass, framebuffer}
    }
}

impl renderer::Pipeline for Pipeline {
    fn as_any(&self) -> &dyn std::any::Any 
    {
        self
    }
}

fn create_shader_module(device: &device::Device, code: &[u8]) -> vk::ShaderModule
{
    let code = unsafe{std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4)};
    let create_info = vk::ShaderModuleCreateInfo::default()
        .code(code);

    unsafe{device.device.create_shader_module(&create_info, None)}
        .expect("failed to create shader module")
}


pub struct RenderPass {
    handle: vk::RenderPass,
    device: Arc<device::Device>,
}

impl Drop for RenderPass {
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.device.device.destroy_render_pass(self.handle, None);
        }
    }
}

impl RenderPass {
    pub(super) fn new(
        device: Arc<device::Device>, 
        swapchain_image_format: vk::Format, 
        depth_format: vk::Format
    ) -> RenderPass
    {
        let color_attachment = vk::AttachmentDescription::default()
            .format(swapchain_image_format)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let color_attachment_resolve = vk::AttachmentDescription::default()
            .format(swapchain_image_format)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_format)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachment_resolve_ref = vk::AttachmentReference::default()
            .attachment(2)
            .layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let subpass = vk::SubpassDescription::default()
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref)
            .resolve_attachments(std::slice::from_ref(&color_attachment_resolve_ref));

        let attachments = [color_attachment, depth_attachment, color_attachment_resolve];

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(attachments.as_slice())
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe {device.device.create_render_pass(&create_info, None)}
            .expect("failed to create render pass");

        RenderPass {handle: render_pass, device}
    }
}

pub(super) struct Framebuffer {
    device: Arc<device::Device>,
    handles: Vec<vk::Framebuffer>
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe 
        {
            for &framebuffer in &self.handles 
            {
                self.device.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }
}

impl Framebuffer
{
    fn new(
        device: Arc<device::Device>, 
        swapchain: &swapchain::Swapchain, 
        color_image_view: vk::ImageView,
        depth_image_view: vk::ImageView,
        render_pass: &RenderPass
    ) -> Framebuffer 
    {
        let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();

        for i in 0..swapchain.image_views.len() 
        {
            let attachments = [color_image_view, depth_image_view, swapchain.image_views[i]];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .attachments(attachments.as_slice())
                .render_pass(render_pass.handle)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);


            framebuffers.push(
                unsafe{device.device.create_framebuffer(&framebuffer_info, None)}
                    .expect("failed to create framebuffer"));
        }

        Framebuffer {device, handles: framebuffers}
    }
}
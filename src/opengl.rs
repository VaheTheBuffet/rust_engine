use std::any::Any;

use glow::HasContext;
use crate::*;
use crate::renderer::*;

pub struct GLinner 
{
    gl: Arc<glow::Context>,
}

impl GLinner 
{
    pub fn new(window: &mut glfw::PWindow) -> GLinner 
    {
        unsafe {
            let gl = glow::Context::from_loader_function(|s| 
                match window.get_proc_address(s) 
                {
                    Some(ptr) => {ptr as _ } 
                    None => {unloaded_function as _}
                }
            );

            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::CULL_FACE);
            gl.enable(glow::BLEND);

            GLinner{gl: Arc::new(gl)}
        }
    }
}

impl Api for GLinner 
{
    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<Box<dyn Pipeline>, ()> 
    {
        let mut pipeline = GLPipeline::new(self.gl.clone());
        pipeline.add_shader_program(pipeline_info.shader_info);
        pipeline.add_vertex_description(pipeline_info.vbo_layout);
        for descriptor_layout in pipeline_info.descriptor_layouts 
        {
            pipeline.add_descriptor(descriptor_layout);
        }
        Ok(Box::new(pipeline))
    }

    fn create_command_buffer<'a>(&self) -> Result<Box<dyn CommandBuffer<'a> + 'a>, ()>
    {
        Ok(Box::new(GLCommandBuffer::new(self.gl.clone())))
    }

    fn create_buffer(&self, buffer_memory: BufferMemory) -> Result<Box<dyn Buffer>, ()> 
    {
        Ok(Box::new(GLBuffer::new(self.gl.clone(), buffer_memory)))
    }

    fn create_texture(&mut self, texture_info: TextureCreateInfo) -> Result<Box<dyn Texture>, ()> 
    {
        let res = Ok(Box::new(GLTexture::new(self.gl.clone(), texture_info)) as _);
        res
    }
}


pub struct GLTexture{
    gl: Arc<glow::Context>,
    tex: glow::NativeTexture,
    info: TextureCreateInfo
}

impl GLTexture 
{
    fn new(gl: Arc<glow::Context>, info: TextureCreateInfo) -> GLTexture 
    {
        unsafe 
        {
            let tex = if info.layers == 1 
            {
                let tex = gl.create_named_texture(glow::TEXTURE_2D).expect("failed to create texture");
                gl.texture_storage_2d(tex, 1, glow::RGBA8, info.width, info.height);
                tex

            }
            else if info.layers > 1
            {
                let tex = gl.create_named_texture(glow::TEXTURE_2D_ARRAY).expect("failed to create texture array");
                gl.bind_texture_unit(0, Some(tex));
                gl.texture_storage_3d(tex, 1, glow::RGBA8, info.width, info.height, info.layers);
                tex
            }
            else
            { 
                panic!("need at least 1 layer for image")
            };

            gl.texture_parameter_i32(tex, glow::TEXTURE_WRAP_S, glow::MIRRORED_REPEAT as i32);
            gl.texture_parameter_i32(tex, glow::TEXTURE_WRAP_T, glow::MIRRORED_REPEAT as i32);
            gl.texture_parameter_i32(tex, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.texture_parameter_i32(tex, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);

            GLTexture{gl, tex, info}
        }
    }
}


impl Texture for GLTexture 
{
    fn texture_data(&mut self, data: &[u8]) {
        unsafe
        {
            if self.info.layers == 1 
            {
                self.gl.texture_sub_image_2d(
                    self.tex, 
                    0, 
                    0, 
                    0, 
                    self.info.width, 
                    self.info.height as _, 
                    glow::RGBA, 
                    glow::UNSIGNED_BYTE, 
                    glow::PixelUnpackData::Slice(Some(data))
                );
            }
            else if self.info.layers > 1 
            {
                self.gl.texture_sub_image_3d(
                    self.tex, 
                    0, 
                    0, 
                    0, 
                    0, 
                    self.info.width, 
                    self.info.height, 
                    self.info.layers, 
                    glow::RGBA, 
                    glow::UNSIGNED_BYTE, 
                    glow::PixelUnpackData::Slice(Some(data))
                );
            }
        }
    }

    fn as_any(&self) -> &dyn Any 
    {
        self
    }
}


pub struct GLBuffer{
    gl: Arc<glow::Context>,
    buf: glow::NativeBuffer,
    memory: BufferMemory
}

impl GLBuffer
{
    fn new(gl: Arc<glow::Context>, memory: BufferMemory) -> GLBuffer
    {
        let buf = unsafe {
            gl.create_buffer().expect("failed to create buffer")
        };

        GLBuffer{gl, buf, memory}
    }
}

impl Drop for GLBuffer
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.gl.delete_buffer(self.buf);
        }
    }
}


impl Buffer for GLBuffer 
{
    fn buffer_data(&self, data: &[u8]) 
    {
        unsafe 
        {
            match self.memory 
            {
                BufferMemory::Dynamic =>
                {
                    self.gl.named_buffer_data_u8_slice(self.buf, data, glow::DYNAMIC_DRAW);
                }

                BufferMemory::ReadOnly =>
                {
                    self.gl.named_buffer_data_u8_slice(self.buf, data, glow::STATIC_DRAW);
                }
            }
        }
    }

    fn allocate(&self, size: i32) 
    {
        unsafe 
        {
            match self.memory 
            {
                BufferMemory::Dynamic =>
                {
                    self.gl.named_buffer_data_size(self.buf, size, glow::DYNAMIC_DRAW);
                }

                BufferMemory::ReadOnly =>
                {
                    self.gl.named_buffer_data_size(self.buf, size, glow::STATIC_DRAW);
                }
            }
        }
    }

    fn buffer_sub_data(&self, data: &[u8], offset: i32) 
    {
        unsafe 
        {
            self.gl.named_buffer_sub_data_u8_slice(self.buf, offset, data);
        }
    }

    fn as_any(&self) -> &dyn Any 
    {
        self
    }
}


struct GLPipeline{
    gl: Arc<glow::Context>,
    vao: glow::NativeVertexArray,
    program: glow::NativeProgram,
    descriptors: Vec<DescriptorInfo>,
    vertex_descriptor: DescriptorInfo
}

impl GLPipeline 
{
    pub fn new(gl: Arc<glow::Context>) -> GLPipeline
    {
        unsafe 
        {
            let vao = gl.create_vertex_array().expect("failed to create vertex array");
            let program = gl.create_program().expect("failed to create program");
            GLPipeline{
                gl,
                vao,
                program,
                descriptors: Vec::new(),
                vertex_descriptor: DescriptorInfo::default()
            }
        }
    }
}

impl Drop for GLPipeline 
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

impl GLPipeline 
{
    fn add_vertex_description(&mut self, vbo_layout: VertexLayout) 
    {
        self.vertex_descriptor = DescriptorInfo::Vertex {
            stride: vbo_layout.size(None) as u8, 
            bind_point: 0
        };

        unsafe 
        {
            self.gl.bind_vertex_array(Some(self.vao));
            for (i, element) in vbo_layout.elements.iter().enumerate()
            {
                let gltype = match element.element_type{
                    BufferElementType::I32 => glow::INT,
                    BufferElementType::U32 => glow::UNSIGNED_INT,
                    BufferElementType::F32 => glow::FLOAT
                };
                
                if element.element_type.is_integral()
                {
                    self.gl.vertex_array_attrib_format_i32(
                        self.vao,
                        i as u32, 
                        element.quantity as i32, 
                        gltype, 
                        vbo_layout.size(Some(i)) as u32
                    );
                }
                else
                {
                    self.gl.vertex_array_attrib_format_f32(
                        self.vao,
                        i as u32, 
                        element.quantity as i32, 
                        gltype, 
                        false, 
                        vbo_layout.size(Some(i)) as u32
                    );
                }
                self.gl.enable_vertex_array_attrib(self.vao, i as u32);
                self.gl.vertex_array_attrib_binding_f32(self.vao, i as u32, vbo_layout.bind_point as u32);
            }
        }
    }

    
    fn add_shader_program(&mut self, shader_info: ShaderInfo) 
    {
        if let ShaderInfo::Text(vert, frag) = shader_info 
        {
            unsafe 
            {
                let vs = self.gl.create_shader(glow::VERTEX_SHADER).expect("failed to create shader");
                self.gl.shader_source(vs, vert);
                self.gl.compile_shader(vs);

                let fs = self.gl.create_shader(glow::FRAGMENT_SHADER).expect("failed to create shader");
                self.gl.shader_source(fs, frag);
                self.gl.compile_shader(fs);

                if !self.gl.get_shader_compile_status(vs) {
                    panic!("VERTEX SHADER FAILED\n{}", self.gl.get_shader_info_log(vs));
                }

                if !self.gl.get_shader_compile_status(fs) {
                    panic!("FRAGMENT SHADER FAILED\n{}", self.gl.get_shader_info_log(fs));
                }

                self.gl.attach_shader(self.program, vs);
                self.gl.attach_shader(self.program, fs);
                self.gl.link_program(self.program);
                self.gl.delete_shader(vs);
                self.gl.delete_shader(fs);
            }
        }
        else
        { 
            panic!("spir-v shaders not yet supported for opengl");
        }
    }
    

    fn add_descriptor(&mut self, descriptor_layout: DescriptorInfo) 
    {
        unsafe 
        {
            match descriptor_layout 
            {
                DescriptorInfo::Uniform{bind_point, size: _} => 
                {
                    self.gl.uniform_block_binding(self.program, bind_point as _, bind_point as _);
                }
                DescriptorInfo::Texture{bind_point} => 
                {
                    self.gl.uniform_block_binding(self.program, bind_point as _, bind_point as _);
                }
                _ => {}
            }
        }
        self.descriptors.push(descriptor_layout);
    }
}


impl Pipeline for GLPipeline 
{
    fn as_any(&self) -> &dyn Any 
    {
        self
    }
}


struct GLCommandBuffer<'a> {
    gl: Arc<glow::Context>,
    pipeline: Option<&'a GLPipeline>
}

impl<'a> GLCommandBuffer<'_> 
{
    fn new(gl: Arc<glow::Context>) -> GLCommandBuffer<'a>
    {
        GLCommandBuffer{gl, pipeline: None}
    }
}

impl<'a> CommandBuffer<'a> for GLCommandBuffer<'a>
{
    fn bind_pipeline(&mut self, pipeline: &'a dyn Pipeline) 
    {
        let pipeline = pipeline.as_any().downcast_ref::<GLPipeline>()
            .expect("wrong type of pipeline for api");
        
        unsafe 
        {
            self.gl.bind_vertex_array(Some(pipeline.vao));
            self.gl.use_program(Some(pipeline.program));
        }

        self.pipeline = Some(pipeline);
    }

    
    fn bind_vertex_buffer(&self, buf: &dyn Buffer)
    {
        let pipeline = self.pipeline.expect("bind pipeline before binding buffer");

        let buffer = buf.as_any().downcast_ref::<GLBuffer>()
            .expect("wrong type of buffer for api");

        let DescriptorInfo::Vertex{stride, bind_point} = pipeline.vertex_descriptor else {
            panic!("vertex layout not set");
        };

        unsafe 
        {
            self.gl.bind_vertex_buffer(bind_point as u32, Some(buffer.buf), 0, stride as i32);
        }
    }


    fn bind_buffer(&self, buf: &dyn Buffer, source_binding: usize)
    {
        let buffer = buf.as_any().downcast_ref::<GLBuffer>()
            .expect("wrong type of buffer for api");

        let pipeline = self.pipeline.expect("bind pipeline before binding buffer");


        let DescriptorInfo::Uniform{bind_point, size} = pipeline.descriptors[source_binding] else {
            panic!("attempted to bind {:?} to Uniform Buffer Descriptor", pipeline.descriptors[source_binding]);
        };

        unsafe 
        {
            self.gl.bind_buffer_range(glow::UNIFORM_BUFFER, bind_point as _, Some(buffer.buf), 0, size as i32);
        }
    }


    fn bind_texture(&self, tex: &dyn Texture, source_binding: usize) 
    {
        let texture = tex.as_any().downcast_ref::<GLTexture>()
            .expect("could not read provided texture");

        let pipeline = self.pipeline.expect("bind pipeline before binding image");

        let DescriptorInfo::Texture{bind_point} = pipeline.descriptors[source_binding] else {
            panic!("attempted to bind {:?} to Texture Descriptor", pipeline.descriptors[source_binding]);
        };

        unsafe 
        {
            self.gl.bind_texture_unit(bind_point as _, Some(texture.tex));
        }
    }


    fn draw(&self, start:i32, end:i32) 
    {
        unsafe 
        {
            self.gl.draw_arrays(glow::TRIANGLES, start, end);
        }
    }


    fn draw_indexed(&self, start:i32, end:i32) 
    {
        unsafe 
        {
            self.gl.draw_elements(glow::TRIANGLES, end - start + 1, glow::UNSIGNED_INT, 0);
        }
    }


    fn submit(&self) 
    {
        unsafe 
        {
            self.gl.clear_color(0.6, 0.8, 0.99, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }
}


unsafe extern "system" fn unloaded_function() 
{
    panic!("OpenGL Function Not Loaded!");
}
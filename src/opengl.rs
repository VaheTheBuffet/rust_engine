use glow::HasContext;
use crate::*;
use crate::renderer::*;

pub struct GLinner 
{
    gl: Arc<glow::Context>
}

impl GLinner 
{
    pub fn new(mut window: glfw::PWindow) -> GLinner 
    {
        unsafe {
            let gl = glow::Context::from_loader_function(|s| 
                match window.get_proc_address(s) 
                {
                    Some(ptr) => {ptr as _ } 
                    None => {std::ptr::null() as _}
                }
            );

            GLinner{gl: Arc::new(gl)}
        }
    }
}

impl Api for GLinner 
{
    fn create_buffer(&self, buffer_info: BufferUsage) -> Result<Box<dyn Buffer>, ()> 
    {
        Ok(Box::new(GLBuffer::new(self.gl.clone(), buffer_info)))
    }

    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<Box<dyn Pipeline>, ()> 
    {
        let pipeline = GLPipeline::new(self.gl.clone());
        pipeline.add_shader_program(pipeline_info.shader_info);
        pipeline.add_vertex_description(pipeline_info.vbo_layout);
        Ok(Box::new(pipeline))
    }

    fn draw(&self, start: i32, end: i32) 
    {
        unsafe 
        {
            let active_array_buffer = self.gl.get_parameter_buffer(glow::ARRAY_BUFFER).unwrap();
            self.gl.bind_vertex_buffer(0, Some(active_array_buffer), 0, 4);
            self.gl.draw_arrays(glow::TRIANGLES, start, end);
        }
    }

    fn draw_indexed(&self, start: i32, end: i32) 
    {
        todo!()
    }
}

pub struct GLBuffer{
    gl: Arc<glow::Context>,
    buf: glow::NativeBuffer,
    usage: u32,
    target: u32,
}

impl GLBuffer
{
    fn new(gl: Arc<glow::Context>, info: BufferUsage) -> GLBuffer
    {
        let (usage, target) = match info {
            BufferUsage::Uniform => {
                (glow::UNIFORM_BUFFER, glow::DYNAMIC_DRAW)
            }
            BufferUsage::Vertex => {
                (glow::ARRAY_BUFFER, glow::STATIC_DRAW)
            }
        };

        let buf = unsafe {
            gl.create_buffer().expect("failed to create buffer")
        };

        GLBuffer{gl, buf, usage, target}
    }
}

impl Buffer for GLBuffer
{
    fn bind(&self) 
    {
        unsafe 
        {
            self.gl.bind_buffer(self.target, Some(self.buf));
        }
    }

    fn data(&self, data: &[u8]) 
    {
        unsafe 
        {
            self.gl.buffer_data_u8_slice(self.target, data, self.usage);
        }
    }

    fn unbind(&self) 
    {
        unsafe 
        {
            self.gl.bind_buffer(self.target, None);
            self.gl.bind_vertex_buffer(0, None, 0, 0);
        }
    }
}

impl Drop for GLBuffer
{
    fn drop(&mut self) 
    {
        unsafe 
        {
            self.unbind();
            self.gl.delete_buffer(self.buf);
        }
    }
}


struct GLPipeline{
    gl: Arc<glow::Context>,
    vao: glow::NativeVertexArray,
    program: glow::NativeProgram
}

impl GLPipeline 
{
    pub fn new(gl: Arc<glow::Context>) -> GLPipeline
    {
        unsafe 
        {
            let vao = gl.create_vertex_array().unwrap();
            let program = gl.create_program().expect("failed to create buffer");
            GLPipeline{
                gl,
                vao,
                program
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
            self.unbind();
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

impl Pipeline for GLPipeline 
{
    fn add_vertex_description(&self, vbo_layout: BufferLayout) 
    {
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
                    self.gl.vertex_attrib_format_i32(
                        i as u32, 
                        element.quantity as i32, 
                        gltype, 
                        vbo_layout.size() as u32
                    );
                }
                else
                {
                    self.gl.vertex_attrib_format_f32(
                        i as u32, 
                        element.quantity as i32, 
                        gltype, 
                        false, 
                        vbo_layout.size() as u32
                    );
                }
                self.gl.vertex_array_attrib_binding_f32(self.vao, i as u32, 0);
                self.gl.enable_vertex_array_attrib(self.vao, i as u32);
            }
        }
    }

    
    fn add_shader_program(&self, shader_info: ShaderInfo) 
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


    fn bind(&self) 
    {
        unsafe 
        {
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.use_program(Some(self.program));
        }
    }

    
    fn unbind(&self) 
    {
        unsafe 
        {
            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
        }
    }
}
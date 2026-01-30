use glow::HasContext;

use crate::{shader_program, opengl, vulkan};
use std::rc::Rc;
use std::sync::Arc;

pub enum ApiCreateInfo {
    VK,
    GL
}

impl ApiCreateInfo {
    fn request_api(&self) -> ApiHandle {
        match self {
            ApiCreateInfo::VK => {
                ApiHandle{inner: Box::new(vulkan::VKinner::new())}
            }

            ApiCreateInfo::GL => {
                ApiHandle{inner: Box::new(opengl::GLinner{})}
            }
        }
    }
}

pub struct ApiHandle {
    inner: Box<dyn Api>,
}

pub trait Api {
    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<u32, ()>;
    fn destroy_pipeline(&self, pipeline: u32) -> Result<(), ()>;
    fn draw(&self, start: i32, end: i32);
    fn draw_indexed(&self, start: i32, end: i32);
    fn create_buffer(&self, buffer_info: u32) -> Result<u32, ()>;
    fn destroy_buffer(&self, buffer: u32) -> Result<(), ()>;
}

pub struct PipelineInfo {
    a: i32
}

#[derive(Default)]
pub struct BufferLayout {
    elements: Vec<BufferElement>
}

impl BufferLayout {
    pub fn add(&mut self, element: BufferElement) {
        self.elements.push(element);
    }

    pub fn size(&self) -> usize {
        let mut size:usize = 0;
        for element in self.elements.iter() {
           size += element.quantity * element.element_type.size();
        }

        size
    }
}


pub struct BufferElement {
    pub element_type: BufferElementType,
    pub quantity: usize,
    pub normalized: bool
}


pub enum BufferElementType {
    I32,
    U32,
    F32
}

impl BufferElementType {
    pub fn size(&self) -> usize {
        match self {
            Self::I32 => 4,
            Self::U32 => 4,
            Self::F32 => 4,
        }
    }

    pub fn gl_type(&self) -> u32 {
        match self {
            Self::I32 => glow::INT,
            Self::U32 => glow::UNSIGNED_INT,
            Self::F32 => glow::FLOAT,

        }
    }

    pub fn is_integral(&self) -> bool {
        if let Self::I32 | Self::U32 = self {
            true
        } else {false}
    }
}


pub struct GLApi {
    ctx: Arc<glow::Context>
}

impl GLApi {
    pub fn new(ctx: Arc<glow::Context>) -> GLApi {
        GLApi {ctx}
    }
    pub fn upload_buffer<T: Sized>(&self, buf: Vec<T>) -> glow::NativeBuffer {
        unsafe {
            let buffer = self.ctx.create_buffer().expect("failed to create buffer");
            self.ctx.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            self.ctx.named_buffer_data_u8_slice(
                buffer,
                std::slice::from_raw_parts(buf.as_ptr() as _, buf.len() * size_of::<T>()), 
                glow::STATIC_DRAW
            );

            buffer
        }
    }

    pub fn bind_buffer(&self, vao: glow::VertexArray) {
        unsafe{
            self.ctx.bind_vertex_array(Some(vao));
        }
    }

    pub fn create_pipeline(
        &self, 
        shader_program: shader_program::GLShaderProgram, 
        layout: BufferLayout,
        ctx: Arc<glow::Context>
    ) -> GLPipeLine {
        GLPipeLine{shader_program, layout, ctx}
    }

    pub fn use_pipeline(&self, pipeline: &GLPipeLine) {
        unsafe {
            self.ctx.use_program(Some(pipeline.shader_program.native));
        }
    }

    pub fn draw(&self, start: i32, end: i32) {
        unsafe {
            self.ctx.draw_arrays(glow::TRIANGLES, start, end);
        }
    }

    pub fn clear_screen(&self) {
        unsafe {
            self.ctx.clear_color(0.6, 0.8, 0.99, 1.0);
            self.ctx.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }
}


pub struct GLPipeLine {
    pub shader_program: shader_program::GLShaderProgram,
    layout: BufferLayout,
    ctx: Arc<glow::Context>
}

impl GLPipeLine {
    pub fn associate_buffer<T>(&self, buf: glow::NativeBuffer) -> glow::VertexArray {
        unsafe {
            let vao = self.ctx.create_vertex_array().expect("failed to create vao");
            self.ctx.bind_vertex_array(Some(vao));
            self.ctx.bind_buffer(glow::ARRAY_BUFFER, Some(buf));

            for (n, attribute) in self.layout.elements.iter().enumerate() {
                if attribute.element_type.is_integral() {
                    self.ctx.vertex_attrib_pointer_i32(
                        n as u32, 
                        attribute.quantity as i32, 
                        attribute.element_type.gl_type(), 
                        self.layout.size() as i32, 
                        0
                    );
                    self.ctx.enable_vertex_attrib_array(n as u32);
                } else {
                    self.ctx.vertex_attrib_pointer_f32(
                        n as u32, 
                        attribute.quantity as i32, 
                        attribute.element_type.gl_type(), 
                        false,
                        self.layout.size() as i32, 
                        0
                    );
                    self.ctx.enable_vertex_attrib_array(n as u32);
                }
            }

            self.ctx.bind_vertex_array(None);
            self.ctx.bind_buffer(glow::ARRAY_BUFFER, None);

            vao
        }
    }
}
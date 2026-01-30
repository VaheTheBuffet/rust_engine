use crate::{opengl, vulkan};


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
                ApiHandle{inner: Box::new(opengl::GLinner::new(todo!()))}
            }
        }
    }
}

pub struct ApiHandle {
    inner: Box<dyn Api>,
}


pub trait Api {
    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<Box<dyn Pipeline>, ()>;
    fn draw(&self, start: i32, end: i32);
    fn draw_indexed(&self, start: i32, end: i32);
    fn create_buffer(&self, buffer_info: BufferUsage) -> Result<Box<dyn Buffer>, ()>;
}


pub trait Buffer {
    fn bind(&self);
    fn unbind(&self);
    fn data(&self, data: &[u8]);
}


pub trait Pipeline {
    fn bind(&self);
    fn unbind(&self);
    fn add_shader_program(&self, shader_info: ShaderInfo);
    fn add_vertex_description(&self, vbo_layout: BufferLayout);
}


pub enum ShaderInfo<'a> {
    Text(&'a str, &'a str),
    SpirV(&'a [u8], &'a [u8]) 
}


pub enum BufferUsage {
    Vertex,
    Uniform
}


pub struct UniformBufferInfo {
    binding: u32,
    size: u32
}


pub struct PipelineInfo<'a> {
    pub vbo_layout: BufferLayout,
    pub shader_info: ShaderInfo<'a>,
    pub uniform_descriptor: UniformBufferInfo
}


#[derive(Default)]
pub struct BufferLayout {
    pub elements: Vec<BufferElement>
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

    pub fn is_integral(&self) -> bool {
        if let Self::I32 | Self::U32 = self {
            true
        } else {false}
    }
}
use std::any::Any;

use crate::{opengl, vulkan};


pub enum ApiCreateInfo {
    VK,
    GL
}

impl ApiCreateInfo {
    pub fn request_api(&self, pwindow: &mut glfw::PWindow) -> ApiHandle {
        match self {
            ApiCreateInfo::VK => {
                ApiHandle{inner: Box::new(vulkan::VKinner::new())}
            }

            ApiCreateInfo::GL => {
                ApiHandle{inner: Box::new(opengl::GLinner::new(pwindow))}
            }
        }
    }
}

pub struct ApiHandle {
    pub inner: Box<dyn Api>,
}


pub trait Api {
    fn create_pipeline(&self, pipeline_info: PipelineInfo) -> Result<Box<dyn Pipeline>, ()>;
    fn create_command_buffer<'a>(&self) -> Result<Box<dyn CommandBuffer<'a> + 'a>, ()>;
    fn create_buffer(&self, buffer_info: BufferMemory) -> Result<Box<dyn Buffer>, ()>;
    fn create_texture(&mut self, texture_info: TextureCreateInfo) -> Result<Box<dyn Texture>, ()>;
}


pub trait Pipeline {
    fn as_any(&self) -> &dyn Any;
}


pub trait CommandBuffer<'a> {
    fn draw(&self, start:i32, end:i32);
    fn draw_indexed(&self, start:i32, end:i32);
    fn bind_pipeline(&mut self, pipeline: &'a dyn Pipeline);
    fn bind_vertex_buffer(&self, buf: &dyn Buffer); 
    fn bind_buffer(&self, buf: &dyn Buffer, source_binding: usize);
    fn bind_texture(&self, tex: &dyn Texture, source_binding: usize);
    fn submit(&self);
}


pub trait Buffer {
    fn buffer_data(&self, data: &[u8]);
    fn allocate(&self, size: i32);
    fn buffer_sub_data(&self, data: &[u8], offset:i32);
    fn as_any(&self) -> &dyn Any;
}


pub trait Texture {
    fn texture_data(&mut self, data: &[u8]);
    fn as_any(&self) -> &dyn Any;
}


#[derive(Default)]
pub enum ShaderInfo<'a> {
    Text(&'a str, &'a str),
    SpirV(&'a [u8], &'a [u8]),
    #[default]
    Default
}


#[derive(Default, Clone, Copy, Debug)]
pub enum DescriptorInfo {
    Vertex{
        stride: u8,
        bind_point: u8,
    },
    Uniform{
        bind_point: u8,
        size: u8
    },
    Texture{
        bind_point: u8
    },
    #[default]
    Default
}


pub enum BufferMemory{
    ReadOnly,
    Dynamic,
}


pub struct TextureCreateInfo{
    pub width: i32,
    pub height: i32,
    pub layers: i32,
}


#[derive(Default)]
pub struct PipelineInfo<'a> {
    pub vbo_layout: VertexLayout,
    pub shader_info: ShaderInfo<'a>,
    pub descriptor_layouts: Vec<DescriptorInfo>
}


#[derive(Default)]
pub struct VertexLayout {
    pub elements: Vec<BufferElement>,
    pub bind_point: u8
}

impl VertexLayout {
    pub fn new(bind_point: u8) -> VertexLayout
    {
        VertexLayout{elements: Vec::new(), bind_point}
    }

    pub fn add(&mut self, element: BufferElement) 
    {
        self.elements.push(element);
    }

    //None for full size
    pub fn size(&self, idx:Option<usize>) -> usize 
    {
        let mut size = 0;
        for i in 0..idx.unwrap_or(self.elements.len()) {
           size += self.elements[i].quantity * self.elements[i].element_type.size();
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
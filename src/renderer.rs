use std::any::Any;
use crate::{opengl, vk::vulkan};


pub enum ApiCreateInfo {
    VK,
    GL
}

impl ApiCreateInfo {
    pub fn request_api(
        &self, glfw: &mut glfw::Glfw
    ) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>, ApiHandle) 
    {

        match self {
            ApiCreateInfo::VK => {
                glfw.window_hint(glfw::WindowHint::FocusOnShow(true));
                glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

                let (window, events) = glfw.create_window(
                    crate::settings::WIDTH, crate::settings::HEIGHT, "Voxel Engine", glfw::WindowMode::Windowed
                ).expect("Failed to create window");

                let api = ApiHandle{inner: Box::new(vulkan::VKInner::new(&window, glfw))};

                (window, events, api)
            }

            ApiCreateInfo::GL => {
                glfw.window_hint(glfw::WindowHint::ContextVersion(4, 5));
                glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
                glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
                glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(false));
                glfw.window_hint(glfw::WindowHint::FocusOnShow(true));
                #[cfg(debug_assertions)]
                glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

                let (mut window, events) = glfw.create_window(
                    crate::settings::WIDTH, crate::settings::HEIGHT, "Voxel Engine", glfw::WindowMode::Windowed
                ).expect("Failed to create window");

                let api = ApiHandle{inner: Box::new(opengl::GLinner::new(&mut window))};

                <glfw::Window as glfw::Context>::make_current(&mut window);
                glfw.set_swap_interval(glfw::SwapInterval::None);
                (window, events, api)
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
    fn create_buffer(&self, buffer_info: BufferCreateInfo) -> Result<Box<dyn Buffer>, ()>;
    fn create_texture(&self, texture_info: TextureCreateInfo<'_>) -> Result<Box<dyn Texture>, ()>;
}

pub trait Pipeline {
    fn as_any(&self) -> &dyn Any;
}

pub trait CommandBuffer<'a> {
    fn draw(&mut self, start:i32, end:i32);
    fn draw_indexed(&mut self, start:i32, end:i32);
    fn bind_pipeline(&mut self, pipeline: &'a dyn Pipeline);
    fn bind_vertex_buffer(&mut self, buf: &dyn Buffer); 
    fn bind_descriptors(&mut self, descriptors: &[DescriptorWriteInfo]);

    //This method is like Buffer::sub_data, except it's guarunteed to be synchronized
    //with the device
    fn update_buffer(&mut self, buffer: &dyn Buffer, data: &[u8], offset: i32);

    fn begin(&mut self);
    fn submit(&mut self);
}

pub trait Buffer {
    fn sub_data(&self, data: &[u8], offset:i32);
    fn as_any(&self) -> &dyn Any;
}

pub trait Texture {
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

pub enum DescriptorWriteInfo<'a> {
    Uniform{
        handle: &'a dyn Buffer
    },
    Texture{
        handle: &'a dyn Texture
    }
}

impl std::fmt::Debug for DescriptorWriteInfo<'_> {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         let name = match self {
             &DescriptorWriteInfo::Uniform {handle: _} => {"Uniform"},
             &DescriptorWriteInfo::Texture {handle: _} => {"Texture"}
         };

         f.debug_tuple("")
          .field(&name)
          .finish()
     }
}

pub enum BufferCreateInfo<'a>{
    ReadOnly(&'a [u8]),
    Dynamic(usize),
}

pub struct TextureCreateInfo<'a>{
    pub width: i32,
    pub height: i32,
    pub layers: i32,
    pub pixels: &'a [u8]
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

    //None for full size or idx to stop at desired attribute
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

#[derive(Debug)]
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

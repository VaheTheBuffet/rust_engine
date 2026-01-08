pub trait VertexArray {
    fn bind(&self);
    fn unbind(&self);
    fn draw(&self, start:i32, end:i32);
    fn delete(self);
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

    pub fn gl_type(&self) -> gl::types::GLenum {
        match self {
            Self::I32 => gl::INT,
            Self::U32 => gl::FLOAT,
            Self::F32 => gl::UNSIGNED_INT,

        }
    }

    pub fn is_integral(&self) -> bool {
        if let Self::I32 | Self::U32 = self {
            true
        } else {false}
    }
}


pub struct VertexBuffer<T: Sized>(pub Vec<T>);

impl<T> VertexBuffer<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn size(&self) -> usize {
        self.0.len() * core::mem::size_of::<T>()
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}


pub struct GLVertexArray {
    vao: u32,
    len: usize
}

impl GLVertexArray {
    pub fn new<T>(buf: VertexBuffer<T>, layout: BufferLayout) -> Self {
        let mut vao = 0;
        let mut vbo = 0;
        let len = buf.len();

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER, 
                buf.size() as isize, 
                buf.as_ptr() as *const core::ffi::c_void, 
                gl::STATIC_DRAW
            );

            for (n, attribute) in layout.elements.iter().enumerate() {
                if attribute.element_type.is_integral() {
                    gl::VertexAttribIPointer(
                        n as u32, 
                        attribute.quantity as i32, 
                        attribute.element_type.gl_type(),
                        layout.size() as i32,   
                        0 as *const core::ffi::c_void
                    );
                    gl::EnableVertexAttribArray(n as u32);
                } else {
                    gl::VertexAttribPointer(
                        n as u32, 
                        attribute.quantity as i32, 
                        attribute.element_type.gl_type(),
                        false as u8,
                        layout.size() as i32,   
                        0 as *const core::ffi::c_void
                    );
                    gl::EnableVertexAttribArray(n as u32);
                }
            }

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        Self{vao, len}
    }
}

impl VertexArray for GLVertexArray {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    fn delete(self) {
        unsafe{
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }

    fn draw(&self, start:i32, end:i32) {
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, start, end);
        }
    }
}
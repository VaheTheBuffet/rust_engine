use std::ffi::{c_void};

pub struct Quad{
    pub vao:u32,
    pub shader_program:u32
}

impl Quad {
    pub fn new(shader_program:u32) -> Quad {
        let mut vbo:u32 = 0;
        let mut vao:u32 = 0;
        let vertices:[i32;_] = [
            0,0,0, 1,0,0, 1,1,0,
            1,1,0, 0,1,0, 0,0,0
        ];


        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as isize,
                    &vertices[0] as *const _ as *const c_void, gl::STATIC_DRAW);
            
            gl::VertexAttribPointer(0, 3, gl::INT, gl::FALSE, 12, 0 as *const c_void);
            gl::EnableVertexAttribArray(0);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        Quad{vao, shader_program}
    }

    pub fn draw(&self) {
        unsafe{
            gl::BindVertexArray(self.vao);
            gl::UseProgram(self.shader_program);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

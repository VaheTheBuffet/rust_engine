use std::ffi::{c_void};
use crate::shader_program::ShaderProgram;
use crate::math::{IDENTITY};
use crate::camera;

pub fn draw_triangle() -> (u32, u32) {
    let mut vbo:u32 = 0;
    let mut vao:u32 = 0;
    let vertices:[f32; 9] = [
        -0.5, -0.5, -0.11, 0.5, -0.5, -0.11,
        0.0, 0.5, -0.11
    ];


    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 9 * 4 as isize,
                &vertices[0] as *const _ as *const c_void, gl::STATIC_DRAW);
        
        let shader_program:u32 = ShaderProgram::new().triangle;

        gl::UseProgram(shader_program);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, 0, 0, 0 as *const c_void);
        gl::EnableVertexAttribArray(0);

        let mut cam = camera::Camera::new();

        let m_model = gl::GetUniformLocation(shader_program, "m_model\0" as *const _ as *const i8);
        gl::UniformMatrix4fv(m_model, 1, 1, IDENTITY.as_ptr());
        
        let m_view = gl::GetUniformLocation(shader_program, "m_view\0" as *const _ as *const i8);
        gl::UniformMatrix4fv(m_view, 1, 1, cam.get_view_mat().as_ptr());

        let m_proj = gl::GetUniformLocation(shader_program, "m_proj\0" as *const _ as *const i8);
        gl::UniformMatrix4fv(m_proj, 1, 1, cam.get_proj_mat().as_ptr());

        let mut proj = [0f32;16];
        gl::GetUniformfv(shader_program, m_proj, &mut proj[0]);

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::UseProgram(0);

        (vao, shader_program)
    }
}
use std::{ptr::null};
use crate::camera::{Camera, HasCamera};
use crate::math::IDENTITY;


pub struct ShaderProgram {
    pub quad:u32,
    pub chunk:u32
}

impl ShaderProgram {
    pub fn new() -> ShaderProgram {
        let quad = compile_shaders(TRIANGLE_VS, TRIANGLE_FS);
        let chunk = compile_shaders(CHUNK_VS, CHUNK_FS);
        ShaderProgram{quad, chunk}
    }

    pub fn set_uniform(&self, shader_program:&u32, player:&Camera) {
        unsafe {
            gl::UseProgram(*shader_program);

            let m_model = gl::GetUniformLocation(*shader_program, "m_model\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_model, 1, 1, IDENTITY.as_ptr());
            
            let m_view = gl::GetUniformLocation(*shader_program, "m_view\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_view, 1, 1, player.get_view_mat().as_ptr());

            let m_proj = gl::GetUniformLocation(*shader_program, "m_proj\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_proj, 1, 1, player.get_proj_mat().as_ptr());
        }
    }

    pub fn update_uniform(&self, shader_program:&u32, player:&Camera) {
        unsafe {
            gl::UseProgram(*shader_program);
            
            let m_view = gl::GetUniformLocation(*shader_program, "m_view\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_view, 1, 1, player.get_view_mat().as_ptr());

            let m_proj = gl::GetUniformLocation(*shader_program, "m_proj\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_proj, 1, 1, player.get_proj_mat().as_ptr());
        }
    }

    pub fn set_all_uniforms(&mut self, player:&Camera) {
        self.set_uniform(&self.quad, player);
        self.set_uniform(&self.chunk, player);
    }

    pub fn update_all_uniforms(&mut self, player:&Camera) {
        self.update_uniform(&self.quad, player);
        self.update_uniform(&self.chunk, player);
    }

    pub fn update(&mut self, player:&Camera) {
        self.update_all_uniforms(player);
    }
}


pub fn compile_shaders(vertex_shader_source:&str, fragment_shader_source:&str) -> u32 {
    let mut vec:Vec<i8> = vec![0; 512];
    let mut success:i32 = 0;

    unsafe {
        let vertex_shader:u32 = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &vertex_shader_source as *const _ as *const *const i8, null());
        gl::CompileShader(vertex_shader);

        let fragment_shader:u32 = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &fragment_shader_source as *const _ as *const *const i8, null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != 1 {
            gl::GetShaderInfoLog(vertex_shader, 512, null::<i32>() as *mut _, vec.as_mut_ptr());
            let log = String::from_utf8(vec.iter().map(|&s| s as u8).collect());
            panic!("{}", log.unwrap());
        }

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != 1 {
            gl::GetShaderInfoLog(fragment_shader, 512, null::<i32>() as *mut _, vec.as_mut_ptr());
            let log = String::from_utf8(vec.iter().map(|&s| s as u8).collect());
            panic!("{}", log.unwrap());
        }

        let shader_program:u32 = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader_program
    }
}

const TRIANGLE_VS:&str = "#version 330 core
layout (location = 0) in ivec3 pos;

uniform mat4 m_model;
uniform mat4 m_view;
uniform mat4 m_proj;

void main()
{
    gl_Position = m_proj*m_view*m_model*vec4(pos, 1.0);
}\0";

const TRIANGLE_FS:&str = "#version 330 core
out vec4 FragColor;
void main()
{
    FragColor = vec4(0.1,0.5,0.5, 1.0f);
}\0";

const CHUNK_VS:&str = "#version 330 core
layout (location = 0) in ivec3 pos;
layout (location = 1) in int face_id;

uniform mat4 m_model;
uniform mat4 m_view;
uniform mat4 m_proj;

out vec3 color;

vec3 color_generator(int n) {
    return vec3[6](
        vec3(1,0,0), vec3(0,1,0), vec3(0,0,1),
        vec3(0,0,0), vec3(1,1,1), vec3(1,0,1)
    )[n];
}

void main()
{
    color = color_generator(face_id);
    gl_Position = m_proj*m_view*m_model*vec4(pos, 1.0);
}\0";

const CHUNK_FS:&str = "#version 330 core
in vec3 color;

out vec4 FragColor;

void main()
{
    FragColor = vec4(color, 1.0f);
}\0";
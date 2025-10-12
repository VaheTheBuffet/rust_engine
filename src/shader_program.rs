use std::{ptr::null};


pub struct ShaderProgram {
    pub triangle:u32,
}

impl ShaderProgram {
    pub fn new() -> ShaderProgram {
        ShaderProgram{triangle:compile_shaders(TRIANGLE_VS, TRIANGLE_FS)}
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
layout (location = 0) in vec3 aPos;

uniform mat4 m_model;
uniform mat4 m_view;
uniform mat4 m_proj;

void main()
{
    gl_Position = m_proj*m_view*m_model*vec4(aPos, 1.0);
}\0";

const TRIANGLE_FS:&str = "#version 330 core
out vec4 FragColor;
void main()
{
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}\0";
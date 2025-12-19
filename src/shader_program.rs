use std::os::raw::c_void;
use std::{ptr::null};
use crate::camera::{Camera, HasCamera};
use crate::math::IDENTITY;
use image::{self};


pub struct ShaderProgram {
    pub chunk:u32
}

impl ShaderProgram {
    pub fn new() -> ShaderProgram {
        let chunk = compile_shaders(CHUNK_VS, CHUNK_FS);
        ShaderProgram::load_texture("test");
        ShaderProgram::load_texture_array("tex_array_0", 8);

        ShaderProgram{chunk}
    }


    pub fn load_texture_array(filename: &str, n_layers:i32) {
        let image_data = image::ImageReader::open(format!("assets/{}.png", filename)).expect("file not found");
        let decoded_reader = image_data.decode().unwrap();
        let (width, height) = (decoded_reader.width() as i32, decoded_reader.height() as i32);
        let pixel_buffer = decoded_reader.into_rgba8().into_raw();

        unsafe {
            let mut texture:u32 = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture);
            gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, gl::RGBA8 as i32, width, height / n_layers, n_layers, 0, 
                           gl::RGBA, gl::UNSIGNED_BYTE, pixel_buffer.as_ptr() as *const c_void);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_MIN_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_MAG_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_WRAP_S,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_WRAP_T,gl::CLAMP_TO_EDGE as i32);
        }
    }


    pub fn load_texture(filename: &str) {
        let image_data = image::ImageReader::open(format!("assets/{}.png", filename)).expect("file not found");
        let decoded_reader = image_data.decode().unwrap();
        let (width, height) = (decoded_reader.width() as i32, decoded_reader.height() as i32);
        let pixel_buffer = decoded_reader.into_rgba8().into_raw();

        unsafe {
            let mut texture:u32 = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, width, height, 0, gl::RGBA, 
                           gl::UNSIGNED_BYTE, pixel_buffer.as_ptr() as *const c_void);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MIN_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MAG_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_S,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_T,gl::CLAMP_TO_EDGE as i32);
        }
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
        self.set_uniform(&self.chunk, player);
    }

    pub fn update_all_uniforms(&mut self, player:&Camera) {
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
            panic!("VERTEX SHADER FAILED\n{}", log.unwrap());
        }

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != 1 {
            gl::GetShaderInfoLog(fragment_shader, 512, null::<i32>() as *mut _, vec.as_mut_ptr());
            let log = String::from_utf8(vec.iter().map(|&s| s as u8).collect());
            panic!("FRAGMENT SHADER FAILED\n{}", log.unwrap());
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


const CHUNK_VS:&str = "#version 330 core
layout (location = 0) in int compressed_data;

uniform mat4 m_model;
uniform mat4 m_view;
uniform mat4 m_proj;

out vec2 uv_coords;
flat out float shading;
flat out int voxel_id;

ivec3 pos;
int face_id;

void unpack_data(int compressed_data) {
    int COORD_STRIDE = 6; int COORD_MASK = (1<<COORD_STRIDE)-1;
    int FACE_ID_STRIDE = 3;
    int VOXEL_ID_STRIDE = 4; int VOXEL_ID_MASK = (1<<VOXEL_ID_STRIDE)-1;

    voxel_id = compressed_data & VOXEL_ID_MASK; compressed_data >>= VOXEL_ID_STRIDE;
    int z = compressed_data & COORD_MASK; compressed_data >>= COORD_STRIDE;
    int y = compressed_data & COORD_MASK; compressed_data >>= COORD_STRIDE;
    int x = compressed_data & COORD_MASK; compressed_data >>= COORD_STRIDE;
    face_id = compressed_data;
    pos = ivec3(x,y,z);
}

float get_shading(int n) {
    return float[6](
        1.0, 0.4, 0.6, 0.4, 0.7, 0.5
    )[n];
}

const vec2 uv[4] = vec2[] (
    vec2(0,0), vec2(1,0), vec2(1,1), vec2(0,1)
);

const int uv_indices[6] = int[] (
    3, 2, 1, 1, 0, 3
);

const vec2 face_texture_offset[6] = vec2[] (
    vec2(2,0), vec2(0,0), vec2(1,0), vec2(1,0), vec2(1,0), vec2(1,0)
);

void main()
{
    unpack_data(compressed_data);
    uv_coords = (uv[uv_indices[gl_VertexID % 6]] + face_texture_offset[face_id]) * vec2(1/3.0,1);
    shading = get_shading(face_id);
    gl_Position = m_proj*m_view*m_model*vec4(pos, 1.0);
}\0";

const CHUNK_FS:&str = "#version 330 core
in vec2 uv_coords;
flat in float shading;
flat in int voxel_id;

out vec4 FragColor;

//#define TESTING
#ifdef TESTING
uniform sampler2D test;
#else
uniform sampler2DArray tex_array;
#endif

void main()
{
#ifdef TESTING
    FragColor = texture(test, uv_coords);
#else
    FragColor = texture(tex_array, vec3(uv_coords, voxel_id));
#endif

    FragColor *= shading;
}\0";
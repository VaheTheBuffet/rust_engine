use std::os::raw::c_void;
use std::{ptr::null};
use crate::{
    camera::{Camera, Player}, 
    math::IDENTITY, 
    settings
};
use image::{self};

pub trait ShaderProgram {
    fn load_texture(&self, filename: &str);
    fn load_texture_array(&self, filename: &str, n_layers:i32);
}

pub trait UniformFn<T>: ShaderProgram {
    fn set_uniform(&self, name: &str, val: T);
}

pub struct GLShaderProgram {
    pub id: u32
}

impl GLShaderProgram {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Self {
        let mut vec:Vec<i8> = vec![0; 512];
        let mut success:i32 = 0;

        let id = unsafe {
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
        };
        Self{id}
    }
}

impl ShaderProgram for GLShaderProgram {
    fn load_texture(&self, filename: &str) {
        let image_data = image::ImageReader::open(format!("assets/{}.png", filename))
            .expect("file not found");
        let decoded_reader = image_data.decode().unwrap();
        let (width, height) = (decoded_reader.width() as i32, decoded_reader.height() as i32);
        let pixel_buffer = decoded_reader.into_rgba8().into_raw();

        unsafe {
            let mut texture:u32 = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA8 as i32, width, height, 0, gl::RGBA, 
                gl::UNSIGNED_BYTE, pixel_buffer.as_ptr() as *const c_void
            );
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MIN_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MAG_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_S,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_T,gl::CLAMP_TO_EDGE as i32);
        }
    }

    fn load_texture_array(&self, filename: &str, n_layers:i32) {
        let image_data = image::ImageReader::open(format!("assets/{}.png", filename))
            .expect("file not found");
        let decoded_reader = image_data.decode().unwrap();
        let (width, height) = (decoded_reader.width() as i32, decoded_reader.height() as i32);
        let pixel_buffer = decoded_reader.into_rgba8().into_raw();

        unsafe {
            let mut texture:u32 = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture);
            gl::TexImage3D(
                gl::TEXTURE_2D_ARRAY, 0, gl::RGBA8 as i32, width, height / n_layers, n_layers, 0, 
                gl::RGBA, gl::UNSIGNED_BYTE, pixel_buffer.as_ptr() as *const c_void
            );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_MIN_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_MAG_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_WRAP_S,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY,gl::TEXTURE_WRAP_T,gl::CLAMP_TO_EDGE as i32);
        }

    }
}

impl UniformFn<[f32; 3]> for GLShaderProgram {
    fn set_uniform(&self, name: &str, val: [f32; 3]){
        unsafe {
            gl::UseProgram(self.id);
            let uniform_id = gl::GetUniformLocation( self.id, name.as_ptr() as *const i8);
            gl::Uniform3fv(uniform_id, 1, val.as_ptr());
        }
    }
}

impl UniformFn<[f32; 16]> for GLShaderProgram {
    //matrices have to be transposed for now
    fn set_uniform(&self, name: &str, val: [f32; 16]){
        unsafe {
            gl::UseProgram(self.id);
            let uniform_id = gl::GetUniformLocation( self.id, name.as_ptr() as *const i8);
            gl::UniformMatrix4fv(uniform_id, 1, true as gl::types::GLboolean, val.as_ptr());
        }
    }
}


pub struct GlobalShaderProgram {
    pub programs: Vec<GLShaderProgram> 
}

impl GlobalShaderProgram {
    pub fn new() -> GlobalShaderProgram {
        let mut programs = Vec::<GLShaderProgram>::new();
        let chunk = GLShaderProgram::new(CHUNK_VS, CHUNK_FS);
        chunk.load_texture("test");
        chunk.load_texture_array("spritesheet", settings::NUM_TEXTURES);
        programs.push(chunk);
        GlobalShaderProgram{programs}
    }


    pub fn set_uniform(&self, player:&Player) {

        self.programs[0].set_uniform("m_model\0", IDENTITY);
        self.programs[0].set_uniform("m_view\0", player.get_view_mat());
        self.programs[0].set_uniform("m_proj\0", player.get_proj_mat());
    }


    pub fn update_uniform(&self, player:&Player) {
        self.programs[0].set_uniform("m_view\0", player.get_view_mat());
        self.programs[0].set_uniform("m_proj\0", player.get_proj_mat());
    }


    pub fn set_all_uniforms(&self, player:&Player) {
        self.set_uniform(player);
    }


    pub fn update_all_uniforms(&self, player:&Player) {
        self.update_uniform(player);
    }


    pub fn update(&self, player:&Player) {
        self.update_all_uniforms(player);
    }
}


const CHUNK_VS: &'static str = 
    "#version 330 core
    layout (location = 0) in int compressed_data;

    uniform mat4 m_model;
    uniform mat4 m_view;
    uniform mat4 m_proj;

    flat out float shading;
    flat out int voxel_id;
    flat out int face_id;
    out vec3 vertex_pos;

    ivec3 pos;

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
        shading = get_shading(face_id);
        vertex_pos = (m_model * vec4(pos, 1.0)).xyz;
        gl_Position = m_proj * m_view * m_model * vec4(pos, 1.0);
    }\0";

const CHUNK_FS: &'static str = 
    "#version 330 core
    flat in float shading;
    flat in int voxel_id;
    flat in int face_id;

    in vec3 vertex_pos;

    out vec4 FragColor;

    const vec2 face_texture_offset[6] = vec2[] (
        vec2(2,0), vec2(0,0), vec2(1,0), vec2(1,0), vec2(1,0), vec2(1,0)
    );

    vec2 uv[6] = vec2[6](
        fract(vertex_pos.xz), vec2(0,1)+vec2(1,-1)*fract(vertex_pos.xz),
        vec2(1,1)+vec2(-1,-1)*fract(vertex_pos.zy), vec2(0,1)+vec2(1,-1)*fract(vertex_pos.zy),
        vec2(0,1)+vec2(1,-1)*fract(vertex_pos.xy), (1,1)+(-1,-1)*fract(vertex_pos.xy)
    );

    //#define TESTING
    #ifdef TESTING
    uniform sampler2D test;
    #else
    uniform sampler2DArray tex_array;
    #endif

    void main()
    {
        vec2 uv_coords = (uv[face_id] + face_texture_offset[face_id]) * vec2(1/3.0,1);
    #ifdef TESTING
        FragColor = texture(test, uv_coords);
    #else
        FragColor = texture(tex_array, vec3(uv_coords, voxel_id));
    #endif

        FragColor *= shading;
    }\0";
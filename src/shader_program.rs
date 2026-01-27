use std::rc::Rc;
use glow::HasContext;
use image::{self};

pub trait ShaderProgram {
    fn load_texture(&self, filename: &str);
    fn load_texture_array(&self, filename: &str, n_layers:i32);
}

pub trait UniformFn<T>: ShaderProgram {
    fn set_uniform(&self, name: &str, val: T);
}

pub struct GLShaderProgram {
    pub native: glow::NativeProgram,
    ctx: Rc<glow::Context>
}

impl GLShaderProgram {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str, ctx: Rc<glow::Context>) -> Self {
        unsafe {
            let vertex_shader = ctx.create_shader(glow::VERTEX_SHADER).expect("failed to create vertex shader");
            ctx.shader_source(vertex_shader, vertex_shader_source);
            ctx.compile_shader(vertex_shader);

            let fragment_shader = ctx.create_shader(glow::FRAGMENT_SHADER).expect("failed to create fragment shader");
            ctx.shader_source(fragment_shader, fragment_shader_source);
            ctx.compile_shader(fragment_shader);

            if !ctx.get_shader_compile_status(vertex_shader) {
                panic!("VERTEX SHADER FAILED\n{}", ctx.get_shader_info_log(vertex_shader));
            }

            if !ctx.get_shader_compile_status(fragment_shader) {
                panic!("FRAGMENT SHADER FAILED\n{}", ctx.get_shader_info_log(fragment_shader));
            }

            let shader_program = ctx.create_program().expect("failed to create shader program");
            ctx.attach_shader(shader_program, vertex_shader);
            ctx.attach_shader(shader_program, fragment_shader);
            ctx.link_program(shader_program);
            ctx.delete_shader(vertex_shader);
            ctx.delete_shader(fragment_shader);

            Self{native: shader_program, ctx}
        }
    }
}

impl ShaderProgram for GLShaderProgram {
    fn load_texture(&self, filename: &str) {
        let image_data = image::ImageReader::open(format!("assets/{}.png", filename))
            .expect("file not found");
        let decoded_reader = image_data.decode().expect("failed to decode image");
        let (width, height) = (decoded_reader.width() as i32, decoded_reader.height() as i32);
        let pixel_buffer = decoded_reader.into_rgba8().into_raw();

        unsafe {
            let texture = self.ctx.create_texture().expect("failed to create texture");
            self.ctx.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.ctx.tex_image_2d(
                glow::TEXTURE_2D, 
                0, 
                glow::RGB8 as i32, 
                width, 
                height, 
                0, 
                glow::RGBA, 
                glow::UNSIGNED_BYTE, 
                glow::PixelUnpackData::Slice(Some(pixel_buffer.as_slice()))
            );
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        }
    }

    fn load_texture_array(&self, filename: &str, n_layers:i32) {
        let image_data = image::ImageReader::open(format!("assets/{}.png", filename))
            .expect("file not found");
        let decoded_reader = image_data.decode().expect("failed to decode image");
        let (width, height) = (decoded_reader.width() as i32, decoded_reader.height() as i32);
        let pixel_buffer = decoded_reader.into_rgba8().into_raw();

        unsafe {
            let texture = self.ctx.create_texture().expect("failed to create texture array");
            self.ctx.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.ctx.tex_image_3d(
                glow::TEXTURE_2D_ARRAY, 
                0, 
                glow::RGB8 as i32, 
                width, 
                height / n_layers, 
                n_layers,
                0, 
                glow::RGBA, 
                glow::UNSIGNED_BYTE, 
                glow::PixelUnpackData::Slice(Some(pixel_buffer.as_slice()))
            );
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D_ARRAY, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D_ARRAY, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D_ARRAY, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.ctx.tex_parameter_i32(glow::TEXTURE_2D_ARRAY, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        }
    }
}

impl UniformFn<[f32; 3]> for GLShaderProgram {
    fn set_uniform(&self, name: &str, val: [f32; 3]){
        unsafe {
            self.ctx.use_program(Some(self.native));
            let uniform_id = self.ctx.get_uniform_location(self.native, name)
                .expect("failed to get uniform location");
            self.ctx.uniform_3_f32_slice(Some(&uniform_id), &val);
        }
    }
}

impl UniformFn<[f32; 16]> for GLShaderProgram {
    //matrices have to be transposed for now
    fn set_uniform(&self, name: &str, val: [f32; 16]){
        unsafe {
            self.ctx.use_program(Some(self.native));
            let uniform_id = self.ctx.get_uniform_location(self.native, name)
                .expect("failed to get unifrom location");
            self.ctx.uniform_matrix_4_f32_slice(Some(&uniform_id), true, &val);
        }
    }
}
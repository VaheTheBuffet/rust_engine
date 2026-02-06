mod window;
mod camera;
mod math;
mod settings;
mod shader_program;
mod scene;
mod chunk;
mod world;
mod util;
mod renderer;
mod vulkan;
mod opengl;

pub use settings::*;
pub use std::ptr;
pub use std::sync::{Mutex, Arc, Weak, mpsc};
pub use shader_program::{ShaderProgram, UniformFn};
pub use camera::Camera;


fn main() {
    let mut app = window::VoxelEngine::new();
    app.run();
}
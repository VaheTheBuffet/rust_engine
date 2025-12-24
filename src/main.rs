mod window;
mod camera;
mod math;
mod settings;
mod shader_program;
mod scene;
mod chunk;
mod world;
mod util;

fn main() {
    let mut app = window::VoxelEngine::new();
    app.init_gl();
    app.run();
    println!();
}
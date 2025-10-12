mod window;
mod camera;
mod math;
mod settings;
mod triangle;
mod shader_program;

fn main() {
    let mut app = window::VoxelEngine::new();
    app.init_gl();
    app.run();
}

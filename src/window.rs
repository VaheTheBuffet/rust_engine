use std::{ffi::CString, os::raw::c_void, ptr::null};
use glfw::{self, fail_on_errors, Action, Context, Key};
use crate::triangle::draw_triangle;


pub struct VoxelEngine {
    glfw:glfw::Glfw,
    window:glfw::PWindow,
    events:glfw::GlfwReceiver<(f64, glfw::WindowEvent)>
}

impl VoxelEngine {
    pub fn new() -> VoxelEngine{
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(false));

        let (mut window, events) = glfw.create_window(1280, 720, "Voxel Engine", glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        window.make_current();
        window.set_key_polling(true);

        VoxelEngine{glfw, window, events}
    }


    pub fn init_gl(&mut self) {
        let func = |s| {self.window.get_proc_address(s).unwrap_or(
            self.window.get_proc_address("glActiveShaderProgram").unwrap()) as *const c_void};
        gl::load_with(func);
    }


    pub fn handle_events(&mut self) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                },
                _ => {},
            }
        }
    }


    pub fn draw(&mut self) {
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }


    pub fn run(&mut self) {
        let (tri_vao, tri_shader) = draw_triangle();

        while !self.window.should_close() {
            self.handle_events();
            self.draw();
            unsafe {
                gl::BindVertexArray(tri_vao);
                gl::UseProgram(tri_shader);
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
            }
            self.window.swap_buffers();
        }
        unsafe{
            gl::UseProgram(0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}


use std::{ptr::null};
use glfw:: {
    self, Action, Context, Key, fail_on_errors, ffi::{
        GLFW_CURSOR, GLFW_CURSOR_DISABLED, GLFW_PRESS, GLFW_RAW_MOUSE_MOTION, GLFW_TRUE, 
        glfwGetCursorPos, glfwGetKey, glfwGetTime, glfwSetInputMode, glfwSetWindowTitle
    }
};
use crate::{camera::{Camera, HasCamera}, scene::Scene};
use crate::settings::*;


pub struct VoxelEngine {
    glfw:glfw::Glfw,
    window:glfw::PWindow,
    events:glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    player:Camera
}

impl VoxelEngine {
    pub fn new() -> VoxelEngine{
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(false));
        glfw.window_hint(glfw::WindowHint::FocusOnShow(true));

        let (mut window, events) = glfw.create_window(
            WIDTH, HEIGHT, "Voxel Engine", glfw::WindowMode::Windowed
        ).expect("Failed to create window");

        unsafe{
            glfwSetInputMode(window.window_ptr(), GLFW_CURSOR, GLFW_CURSOR_DISABLED);
            glfwSetInputMode(window.window_ptr(), GLFW_RAW_MOUSE_MOTION, GLFW_TRUE);
        }

        window.make_current();
        window.show();
        window.set_key_polling(true);


        let player = Camera::new();

        VoxelEngine{glfw, window, events, player}
    }


    pub fn init_gl(&mut self) {
        let func = |s| {
            match self.window.get_proc_address(s) {Some(f) => f as _, _ => null()} 
        };

        gl::load_with(func);
        unsafe{gl::Enable(gl::DEPTH_TEST);gl::Enable(gl::CULL_FACE)}
    }


    pub fn handle_events(&mut self, delta_time:&f32) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                },
                _ => {},
            }
        }
        unsafe{
            if glfwGetKey(self.window.window_ptr(), Key::W as i32) == GLFW_PRESS {
                self.player.move_forward(delta_time);
            }
            if glfwGetKey(self.window.window_ptr(), Key::S as i32) == GLFW_PRESS {
                self.player.move_backward(delta_time);
            }
            if glfwGetKey(self.window.window_ptr(), Key::A as i32) == GLFW_PRESS {
                self.player.move_left(delta_time);
            }
            if glfwGetKey(self.window.window_ptr(), Key::D as i32) == GLFW_PRESS {
                self.player.move_right(delta_time);
            }
            if glfwGetKey(self.window.window_ptr(), Key::Q as i32) == GLFW_PRESS {
                self.player.move_up(delta_time);
            }
            if glfwGetKey(self.window.window_ptr(), Key::E as i32) == GLFW_PRESS {
                self.player.move_down(delta_time);
            }
        }
    }


    pub fn handle_mouse_move(&mut self, dx:f64, dy:f64) {
        self.player.rot_yaw(&(dx as f32));
        self.player.rot_pitch(&(dy as f32));
    }


    pub fn draw(&mut self) {
        unsafe {
            gl::ClearColor(0.6, 0.8, 0.99, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn run(&mut self) {
        let mut last_update_time = 0.0;
        let mut second = 1.0;
        //unsafe{gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE)};

        let (mut x1, mut y1) = (0f64, 0f64);
        unsafe {
            glfwGetCursorPos(self.window.window_ptr(), &mut x1, &mut y1);
        }

        let mut scene = Scene::new();

        //mesh_builder_thread is currently mostly unsynchronized 
        //TODO: remake this in safe rust which would require a bunch of arcs idk
        let engine_ptr = self as *mut _ as u64;
        let scene_ptr = &mut scene as *mut _ as u64;
        std::thread::spawn(move|| {
            let class_ref = unsafe{&mut *(engine_ptr as *mut VoxelEngine)};
            let scene_ref = unsafe{&mut *(scene_ptr as *mut Scene)};
            loop {
                scene_ref.mesh_builder_thread(&class_ref.player);
            }
        });

        while !self.window.should_close() {
            self.draw();
            scene.draw(&self.player);
            let now = unsafe{glfwGetTime()};
            let delta_time = now - last_update_time;
            second -= delta_time;
            if second <= 0.0 {
                unsafe{glfwSetWindowTitle(
                    self.window.window_ptr(), 
                    format!("{}\0", (1.0/delta_time) as i32).as_ptr() as *const _
                );}
                second = 1.0;
            }

            let (x0, y0) = (x1, y1);
            unsafe{glfwGetCursorPos(self.window.window_ptr(), &mut x1, &mut y1);}
            self.handle_mouse_move(x1-x0, y1-y0);
            self.handle_events(&(delta_time as f32));
            self.player.update();
            scene.update(&self.player);
            self.window.swap_buffers();
            last_update_time = now;
        }

        unsafe{
            gl::UseProgram(0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}
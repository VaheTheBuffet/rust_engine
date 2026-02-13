use glfw::{
    self, Action, Context, Key, fail_on_errors,
};
use crate::{*, camera::{self, Camera, Player}, scene::Scene};
pub struct VoxelEngine 
{
    glfw:glfw::Glfw,
    window:glfw::PWindow,
    events:glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    player: Player,
}

impl VoxelEngine 
{
    pub fn new() -> VoxelEngine
    {
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();

        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 5));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(false));
        glfw.window_hint(glfw::WindowHint::FocusOnShow(true));

        let (mut window, events) = glfw.create_window(
            WIDTH, HEIGHT, "Voxel Engine", glfw::WindowMode::Windowed
        ).expect("Failed to create window");

        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_raw_mouse_motion(true);

        window.make_current();
        window.show();
        window.set_key_polling(true);

        let player = Player::new();

        VoxelEngine{
            glfw, 
            window, 
            events, 
            player,
        }
    }


    pub fn handle_events(&mut self) -> Vec<crate::camera::PlayerEvent>
    {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) 
        {
            match event 
            {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                    break;
                }
                _ => {},
            }
        }

        use crate::camera::PlayerEvent;
        let mut player_events = Vec::<PlayerEvent>::new();
        if self.window.get_key(Key::W) == Action::Press 
        {
            player_events.push(PlayerEvent::MoveForward);
        }
        if self.window.get_key(Key::S) == Action::Press 
        {
            player_events.push(PlayerEvent::MoveBackward);
        }
        if self.window.get_key(Key::D) == Action::Press 
        {
            player_events.push(PlayerEvent::MoveRight);
        }
        if self.window.get_key(Key::A) == Action::Press 
        {
            player_events.push(PlayerEvent::MoveLeft);
        }
        if self.window.get_key(Key::Q) == Action::Press 
        {
            player_events.push(PlayerEvent::MoveUp);
        }
        if self.window.get_key(Key::E) == Action::Press 
        {
            player_events.push(PlayerEvent::MoveDown);
        }

        player_events
    }


    pub fn handle_mouse_move(&mut self, dx:f64, dy:f64) -> Vec<camera::PlayerEvent> {
        use crate::camera::PlayerEvent;
        vec![PlayerEvent::RotYaw(dx as f32), PlayerEvent::RotPitch(dy as f32)]
    }


    pub fn run(&mut self) 
    {
        let mut last_update_time = 0.0;
        let mut second = 1.0;

        let (mut x1, mut y1) = self.window.get_cursor_pos();
        let (chunk_tx, chunk_rx) = std::sync::mpsc::channel::<world::ChunkCluster>();
        let (mesh_tx, mesh_rx) = std::sync::mpsc::channel::<chunk::ChunkMesh>();

        let api = renderer::ApiCreateInfo::GL.request_api(&mut self.window, &self.glfw);

        let mut scene = Scene::new(api, mesh_rx, chunk_tx);
        let mesh_builder = scene::MeshBuilder::new(chunk_rx, mesh_tx);

        std::thread::spawn( move|| 
            {
                loop 
                {
                    mesh_builder.build_mesh();
                }
            });

        let mut n_frames = 0;

        while !self.window.should_close() 
        {
            n_frames += 1;
            scene.draw(&self.player);
            let now = self.glfw.get_time();
            let delta_time = now - last_update_time;
            second -= delta_time;
            if second <= 0.0 
            {
                self.window.set_title(&format!("{}", n_frames));
                n_frames = 0;
                second = 1.0;
            }

            let (x0, y0) = (x1, y1);
            (x1, y1) = self.window.get_cursor_pos();

            let mut player_events = self.handle_events();
            player_events.extend(self.handle_mouse_move(x1 - x0, y1 - y0));

            self.player.update(player_events.as_slice(), delta_time as f32);
            scene.update(&self.player);
            self.window.swap_buffers();
            last_update_time = now;
        }
    }
}
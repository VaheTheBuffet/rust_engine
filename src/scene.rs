use crate::camera::Camera;
use crate::shader_program::ShaderProgram;
use crate::world::World;

pub struct Scene {
    shader_program: ShaderProgram,
    world: World,
}

impl Scene {
    pub fn new()-> Scene {
        let shader_program = ShaderProgram::new();
        let world = World::new();
        World::init(&shader_program);

        Scene{shader_program, world}
    }


    pub fn draw(&mut self, player:&Camera) {
        let time = std::time::Instant::now();
        static mut AVERAGE:std::time::Duration = std::time::Duration::from_secs(0);
        static mut N:f32 = 0.0;

        self.world.threaded_update_visible_chunks(player);
        self.world.draw(player);

        //print!("{:?} {:?} {}\r", unsafe{AVERAGE}, time.elapsed(), unsafe{N});
        unsafe{
            N += 1.0;
            AVERAGE = AVERAGE.mul_f32(1.0-2.0/(1.0+N)) + time.elapsed().mul_f32(2.0/(1.0+N));
        };
    }


    pub fn mesh_builder_thread(&mut self, player: &Camera) {
        self.world.mesh_builder_thread(player);
    }


    pub fn init(&mut self, player:&Camera) {
        self.shader_program.set_all_uniforms(player);
    }


    pub fn update(&mut self, player:&Camera) {
        self.shader_program.update(player);
    }

}
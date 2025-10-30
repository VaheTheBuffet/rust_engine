use crate::camera::Camera;
use crate::chunk::Chunk;
use crate::shader_program::ShaderProgram;
use crate::world::World;

pub struct Scene {
    shader_program:ShaderProgram,
    world:World
}

impl Scene {
    pub fn new()-> Scene {
        let shader_program = ShaderProgram::new();
        Chunk::init(&shader_program);
        let mut world = World::new();
        World::init(&shader_program);
        world.build_chunks();

        Scene{shader_program, world}
    }

    pub fn draw(&self) {
        self.world.draw();
    }

    pub fn init(&mut self, player:&Camera) {
        self.shader_program.set_all_uniforms(player);
    }

    pub fn update(&mut self, player:&Camera) {
        self.shader_program.update(player)
    }
}
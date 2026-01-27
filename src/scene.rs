use std::collections::HashMap;
use crate::camera::{Camera, Player};
use crate::chunk::{self, ChunkMesh};
use crate::{math, renderer, settings};
use crate::shader_program::{self, ShaderProgram, UniformFn};
use crate::util::render_range;
use crate::world::World;
use glow::HasContext;
use rayon::prelude::*;

pub struct Scene {
    api: renderer::GLApi,
    world: World,
    mesh_load_queue: std::sync::Mutex<Vec<ChunkMesh>>,
    meshes: HashMap<(i32,i32,i32), (glow::VertexArray, usize)>,
    chunk_pipeline: renderer::GLPipeLine
}

impl Scene {
    pub fn new(ctx: std::rc::Rc<glow::Context>)-> Scene {
        let chunk_vert = std::fs::read_to_string("./shaders/chunk.vert").expect("failed to find shader file");
        let chunk_frag = std::fs::read_to_string("./shaders/chunk.frag").expect("failed to find shader file");
        let shader_program = shader_program::GLShaderProgram::new(
            &chunk_vert,
            &chunk_frag,
            ctx.clone()
        );

        unsafe {
            ctx.enable(glow::DEPTH_TEST);
            ctx.enable(glow::CULL_FACE);
            ctx.enable(glow::BLEND);
        }

        shader_program.load_texture("test");
        shader_program.load_texture_array("spritesheet", settings::NUM_TEXTURES);
        let mut layout = renderer::BufferLayout::default();
        layout.add(renderer::BufferElement{
            element_type: renderer::BufferElementType::U32, 
            quantity: 1, 
            normalized: false
        });

        let api = renderer::GLApi::new(ctx.clone());
        let chunk_pipeline = api.create_pipeline(shader_program, layout, ctx.clone());

        let world = World::new();
        Scene{
            api, 
            world, 
            mesh_load_queue:std::sync::Mutex::new(Vec::new()), 
            meshes:HashMap::new(),
            chunk_pipeline
        }
    }

    pub fn draw(&mut self, player:&Player) {
        self.api.use_pipeline(&self.chunk_pipeline);
        self.api.clear_screen();
        //let time = std::time::Instant::now();
        //static mut AVERAGE:std::time::Duration = std::time::Duration::from_secs(0);
        //static mut N:f32 = 0.0;

        let ptr = renderer::GLApi::draw as *const std::ffi::c_void as usize;
        for pos in render_range((player.chunk_x, player.chunk_y, player.chunk_z)) {
            if let Some((mesh, len)) = self.meshes.get(&pos) {
                if *len > 0 && player.is_in_frustum(self.world.chunks.get(&pos).unwrap().center) {
                    self.chunk_pipeline.shader_program.set_uniform("m_model", math::get_model(pos));
                    self.api.bind_buffer(*mesh);
                    self.api.draw(0, *len as i32);
                }
            }
        }

        //print!("{:?} {:?} {}\r", unsafe{AVERAGE}, time.elapsed(), unsafe{N});
        //unsafe{
        //    N += 1.0;
        //    AVERAGE = AVERAGE.mul_f32(1.0-2.0/(1.0+N)) + time.elapsed().mul_f32(2.0/(1.0+N));
        //};
    }

    pub fn mesh_builder_thread(&mut self, player: &Player) {
        let (build_positions, terrain_positions, dirty_positions) = self.world.promote_chunks(player);

        let new_chunks: Vec<_> = build_positions.par_iter().map(
            |&pos| {self.world.chunk_build_task(pos, &self.world.noise)}
        ).collect();
        for chunk in new_chunks {self.world.chunks.insert(chunk.pos, chunk);}

        let entities: Vec<_> = terrain_positions.par_iter().map(
            |&pos| {self.world.entity_build_task(pos)}
        ).collect();
        self.world.decorate(entities).expect("entity generation failed");

        if dirty_positions.len() > 0 {
            self.mesh_load_queue.lock().unwrap().par_extend(dirty_positions.par_iter().map(
                |&pos| {self.world.mesh_build_task(pos)}
            ));
        }
        dirty_positions.iter().for_each(|p| self.world.chunks.get_mut(p).unwrap().status = chunk::ChunkStatus::Clean);
    }

    pub fn update(&mut self, player:&Player) {
        self.chunk_pipeline.shader_program.set_uniform("m_view", player.get_view_mat());
        self.chunk_pipeline.shader_program.set_uniform("m_proj", player.get_proj_mat());
        if let Some(mesh) = self.mesh_load_queue.lock().unwrap().pop() {
            let len = mesh.vertices.len();
            let buf = self.api.upload_buffer(mesh.vertices);
            let vao = self.chunk_pipeline.associate_buffer::<u32>(buf);
            self.meshes.insert(mesh.pos, (vao, len));
        }
    }
}
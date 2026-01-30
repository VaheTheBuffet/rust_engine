use crate::*;
use std::collections::HashMap;
use glow::HasContext;
use rayon::prelude::*;

pub struct Scene 
{
    api: renderer::GLApi,
    world: world::World,
    chunk_mesh_rx: mpsc::Receiver<chunk::ChunkMesh>,
    chunk_tx: mpsc::Sender<world::ChunkCluster>,
    meshes: HashMap<(i32,i32,i32), (glow::VertexArray, usize)>,
    chunk_pipeline: renderer::GLPipeLine
}

impl Scene 
{
    pub fn new(
        ctx: std::sync::Arc<glow::Context>, 
        chunk_mesh_rx: mpsc::Receiver<chunk::ChunkMesh>,
        chunk_tx: mpsc::Sender<world::ChunkCluster>) -> Scene 
    {
        let chunk_vert = std::fs::read_to_string("./shaders/chunk.vert")
            .expect("failed to find shader file");
        let chunk_frag = std::fs::read_to_string("./shaders/chunk.frag")
            .expect("failed to find shader file");
        let shader_program = shader_program::GLShaderProgram::new(
            &chunk_vert,
            &chunk_frag,
            ctx.clone());

        unsafe 
        {
            ctx.enable(glow::DEPTH_TEST);
            ctx.enable(glow::CULL_FACE);
            ctx.enable(glow::BLEND);
        }

        shader_program.load_texture("test");
        shader_program.load_texture_array("spritesheet", settings::NUM_TEXTURES);
        let mut layout = renderer::BufferLayout::default();
        layout.add(
            renderer::BufferElement{
                element_type: renderer::BufferElementType::U32, 
                quantity: 1, 
                normalized: false});

        let api = renderer::GLApi::new(ctx.clone());
        let chunk_pipeline = api.create_pipeline(shader_program, layout, ctx.clone());

        let world = world::World::new();
        Scene
        {
            api, 
            world, 
            chunk_mesh_rx,
            chunk_tx,
            meshes: HashMap::new(),
            chunk_pipeline
        }
    }

    pub fn draw(&self, player:&camera::Player) 
    {
        self.api.use_pipeline(&self.chunk_pipeline);
        self.api.clear_screen();
        //let time = std::time::Instant::now();
        //static mut AVERAGE:std::time::Duration = std::time::Duration::from_secs(0);
        //static mut N:f32 = 0.0;

        for pos in util::render_range((player.chunk_x, player.chunk_y, player.chunk_z)) 
        {
            if let Some((mesh, len)) = self.meshes.get(&pos) 
            {
                if *len > 0 && player.is_in_frustum(util::chunk_center_from_global_index(pos)) 
                {
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

    pub fn update(&mut self, player:&camera::Player) 
    {
        self.chunk_pipeline.shader_program.set_uniform("m_view", player.get_view_mat());
        self.chunk_pipeline.shader_program.set_uniform("m_proj", player.get_proj_mat());
        for _ in 0..2
        {
            if let Ok(mesh) = self.chunk_mesh_rx.try_recv()
            {
                let len = mesh.vertices.len();
                let buf = self.api.upload_buffer(mesh.vertices);
                let vao = self.chunk_pipeline.associate_buffer::<u32>(buf);
                self.meshes.insert(mesh.pos, (vao, len));
            }
        }

        self.update_world(player);
    }


    pub fn update_world(&mut self, player: &camera::Player) 
    {
        let (build_positions,
            terrain_positions,
            dirty_positions) = self.world.promote_chunks(player);

        let new_chunks: Vec<_> = build_positions.par_iter()
            .map(|&p| 
            {
                self.world.chunk_build_task(p)
            }).collect();
        
        for chunk in new_chunks 
        {
            self.world.chunks.insert(chunk.pos, Arc::new(chunk));
        }

        let entities: Vec<_> = terrain_positions.par_iter()
            .map(|&p| 
            {
                self.world.entity_build_task(p)
            }).collect();
        
        self.world.decorate(entities).expect("failed to generate entities");

        if !dirty_positions.is_empty()
        {
            for p in dirty_positions.iter()
            {
                let chunk = Arc::try_unwrap(self.world.chunks.remove(p).unwrap());
                if let Ok(mut chunk) = chunk 
                {
                    chunk.status = chunk::ChunkStatus::Clean;
                    self.world.chunks.insert(*p, Arc::new(chunk));
                }
                else 
                {
                    panic!("tried to gain exclusive access to chunk at {:?}", *p);
                }
            }

            dirty_positions.par_iter().for_each(|p|
            {
                let cluster = world::ChunkCluster::new(&self.world, p.0, p.1, p.2);
                let _ = self.chunk_tx.send(cluster);
            });
        }
    }
}


pub struct MeshBuilder 
{
    mesh_tx: mpsc::Sender<chunk::ChunkMesh>,
    chunk_rx: mpsc::Receiver<world::ChunkCluster>
}

impl MeshBuilder 
{
    pub fn new( chunk_rx: mpsc::Receiver<world::ChunkCluster>, 
            mesh_tx: mpsc::Sender<chunk::ChunkMesh>) -> MeshBuilder
    {
        MeshBuilder{mesh_tx, chunk_rx}
    }

    pub fn build_mesh(&self) 
    {
        for chunk in self.chunk_rx.iter()
        {
            let tx_clone = self.mesh_tx.clone();
            rayon::spawn( move|| 
                {
                    let center = chunk.clone().center.unwrap();
                    let _ = tx_clone.send(center.get_mesh(chunk));
                });
        }
    }
}
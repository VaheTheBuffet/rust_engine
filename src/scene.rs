use crate::*;
use std::{collections::HashMap, mem::offset_of};
use image::EncodableLayout;
use rayon::prelude::*;


pub struct Transform 
{
    model: [f32; 16],
    view: [f32; 16],
    proj: [f32; 16]
}

impl Transform {
    pub fn as_bytes(&self) -> &[u8] 
    {
        unsafe 
        {
            std::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Transform>())
        }
    }
}

pub struct Scene<'a>
{
    world: world::World,
    chunk_mesh_rx: mpsc::Receiver<chunk::ChunkMesh>,
    chunk_tx: mpsc::Sender<world::ChunkCluster>,

    command_buffer: Box<dyn renderer::CommandBuffer<'a> +'a>,

    meshes: HashMap<(i32,i32,i32), (Box<dyn renderer::Buffer>, i32)>,
    uniform_buffer: Box<dyn renderer::Buffer>,
    texture: Box<dyn renderer::Texture>,

    chunk_pipeline: Box<dyn renderer::Pipeline>,
    api: renderer::ApiHandle,
}

pub struct MeshBuilder 
{
    mesh_tx: mpsc::Sender<chunk::ChunkMesh>,
    chunk_rx: mpsc::Receiver<world::ChunkCluster>
}


impl<'a> Scene<'a>
{
    pub fn new(
        mut api: renderer::ApiHandle,
        chunk_mesh_rx: mpsc::Receiver<chunk::ChunkMesh>,
        chunk_tx: mpsc::Sender<world::ChunkCluster>) -> Scene<'a>
    {
        let mut layout = renderer::VertexLayout::new(0);
        layout.add(
            renderer::BufferElement{
                element_type: renderer::BufferElementType::U32, 
                quantity: 1, 
                normalized: false
            }
        );

//        let shader_info = renderer::ShaderInfo::SpirV(
//            &std::fs::read("./shaders/chunk_vert.spv")
//                .expect("failed to read shader"), 
//            &std::fs::read("./shaders/chunk_frag.spv")
//                .expect("failed to read shader")
//        );

        let shader_info = renderer::ShaderInfo::Text(
            &std::fs::read_to_string("./shaders/chunk.vert")
                .expect("failed to read shader"), 
            &std::fs::read_to_string("./shaders/chunk.frag")
                .expect("failed to read shader")
        );

        let uniform_descriptor = renderer::DescriptorInfo::Uniform{
            size: size_of::<Transform>() as i32 as _, 
            bind_point: 0
        };

        let texture_descriptor = renderer::DescriptorInfo::Texture {
            bind_point: 1
        };

        let mut pipeline_info = renderer::PipelineInfo::default();
        pipeline_info.vbo_layout = layout;
        pipeline_info.shader_info = shader_info;
        pipeline_info.descriptor_layouts = vec![uniform_descriptor, texture_descriptor];

        let chunk_pipeline = api.inner.create_pipeline(pipeline_info)
            .expect("failed to create chunk pipeline");
        
        let mut command_buffer = api.inner.create_command_buffer()
            .expect("failed to create command buffer");

        let uniform_buffer = api.inner.create_buffer(
            renderer::BufferCreateInfo::Dynamic(size_of::<Transform>())
        ).expect("failed to create uniform buffer");

        let tex_array_data = image::open("./assets/spritesheet.png")
            .expect("failed to read spritesheet")
            .into_rgba8();

        let texture = api.inner.create_texture(
            renderer::TextureCreateInfo{
                width: tex_array_data.width().try_into().unwrap(),
                height: <u32 as TryInto<i32>>::try_into(tex_array_data.height()).unwrap() / NUM_TEXTURES,
                layers: NUM_TEXTURES,
                pixels: tex_array_data.into_raw().as_slice()}
        ).expect("failed to create texture resource");

        let descriptors = vec![
            renderer::DescriptorWriteInfo::Uniform{handle: uniform_buffer.as_ref()},
            renderer::DescriptorWriteInfo::Texture{handle: texture.as_ref()}
        ];

        command_buffer.bind_pipeline(unsafe{&*((&*chunk_pipeline) as *const _)});
        command_buffer.bind_descriptors(descriptors.as_slice());

        let world = world::World::new();
        Scene
        {
            api, 
            world, 
            chunk_mesh_rx,
            chunk_tx,
            meshes: HashMap::new(),
            chunk_pipeline,
            command_buffer,
            uniform_buffer,
            texture
        }
    }


    pub fn draw(&mut self, player:&camera::Player) 
    {
        self.command_buffer.begin();
        for pos in util::render_range((player.chunk_x, player.chunk_y, player.chunk_z)) 
        {
            if let Some((mesh, len)) = self.meshes.get(&pos) 
            {
                if *len > 0 //&& player.is_in_frustum(util::chunk_center_from_global_index(pos)) 
                {
                    let transform = Transform{
                        model: math::get_model(pos),
                        view: player.get_view_mat(),
                        proj: player.get_proj_mat()
                    };

                    self.command_buffer.update_buffer(self.uniform_buffer.as_ref(), transform.as_bytes(), 0);
                    self.command_buffer.bind_vertex_buffer(mesh.as_ref());
                    self.command_buffer.draw(0, *len as i32);
                }
            }
        }
        self.command_buffer.submit();
    }


    pub fn update(&mut self, player:&camera::Player) 
    {
        for _ in 0..10
        {
            if let Ok(mesh) = self.chunk_mesh_rx.try_recv() && mesh.vertices.len() > 0
            {
                let len = mesh.vertices.len();
                let bytes = unsafe{std::slice::from_raw_parts(mesh.vertices.as_ptr() as *const u8, len * 4)};
                let buf = self.api.inner.create_buffer(
                    renderer::BufferCreateInfo::ReadOnly(bytes))
                    .expect("failed to create vertex buffer");

                self.meshes.insert(mesh.pos, (buf, len as i32));
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
                let new_chunk = self.world.chunks.remove(p).unwrap()
                    .unwrap_arc()
                    .with_status(chunk::ChunkStatus::Clean)
                    .wrap_arc();
                self.world.chunks.insert(*p, new_chunk);
            }

            dirty_positions.par_iter().for_each(|p|
            {
                let cluster = world::ChunkCluster::new(&self.world, p.0, p.1, p.2);
                let _ = self.chunk_tx.send(cluster);
            });
        }
    }
}


impl MeshBuilder 
{
    pub fn new( 
        chunk_rx: mpsc::Receiver<world::ChunkCluster>, 
        mesh_tx: mpsc::Sender<chunk::ChunkMesh>) -> MeshBuilder
    {
        MeshBuilder{mesh_tx, chunk_rx}
    }

    pub fn build_mesh(&self) 
    {
        //for chunk in self.chunk_rx.iter()
        if let Ok(chunk) = self.chunk_rx.recv()
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

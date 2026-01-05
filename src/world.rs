use crate::{
    camera::Camera, 
    chunk::{Chunk, ChunkMeshData, ChunkStatus}, 
    math, 
    settings::*, 
    shader_program::ShaderProgram, 
    util::{self, Noise, render_range}
};
use std::{collections::HashMap};
use rayon::prelude::*;


pub struct World {
    chunks:HashMap<(i32,i32,i32), Chunk>,
    noise: Noise,
    meshes:std::sync::Mutex<Vec<ChunkMeshData>>
}

impl World {
    pub fn new() -> Self {
        let noise = Noise::new(SEED);
        Self{chunks:HashMap::new(), noise, meshes: std::sync::Mutex::new(Vec::new())}
    }


    fn chunk_build_task(&self, (x,y,z):(i32,i32,i32), noise:&Noise) -> Chunk {
        let mut chunk = Chunk::new(x, y, z);
        chunk.build_voxels(noise);
        chunk
    }


    fn entity_build_task(&self, (x,y,z):(i32,i32,i32)) -> Vec<(i32,i32,i32,ENTITIES)> {
        self.chunks.get(&(x,y,z)).unwrap().generate_entities()
    }


    fn mesh_build_task(&self, (x, y, z):(i32, i32, i32)) -> ChunkMeshData {
        let chunk_cluster = ChunkCluster::new(&self, x, y, z);
        self.chunks.get(&(x,y,z)).unwrap().get_vertex_data_optimized(chunk_cluster)
    }


    pub fn get_voxel(&self, global_x:i32, global_y:i32, global_z:i32) -> VOXELS{
        let cx = (global_x as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cy = (global_y as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cz = (global_z as f32 * INV_CHUNK_SIZE).floor() as i32;
        let lx = global_x % CHUNK_SIZE;
        let ly = global_y % CHUNK_SIZE;
        let lz = global_z % CHUNK_SIZE;

        self.chunks.get(&(cx,cy,cz)).unwrap().get_voxel(lx, ly, lz)
    }


    pub fn set_voxel(&mut self, global_x:i32, global_y:i32, global_z:i32, voxel:VOXELS) -> Result<(), ()> {
        let cx = (global_x as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cy = (global_y as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cz = (global_z as f32 * INV_CHUNK_SIZE).floor() as i32;
        let lx = global_x.rem_euclid(CHUNK_SIZE);
        let ly = global_y.rem_euclid(CHUNK_SIZE);
        let lz = global_z.rem_euclid(CHUNK_SIZE);

        self.chunks.get_mut(&(cx,cy,cz)).expect(&format!(
            "no chunk at {}, {}, {}\n", cx, cy, cz
        )).set_voxel(lx, ly, lz, voxel)
    }


    pub fn draw(&mut self, player:&Camera) {
        for x in player.chunk_x-RENDER_DISTANCE..=player.chunk_x+RENDER_DISTANCE {
            for y in player.chunk_y-RENDER_DISTANCE..=player.chunk_y+RENDER_DISTANCE {
                for z in player.chunk_z-RENDER_DISTANCE..=player.chunk_z+RENDER_DISTANCE {
                    //let chunk = self.chunks.get(&(x,y,z)).expect("chunk not set before drawing");
                    if let Some(chunk) = self.chunks.get(&(x,y,z)) {
                        if chunk.status == ChunkStatus::Clean {
                            chunk.draw();
                       } 
                    }
                }
            }
        }
    }


    pub fn promote_chunks(&mut self, player:&Camera) -> (
        Vec<(i32, i32, i32)>, Vec<(i32, i32, i32)>, Vec<(i32, i32, i32)>
    ) {
        let (px, py, pz) = (player.chunk_x, player.chunk_y, player.chunk_z);
        let dirty_positions: Vec<_> = render_range((px,py,pz)).filter(|&pos| {
            if let Some(chunk) = self.chunks.get(&pos) {
                match chunk.status {
                    ChunkStatus::Dirty => {
                        true
                    }
                    ChunkStatus::Terrain => {
                        true
                    }
                    _ => false
                }
            } else {
                true
            }
        }).collect();

        let mut build_positions: Vec<_> = render_range((px,py,pz)).filter(|&pos| {
            if let None = self.chunks.get(&(pos)) {true} else {false}
        }).collect();

        let terrain_positions: Vec<_> = render_range((px,py,pz)).filter(|&pos| {
            if let Some(chunk) = self.chunks.get(&pos) {
                match chunk.status {
                    ChunkStatus::Terrain => {
                        true
                    }
                    _ => false
                }
            } else {
                true
            }
        }).collect();

        //handle edge chunks
        build_positions.extend(util::border_range((px,py,pz)).filter(|&pos| {
            if let None = self.chunks.get(&pos) {true} else {false}
        }).collect::<Vec<_>>());
        (build_positions, terrain_positions, dirty_positions)
    }


    pub fn mesh_builder_thread(&mut self, player:&Camera) {
        let (build_positions, terrain_positions, dirty_positions) = self.promote_chunks(player);

        let new_chunks: Vec<_> = build_positions.par_iter().map(
            |&pos| {self.chunk_build_task(pos, &self.noise)}
        ).collect();
        for chunk in new_chunks {self.chunks.insert(chunk.pos, chunk);}

        let entities: Vec<_> = terrain_positions.par_iter().map(
            |&pos| {self.entity_build_task(pos)}
        ).collect();
        self.decorate(entities).expect("entity generation failed");

        if dirty_positions.len() == 0 {return}
        let new_meshes: Vec<_> = dirty_positions.par_iter().map(
            |&pos| {self.mesh_build_task(pos)}
        ).collect();

        dirty_positions.iter().for_each(|p| self.chunks.get_mut(p).unwrap().status = ChunkStatus::Clean);
        let mut meshes = self.meshes.lock().unwrap();
        meshes.extend(new_meshes);
    }


    pub fn threaded_update_visible_chunks(&mut self, player:&Camera) {
        let mut meshes = self.meshes.lock().unwrap();
        while let Some(mesh) = meshes.pop() {
            self.chunks.get_mut(&(mesh.pos)).unwrap().build_mesh(mesh);
        }
    }


    pub fn init(shader_program:&ShaderProgram) {
        Chunk::init(shader_program);
    }


    pub fn build_tree_at(&mut self, (x,y,z):(i32,i32,i32)) -> Result<(), ()> {
        static HASH_LEN:usize = math::HASH.len();
        let key1 = (x*13+y*3+z*3) as usize % HASH_LEN;

        let height = math::HASH[key1].rem_euclid(4) + 4;
        for ty in 0..height {
            self.set_voxel(x,y+ty,z, VOXELS::WOOD)?;
        }

        self.set_voxel(x, y+height, z, VOXELS::LEAF)?;

        for ty in 0..=3 {
            let key2 = (x*5+ty*11+z*13) as usize %HASH_LEN;
            let stride = (3-ty)+math::HASH[key2] % 3;
            for tx in -stride..=stride {
                for tz in -stride..=stride {
                    self.set_voxel(x+tx, y+height+ty, z+tz, VOXELS::LEAF)?;
                }
            }
        }

        Ok(())
    }


    pub fn decorate(&mut self, entities: Vec<Vec<(i32, i32, i32, ENTITIES)>>) -> Result<(), ()> {
        for chunk in entities {
            for (x,y,z,entity) in chunk {
                match entity {
                    ENTITIES::SEED => {
                        self.build_tree_at((x,y,z))?;
                    }
                }
            }
        }
        Ok(())
    }
}


#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Face {
    Top, Bottom, Right, Left, Front, Back
}

impl Face {
    pub fn id(&self) -> i32 {
        match self {
            Face::Top => 0,
            Face::Bottom => 1,
            Face::Right => 2,
            Face::Left => 3,
            Face::Front => 4,
            Face::Back => 5
        }
    }

    pub fn offset(&self) -> (i32, i32, i32) {
        match self {
            Face::Top => (0,1,0),
            Face::Bottom => (0,-1,0),
            Face::Right => (1,0,0),
            Face::Left => (-1,0,0),
            Face::Front => (0,0,1),
            Face::Back => (0,0,-1)
        }
    }

    #[inline]
    pub fn iter() -> impl Iterator<Item = Face> {
        [
            Face::Top, Face::Bottom, Face::Right, 
            Face::Left, Face::Front, Face::Back
        ].into_iter()
    }

    pub fn coords(&self) -> [[i32; 3]; 6] {
        match self {
            Face::Top => [[0,1,1], [1,1,1], [1,1,0], [1,1,0], [0,1,0], [0,1,1]],
            Face::Bottom => [[0,0,0], [1,0,0], [1,0,1], [1,0,1], [0,0,1], [0,0,0]],
            Face::Right => [[1,0,1], [1,0,0], [1,1,0], [1,1,0], [1,1,1], [1,0,1]],
            Face::Left =>[[0,0,0], [0,0,1], [0,1,1], [0,1,1], [0,1,0], [0,0,0]],
            Face::Front => [[0,0,1], [1,0,1], [1,1,1], [1,1,1], [0,1,1], [0,0,1]],
            Face::Back => [[1,0,0], [0,0,0], [0,1,0], [0,1,0], [1,1,0], [1,0,0]]
        }
    }
}


pub struct ChunkCluster<'a> {
    center:Option<&'a Chunk>,
    top:Option<&'a Chunk>,
    bottom:Option<&'a Chunk>,
    right:Option<&'a Chunk>,
    left:Option<&'a Chunk>,
    front:Option<&'a Chunk>,
    back:Option<&'a Chunk>
}

impl<'a> ChunkCluster<'a> {
    pub fn new(world:&'a World, x:i32, y:i32, z:i32) -> Self { 
        ChunkCluster {
            center:Some(world.chunks.get(&(x,y,z)).expect("Center must be valid")),
            top:world.chunks.get(&(x,y+1,z)),
            bottom:world.chunks.get(&(x,y-1,z)),
            right:world.chunks.get(&(x+1,y,z)),
            left:world.chunks.get(&(x-1,y,z)),
            front:world.chunks.get(&(x,y,z+1)),
            back:world.chunks.get(&(x,y,z-1))
        }
    }


    pub fn get_voxel(&self, local_x:i32, local_y:i32, local_z:i32) -> VOXELS {
        if let Some(chunk) = match (
            local_x.div_euclid(CHUNK_SIZE),
            local_y.div_euclid(CHUNK_SIZE),
            local_z.div_euclid(CHUNK_SIZE)
        ) {
            (0,0,0) => self.center,
            (0,1,0) => self.top,
            (0,-1,0) => self.bottom,
            (1,0,0) => self.right,
            (-1,0,0) => self.left,
            (0,0,1) => self.front,
            (0,0,-1) => self.back,
            _ => None
        } {
            let chunk_x = local_x.rem_euclid(CHUNK_SIZE);
            let chunk_y = local_y.rem_euclid(CHUNK_SIZE);
            let chunk_z = local_z.rem_euclid(CHUNK_SIZE);

            chunk.get_voxel(chunk_x, chunk_y, chunk_z)
        } else {
            VOXELS::EMPTY
        }
    }


    pub fn is_solid(&self, local_x:i32, local_y:i32, local_z:i32) -> bool {
        self.get_voxel(local_x, local_y, local_z) != VOXELS::EMPTY
    }


    pub fn is_face_visible(&self, x:i32, y:i32, z:i32, face:Face) -> bool {
        let (dx, dy, dz) = face.offset();
        self.get_voxel(x+dx, y+dy, z+dz) == VOXELS::EMPTY
    }
}
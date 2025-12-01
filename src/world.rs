use std::{collections::HashMap, iter::zip};
use crate::{camera::Camera, chunk::{Chunk, ChunkMeshData}, settings::*, shader_program::ShaderProgram};
use noise::Perlin;
use rayon::prelude::*;

pub struct World {
    chunks:HashMap<(i32,i32,i32), Chunk>
}

impl World {
    pub fn new() -> Self {
        Self{chunks:HashMap::new()}
    }


    pub fn _build_chunks(&mut self) {
        for x in 0..WORLD_W {
            for z in 0..WORLD_W {
                for y in 0..WORLD_H {
                    self.chunks.insert((x,y,z), Chunk::new(x,y,z));
                    //self.chunks.get_mut(&(x,y,z)).unwrap().build_voxels();
                }
            }
        }
        
        self._build_all_meshes();
    }


    fn chunk_build_task(&self, x:i32, y:i32, z:i32, perlin:&Perlin) -> Chunk {
        let mut chunk = Chunk::new(x, y, z);
        chunk.build_voxels(perlin);
        chunk
    }



    fn mesh_build_task(&self, x:i32, y:i32, z:i32) -> ChunkMeshData {
        let chunk_cluster = ChunkCluster::new(&self, x, y, z);
        self.chunks.get(&(x,y,z)).unwrap().get_vertex_data(chunk_cluster)
    }


    fn _build_all_meshes(&mut self) {
        for x in 0..WORLD_W {
            for z in 0..WORLD_W {
                for y in 0..WORLD_H {
                    self.mesh_build_task(x, y, z);
                }
            }
        }
    }


    pub fn get_voxel(&self, x:i32, y:i32, z:i32) {
        
    }


    pub fn draw(&mut self, player:&Camera) {
        self.threaded_update_visible_chunks(player);

        let (px, py, pz) = (player.x as i32 / CHUNK_SIZE, player.y as i32 / CHUNK_SIZE, player.z as i32 / CHUNK_SIZE);
        for x in px-RENDER_DISTANCE..px+RENDER_DISTANCE {
            for y in py-RENDER_DISTANCE..py+RENDER_DISTANCE {
                for z in pz-RENDER_DISTANCE..pz+RENDER_DISTANCE {
                    let chunk = self.chunks.get(&(x,y,z)).expect("chunk not set before drawing");
                    if !chunk.empty {
                        chunk.draw();
                    }
                }
            }
        }
    }


    pub fn threaded_update_visible_chunks(&mut self, player:&Camera) {
        let (px, py, pz) = (player.x as i32 / CHUNK_SIZE, player.y as i32 / CHUNK_SIZE, player.z as i32 / CHUNK_SIZE);
        let mut dirty_positions: Vec<(i32, i32, i32)> = Vec::new();
        let mut build_positions: Vec<(i32, i32, i32)> = Vec::new();

        for x in px-RENDER_DISTANCE..px+RENDER_DISTANCE {
            for y in py-RENDER_DISTANCE..py+RENDER_DISTANCE {
                for z in pz-RENDER_DISTANCE..pz+RENDER_DISTANCE {
                    if let Some(chunk) = self.chunks.get(&(x,y,z)) {
                        if chunk.dirty {
                            dirty_positions.push((x,y,z));
                        }                    
                    } else {
                        build_positions.push((x,y,z));
                        dirty_positions.push((x,y,z));
                    }
                }
            }
        }

        let perlin = Perlin::new(1);
        let chunks: Vec<Chunk> = build_positions.par_iter().map(|(x,y,z)| {self.chunk_build_task(*x,*y,*z, &perlin)}).collect();
        for chunk in chunks { self.chunks.insert(chunk.pos, chunk); }
        let mesh_data:Vec<ChunkMeshData> = dirty_positions.par_iter_mut().map(|(x,y,z)| {self.mesh_build_task(*x,*y,*z)}).collect();
        for (pos, data) in zip(dirty_positions, mesh_data) {
            let chunk = self.chunks.get_mut(&pos).unwrap();
            chunk.build_mesh(data);
            chunk.dirty = false;
        }
    }


    pub fn init(shader_program:&ShaderProgram) {
        Chunk::init(shader_program);
    }

}


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
    pub fn iter() -> [Face; 6] {
        [Face::Top, Face::Bottom, Face::Right, Face::Left, Face::Front, Face::Back]
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


    pub fn is_face_visible(&self, x:i32, y:i32, z:i32, face:Face) -> bool {
        let (dx, dy, dz) = face.offset();
        self.get_voxel(x+dx, y+dy, z+dz) == VOXELS::EMPTY
    }
}
use std::collections::HashMap;
use crate::{chunk::Chunk, shader_program::ShaderProgram};
use crate::{chunk, settings::*};

pub struct World {
    chunks:HashMap<(i32, i32, i32), Chunk>
}

impl World {
    pub fn new() -> Self {
        Self{chunks:HashMap::new()}
    }


    pub fn build_chunks(&mut self) {
        for x in 0..WORLD_W {
            for z in 0..WORLD_W {
                for y in 0..WORLD_H {
                    self.chunks.insert((x, y, z), Chunk::new(x, y, z));
                    self.chunks.get_mut(&(x,y,z)).unwrap().build_voxels();
                }
            }
        }
        
        self.build_all_meshes();
    }


    fn build_all_meshes(&mut self) {
        for x in 0..WORLD_W {
            for z in 0..WORLD_W {
                for y in 0..WORLD_H {
                    let chunk_cluster = ChunkCluster::new(self, x, y, z);
                    let mesh_data = self.chunks.get(&(x,y,z)).unwrap().get_vertex_data(chunk_cluster);
                    self.chunks.get_mut(&(x,y,z)).unwrap().build_mesh(mesh_data);
                }
            }
        }
    }


    pub fn draw(&self) {
        for (_, chunk) in self.chunks.iter() {
            chunk.draw();
        }
    }


    pub fn update(&mut self) {
        for (_, chunk) in self.chunks.iter_mut() {
            chunk.update();
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
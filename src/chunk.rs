use std::ffi::c_void;
use crate::math;
use crate::util::Noise;
use crate::world::{ChunkCluster, Face};
use crate::{math::get_model, settings::*};
use crate::shader_program::ShaderProgram;
use noise::{Perlin};

static mut PROGRAM:u32 = 0;
static mut PERLIN:Option<Perlin> = None;


pub struct Chunk {
    pub pos: (i32, i32, i32),
    pub voxels: Vec<VOXELS>,
    pub vao: Option<u32>,
    pub vertex_count: i32,
    pub status: ChunkStatus
}

impl Chunk {
    pub fn init(shader_program:&ShaderProgram) {
        unsafe{
            PROGRAM = shader_program.chunk;
            PERLIN = Some(Perlin::new(1));
        }
    }


    pub fn new(x:i32, y:i32, z:i32) -> Chunk {
        Chunk{
            pos:(x, y, z), 
            voxels:vec![VOXELS::EMPTY;CHUNK_VOL as usize], 
            vao:None, 
            vertex_count:0, 
            status:ChunkStatus::Empty}
    }


    pub fn get_voxel(&self, x:i32, y:i32, z:i32) -> VOXELS {
        self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize]
    }


    pub fn set_voxel(&mut self, x:i32, y:i32, z:i32, voxel:VOXELS) -> Result<(), ()> {
        if x >= 0 && x < CHUNK_SIZE && y >= 0 && y < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE {
            self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize] = voxel;
            Ok(())
        }else {
            Err(())
        }
    }


    pub fn draw(&self) {
        unsafe {
            gl::UseProgram(PROGRAM);
            let m_model = gl::GetUniformLocation(PROGRAM, "m_model\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_model, 1, 1, get_model(self.pos).as_ptr());
            gl::BindVertexArray(self.vao.unwrap());
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }


    pub fn _update(&mut self) {
    }


    pub fn build_voxels(&mut self, noise:&Noise) {
        let global_pos = (
            self.pos.0*CHUNK_SIZE, self.pos.1*CHUNK_SIZE, self.pos.2*CHUNK_SIZE
        );

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let mut global_height: i32 = noise.get_height(
                    (x + global_pos.0) as f64, 
                    (z + global_pos.2) as f64
                );
                global_height = if global_height > 0 {global_height} else {1};
                for y in {
                    std::cmp::max(-global_pos.1, 0)
                    ..std::cmp::min(global_height-global_pos.1, CHUNK_SIZE) 
                }{
                    _ = self.generate_terrain((x, y, z), global_pos, global_height);
                }
            }
        }
        self.status = ChunkStatus::Dirty;
    }


    pub fn generate_terrain(
        &mut self, (x,y,z):(i32, i32, i32), (cx,cy,cz):(i32, i32, i32),
        global_height:i32
    ) -> Result<(), ()> {

        let (_, gy, _) = (cx+x,cy+y,cz+z);
        let voxel = {
            if cy >= 50 && y > 5 {VOXELS::SNOW}
            else if gy >= 35 && gy < 50 {VOXELS::COBBLESTONE}
            else if gy >= 10 && gy < 35 && gy == global_height-1 {VOXELS::GRASS}
            else if gy >= 7 && gy < 10 {VOXELS::DIRT}
            else {VOXELS::SAND}
        };

        match voxel {
            VOXELS::SAND => {
                if gy == 0 {
                    self.set_voxel(x, y+1, z, VOXELS::WOOD)?;
                    self.set_voxel(x, y+2, z, VOXELS::WOOD)?;
                    self.set_voxel(x, y+3, z, VOXELS::WOOD)?;
                    self.set_voxel(x, y+4, z, VOXELS::WOOD)?;
                }
            }
            _ => {}
        }

        self.set_voxel(x, y, z, voxel)?;
        Ok(())
    }


    pub fn generate_entities(&self) -> Vec<(i32,i32,i32,ENTITIES)> {
        let mut entities:Vec<(i32, i32, i32, ENTITIES)> = Vec::new();
        let (cx,cy,cz) = (self.pos.0*CHUNK_SIZE,self.pos.1*CHUNK_SIZE,self.pos.2*CHUNK_SIZE);
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel_global_pos = (cx+x,cy+y,cz+z);
                    let voxel = self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize];
                    self.add_entity(voxel_global_pos, voxel, &mut entities);
                }
            }
        }
        entities
    }


    pub fn add_entity(
        &self, (x,y,z):(i32,i32,i32), voxel:VOXELS, entities:&mut Vec<(i32,i32,i32,ENTITIES)>
    ) {
        match voxel {
            VOXELS::GRASS => {
                if math::HASH[((x*3+z*17) as usize)%math::HASH.len()] < 1 {
                    entities.push((x,y,z,ENTITIES::SEED));
                }
            }
            _ => {}
        }
    }


    pub fn get_vertex_data(&self, chunk_cluster:ChunkCluster) -> ChunkMeshData {
        let mut vertex_data: Vec<u32> = Vec::new();

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {

                    let voxel_id = self.get_voxel(x, y, z);
                    if voxel_id == VOXELS::EMPTY {continue;}

                    for face in Face::iter() {
                        if chunk_cluster.is_face_visible(x, y, z, face) {
                            for coord in face.coords() {
                                let (vertex_x, vertex_y, vertex_z) = (x + coord[0], y + coord[1], z + coord[2]);
                                vertex_data.push(self.compress_data(vertex_x, vertex_y, vertex_z, face, voxel_id));
                            }
                        }
                    }
                }
            }
        }

        ChunkMeshData::new(vertex_data)
    }


    pub fn compress_data(&self, x:i32, y:i32, z:i32, face:Face, voxel_id:VOXELS) -> u32 {
        static COORD_STRIDE:u8 = 6;
        static FACE_ID_STRIDE:u8 = 3;
        static VOXEL_ID_STRIDE:u8 = 4;

        let mut res:u32 = 0;

        res |= face.id() as u32;
        res <<= COORD_STRIDE; res |= x as u32; 
        res <<= COORD_STRIDE; res |= y as u32; 
        res <<= COORD_STRIDE; res |= z as u32; 
        res <<= VOXEL_ID_STRIDE; res |= voxel_id as u32;

        res
    }


    pub fn build_mesh(&mut self, chunk_mesh_data:ChunkMeshData) -> (i32, u32) {
        let mut vao = 0;
        let mut vbo = 0;

        let (len, vertex_data) = (chunk_mesh_data.vertices.len(), chunk_mesh_data.vertices);
        self.vertex_count = len as i32;
        self.status = if len > 0 {ChunkStatus::Clean} else {ChunkStatus::Empty};

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, 4*len as isize, vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
            gl::VertexAttribIPointer(0, 1, gl::INT, 4,  0 as *const c_void);
            gl::EnableVertexAttribArray(0);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        self.vao = Some(vao);
        (len as i32, vao)
    }


    fn get_face_vertices(&self, x:i32, y:i32, z:i32, face:Face) -> Vec<i32> {
        static INDICES:[[i32;4];8] = [
            [0,0,1,0], [1,0,1,0], [1,1,1,0], [0,1,1,0],
            [1,0,0,0], [0,0,0,0], [0,1,0,0], [1,1,0,0]
        ];

        let face_indices = match face {
            Face::Top => [3,2,7,7,6,3],
            Face::Bottom => [5,4,1,1,0,5],
            Face::Right => [1,4,7,7,2,1],
            Face::Left => [5,0,3,3,6,5],
            Face::Front => [0,1,2,2,3,0],
            Face::Back => [4,5,6,6,7,4]
        };
        face_indices.iter().flat_map(|i| INDICES[*i as usize].iter().enumerate().map(|(idx, dr)| [x,y,z,face.id()][idx] + *dr)).collect()
    }
}


pub struct ChunkMeshData {
    vertices:Vec<u32>
}

impl ChunkMeshData {
    pub fn new(vertices:Vec<u32>) -> Self {
        ChunkMeshData{vertices}
    }
}


#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum ChunkStatus {
    Empty,
    Terrain,
    Dirty,
    Clean,
}

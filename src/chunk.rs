use std::ffi::c_void;
use crate::world::{ChunkCluster, Face};
use crate::{math::get_model, settings::*};
use crate::shader_program::ShaderProgram;
use noise::{NoiseFn, Perlin};

static mut PROGRAM:u32 = 0;

pub struct Chunk {
    pos:(i32, i32, i32),
    voxels:Vec<VOXELS>,
    vao:Option<u32>,
    vertex_count:i32
}

impl Chunk {
    pub fn init(shader_program:&ShaderProgram) {
        unsafe{PROGRAM = shader_program.chunk;}
    }


    pub fn new(x:i32, y:i32, z:i32) -> Chunk {
        Chunk{pos:(x, y, z), voxels:vec![VOXELS::EMPTY;CHUNK_VOL as usize], vao:None, vertex_count:0}
    }


    pub fn get_voxel(&self, x:i32, y:i32, z:i32) -> VOXELS {
        if x >= 0 && x < CHUNK_SIZE && y >= 0 && y < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE {
            self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize]
        } 
        else {
            VOXELS::EMPTY
        }
    }


    pub fn set_voxel(&mut self, x:i32, y:i32, z:i32, voxel:VOXELS) -> Result<(), ()> {
        if x >= 0 && x < CHUNK_SIZE && y >= 0 && y < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE {
            self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize] = voxel;
            Ok(())
        } 
        else {
            Err(())
        }
    }


    pub fn draw(&self) {
        unsafe{
            gl::UseProgram(PROGRAM);
            let m_model = gl::GetUniformLocation(PROGRAM, "m_model\0" as *const _ as *const i8);
            gl::UniformMatrix4fv(m_model, 1, 1, get_model(self.pos).as_ptr());
            gl::BindVertexArray(self.vao.unwrap());
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }


    pub fn update(&mut self) {
        return 
    }


    pub fn build_voxels(&mut self) {
        let perlin = Perlin::new(1);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let global_height = CHUNK_SIZE + (perlin.get([(x+self.pos.0*CHUNK_SIZE) as f64 / 100.1, (z+self.pos.2*CHUNK_SIZE) as f64 / 100.1]) * CHUNK_SIZE as f64) as i32;
                for y in 0..global_height-self.pos.1*CHUNK_SIZE {
                    _ = self.set_voxel(x, y, z, VOXELS::SOLID);
                }
            }
        }
    }


    pub fn get_vertex_data(&self, chunk_cluster:ChunkCluster) -> ChunkMeshData {
        let mut vertex_data = Vec::new();

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {

                    if self.get_voxel(x, y, z) == VOXELS::EMPTY {continue;}

                    for face in [Face::Top, Face::Bottom, Face::Right, Face::Left, Face::Front, Face::Back] {
                        if chunk_cluster.is_face_visible(x, y, z, face) {
                            vertex_data.extend(self.get_face_vertices(x, y, z, face));
                        }
                    }
                }
            }
        }

        ChunkMeshData::new(vertex_data)
    }


    pub fn build_mesh(&mut self, chunk_mesh_data:ChunkMeshData) {
        let mut vao = 0;
        let mut vbo = 0;

        let (len, vertex_data) = (chunk_mesh_data.vertices.len(), chunk_mesh_data.vertices);
        self.vertex_count = len as i32;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, 4*len as isize, vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 16, 0 as *const c_void);
            gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 16, 12 as *const c_void);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        self.vao = Some(vao);
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


fn add_data(data:&mut [i32], idx:&mut usize, new_data:[(i32, i32, i32, i32);6]) {
    for (x, y, z, face_id) in new_data {
        data[*idx] = x;
        *idx+=1;
        data[*idx] = y;
        *idx+=1;
        data[*idx] = z;
        *idx+=1;
        data[*idx] = face_id;
        *idx+=1
    }
}

pub struct ChunkMeshData {
    vertices:Vec<i32>
}

impl ChunkMeshData {
    pub fn new(vertices:Vec<i32>) -> Self {
        ChunkMeshData{vertices}
    }
}
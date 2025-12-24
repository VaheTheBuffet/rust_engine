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


    #[inline(always)]
    pub fn get_voxel(&self, x:i32, y:i32, z:i32) -> VOXELS {
        self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize]
    }


    pub fn set_voxel(&mut self, x:i32, y:i32, z:i32, voxel:VOXELS) -> Result<(), ()> {
        self.status = ChunkStatus::Dirty;
        if x >= 0 && x < CHUNK_SIZE && y >= 0 && y < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE {
            self.voxels[(x+z*CHUNK_SIZE+y*CHUNK_AREA) as usize] = voxel;
            Ok(())
        } else {
            Err(())
        }
    }


    pub fn draw(&self) {
        unsafe {
            if let Some(vao) = self.vao {
                gl::UseProgram(PROGRAM);
                let m_model = gl::GetUniformLocation(PROGRAM, "m_model\0" as *const _ as *const i8);
                gl::UniformMatrix4fv(m_model, 1, 1, get_model(self.pos).as_ptr());
                gl::BindVertexArray(vao);
                gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
            }
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
        self.status = if self.voxels.iter().all(|v| *v == VOXELS::EMPTY) {
            ChunkStatus::Empty
        } else {ChunkStatus::Dirty}
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
        if self.status == ChunkStatus::Empty {
            return ChunkMeshData::new(self.pos, vertex_data)
        }

        //face id order
        let mut mask_0: Vec<u64> = vec![0;CHUNK_AREA as usize];
        let mut mask_1: Vec<u64> = vec![0;CHUNK_AREA as usize];
        let mut mask_2: Vec<u64> = vec![0;CHUNK_AREA as usize];
        let mut mask_3: Vec<u64> = vec![0;CHUNK_AREA as usize];
        let mut mask_4: Vec<u64> = vec![0;CHUNK_AREA as usize];
        let mut mask_5: Vec<u64> = vec![0;CHUNK_AREA as usize];

        for a in 0..CHUNK_SIZE {
            for b in 0..CHUNK_SIZE {
                mask_0[(a + b * CHUNK_SIZE) as usize] |= (chunk_cluster.get_voxel(a,-1,b) != VOXELS::EMPTY) as u64;
                mask_1[(a + b * CHUNK_SIZE) as usize] |= (chunk_cluster.get_voxel(a,-1,b) != VOXELS::EMPTY) as u64;
                mask_2[(a + b * CHUNK_SIZE) as usize] |= (chunk_cluster.get_voxel(-1,a,b) != VOXELS::EMPTY) as u64;
                mask_3[(a + b * CHUNK_SIZE) as usize] |= (chunk_cluster.get_voxel(-1,a,b) != VOXELS::EMPTY) as u64;
                mask_4[(a + b * CHUNK_SIZE) as usize] |= (chunk_cluster.get_voxel(a,b,-1) != VOXELS::EMPTY) as u64;
                mask_5[(a + b * CHUNK_SIZE) as usize] |= (chunk_cluster.get_voxel(a,b,-1) != VOXELS::EMPTY) as u64;
                for t in 0..CHUNK_SIZE {
                    mask_0[(a + b * CHUNK_SIZE) as usize] |= ((self.get_voxel(a,t,b) != VOXELS::EMPTY) as u64) << t+1;
                    mask_1[(a + b * CHUNK_SIZE) as usize] |= ((self.get_voxel(a,t,b) != VOXELS::EMPTY) as u64) << t+1;
                    mask_2[(a + b * CHUNK_SIZE) as usize] |= ((self.get_voxel(t,a,b) != VOXELS::EMPTY) as u64) << t+1;
                    mask_3[(a + b * CHUNK_SIZE) as usize] |= ((self.get_voxel(t,a,b) != VOXELS::EMPTY) as u64) << t+1;
                    mask_4[(a + b * CHUNK_SIZE) as usize] |= ((self.get_voxel(a,b,t) != VOXELS::EMPTY) as u64) << t+1;
                    mask_5[(a + b * CHUNK_SIZE) as usize] |= ((self.get_voxel(a,b,t) != VOXELS::EMPTY) as u64) << t+1;
                }
                mask_0[(a + b * CHUNK_SIZE) as usize] |= ((chunk_cluster.get_voxel(a,CHUNK_SIZE,b) != VOXELS::EMPTY) as u64) << 33;
                mask_1[(a + b * CHUNK_SIZE) as usize] |= ((chunk_cluster.get_voxel(a,CHUNK_SIZE,b) != VOXELS::EMPTY) as u64) << 33;
                mask_2[(a + b * CHUNK_SIZE) as usize] |= ((chunk_cluster.get_voxel(CHUNK_SIZE,a,b) != VOXELS::EMPTY) as u64) << 33;
                mask_3[(a + b * CHUNK_SIZE) as usize] |= ((chunk_cluster.get_voxel(CHUNK_SIZE,a,b) != VOXELS::EMPTY) as u64) << 33;
                mask_4[(a + b * CHUNK_SIZE) as usize] |= ((chunk_cluster.get_voxel(a,b,CHUNK_SIZE) != VOXELS::EMPTY) as u64) << 33;
                mask_5[(a + b * CHUNK_SIZE) as usize] |= ((chunk_cluster.get_voxel(a,b,CHUNK_SIZE) != VOXELS::EMPTY) as u64) << 33;
            }
        }

        for mask in [mask_0.iter_mut(), mask_2.iter_mut(), mask_4.iter_mut()] {
            for row in mask {
                *row = (!*row >> 1) & *row;
                *row >>= 1;
                *row &= 0xFFFFFFFF;
            }
        }

        for mask in [mask_1.iter_mut(), mask_3.iter_mut(), mask_5.iter_mut()] {
            for row in mask {
                *row = (!*row << 1) & *row;
                *row >>= 1;
                *row &= 0xFFFFFFFF;
            }
        }

        //for mask in [
            //&mut mask_0, &mut mask_1, &mut mask_2, 
            //&mut mask_3, &mut mask_4, &mut mask_5 
            //{l}] {
        //}

        vertex_data.extend(mask_0.iter().enumerate().flat_map(|(i, &(mut row))| {
            let mut res = Vec::<u32>::new();
            let (mut x, mut y, mut z); y = 0;
            while row > 0 {
                x = (i & 0x1F) as i32;
                let r = row.trailing_zeros();
                y += r as i32;
                z = (i >> 5) as i32;
                let voxel = self.get_voxel(x, y, z);
                res.push(self.compress_data(x, y+1, z+1, Face::Top, voxel));
                res.push(self.compress_data(x+1, y+1, z+1, Face::Top, voxel));
                res.push(self.compress_data(x+1, y+1, z, Face::Top, voxel));
                res.push(self.compress_data(x+1, y+1, z, Face::Top, voxel));
                res.push(self.compress_data(x, y+1, z, Face::Top, voxel));
                res.push(self.compress_data(x, y+1, z+1, Face::Top, voxel));
                row >>= r; row >>= 1; y += 1;
            }
            res
        }));

        vertex_data.extend(mask_1.iter().enumerate().flat_map(|(i, &(mut row))| {
            let mut res = Vec::<u32>::new();
            let (mut x, mut y, mut z); y = 0;
            while row > 0 {
                x = (i & 0x1F) as i32;
                let r = row.trailing_zeros();
                y += r as i32;
                z = (i >> 5) as i32;
                let voxel = self.get_voxel(x, y, z);
                res.push(self.compress_data(x, y, z, Face::Bottom, voxel));
                res.push(self.compress_data(x+1, y, z, Face::Bottom, voxel));
                res.push(self.compress_data(x+1, y, z+1, Face::Bottom, voxel));
                res.push(self.compress_data(x+1, y, z+1, Face::Bottom, voxel));
                res.push(self.compress_data(x, y, z+1, Face::Bottom, voxel));
                res.push(self.compress_data(x, y, z, Face::Bottom, voxel));
                row >>= r; row >>= 1; y += 1;
            }
            res
        }));

        vertex_data.extend(mask_2.iter().enumerate().flat_map(|(i, &(mut row))| {
            let mut res = Vec::<u32>::new();
            let (mut x, mut y, mut z); x = 0;
            while row > 0 {
                let r = row.trailing_zeros();
                x += r as i32;
                y = (i & 0x1F) as i32;
                z = (i >> 5) as i32;
                let voxel = self.get_voxel(x, y, z);
                res.push(self.compress_data(x+1, y, z+1, Face::Right, voxel));
                res.push(self.compress_data(x+1, y, z, Face::Right, voxel));
                res.push(self.compress_data(x+1, y+1, z, Face::Right, voxel));
                res.push(self.compress_data(x+1, y+1, z, Face::Right, voxel));
                res.push(self.compress_data(x+1, y+1, z+1, Face::Right, voxel));
                res.push(self.compress_data(x+1, y, z+1, Face::Right, voxel));
                row >>= r; row >>= 1; x+=1;
            }
            res
        }));

        vertex_data.extend(mask_3.iter().enumerate().flat_map(|(i, &(mut row))| {
            let mut res = Vec::<u32>::new();
            let (mut x, mut y, mut z); x = 0;
            while row > 0 {
                let r = row.trailing_zeros();
                x += r as i32;
                y = (i & 0x1F) as i32;
                z = (i >> 5) as i32;
                let voxel = self.get_voxel(x, y, z);
                res.push(self.compress_data(x, y, z, Face::Left, voxel));
                res.push(self.compress_data(x, y, z+1, Face::Left, voxel));
                res.push(self.compress_data(x, y+1, z+1, Face::Left, voxel));
                res.push(self.compress_data(x, y+1, z+1, Face::Left, voxel));
                res.push(self.compress_data(x, y+1, z, Face::Left, voxel));
                res.push(self.compress_data(x, y, z, Face::Left, voxel));
                row >>= r; row >>= 1; x+=1;
            }
            res
        }));

        vertex_data.extend(mask_4.iter().enumerate().flat_map(|(i, &(mut row))| {
            let mut res = Vec::<u32>::new();
            let (mut x, mut y, mut z); z = 0;
            while row > 0 {
                let r = row.trailing_zeros();
                x = (i & 0x1F) as i32;
                y = (i >> 5) as i32;
                z += r as i32;
                let voxel = self.get_voxel(x, y, z);
                res.push(self.compress_data(x, y, z+1, Face::Front, voxel));
                res.push(self.compress_data(x+1, y, z+1, Face::Front, voxel));
                res.push(self.compress_data(x+1, y+1, z+1, Face::Front, voxel));
                res.push(self.compress_data(x+1, y+1, z+1, Face::Front, voxel));
                res.push(self.compress_data(x, y+1, z+1, Face::Front, voxel));
                res.push(self.compress_data(x, y, z+1, Face::Front, voxel));
                row >>= r; row >>= 1; z+=1;
            }
            res
        }));

        vertex_data.extend(mask_5.iter().enumerate().flat_map(|(i, &(mut row))| {
            let mut res = Vec::<u32>::new();
            let (mut x, mut y, mut z); z = 0;
            while row > 0 {
                let r = row.trailing_zeros();
                x = (i & 0x1F) as i32;
                y = (i >> 5) as i32;
                z += row.trailing_zeros() as i32;
                let voxel = self.get_voxel(x, y, z);
                res.push(self.compress_data(x+1, y, z, Face::Back, voxel));
                res.push(self.compress_data(x, y, z, Face::Back, voxel));
                res.push(self.compress_data(x, y+1, z, Face::Back, voxel));
                res.push(self.compress_data(x, y+1, z, Face::Back, voxel));
                res.push(self.compress_data(x+1, y+1, z, Face::Back, voxel));
                res.push(self.compress_data(x+1, y, z, Face::Back, voxel));
                row >>= r; row >>= 1; z+=1;
            }
            res
        }));
        
        ChunkMeshData::new(self.pos, vertex_data)
    }


    pub fn compress_data(&self, x:i32, y:i32, z:i32, face:Face, voxel_id:VOXELS) -> u32 {
        static COORD_STRIDE:u8 = 6;
        //static FACE_ID_STRIDE:u8 = 3;
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
}


pub struct ChunkMeshData {
    pub pos:(i32, i32, i32),
    pub vertices:Vec<u32>
}

impl ChunkMeshData {
    pub fn new(pos: (i32, i32, i32), vertices:Vec<u32>) -> Self {
        ChunkMeshData{pos, vertices}
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

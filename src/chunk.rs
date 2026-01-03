use std::collections::HashMap;
use std::default;
use std::ffi::c_void;
use crate::{math, util};
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


    pub fn get_vertex_data_optimized(&self, chunk_cluster: ChunkCluster) -> ChunkMeshData {
        let mut mesh = ChunkMeshData::default();
        let mut solid_mask = vec![0u64; 3 * CHUNK_AREA as usize];
        let mut culled_solid_mask = vec![0u64; 6 * CHUNK_AREA as usize];
        let mut greedy_meshing_planes: [HashMap<VOXELS, HashMap<i32, [u32; 32]>>; 6];
        greedy_meshing_planes = [
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new()
        ];

        for z in -1..=CHUNK_SIZE {
            for y in -1..=CHUNK_SIZE {
                for x in -1..=CHUNK_SIZE {
                    if chunk_cluster.get_voxel(x, y, z) != VOXELS::EMPTY {
                        solid_mask[(x+z*CHUNK_SIZE) as usize] |= 1<<y;
                        solid_mask[(y+z*CHUNK_SIZE + CHUNK_AREA) as usize] |= 1<<x;
                        solid_mask[(x+z*CHUNK_SIZE + 2*CHUNK_AREA) as usize] |= 1<<z;
                    }
                }
            }
        }

        for (i, &row) in solid_mask.iter().enumerate() {
            let cullr  = (row & (!row << 1)) >> 1 & 0xFF;
            let culll  = (row & (!row >> 1)) >> 1 & 0xFF;
            culled_solid_mask[2*i] = culll;
            culled_solid_mask[2*i+1] = cullr;
        }

        for face in Face::iter() {
            for a in 0..CHUNK_SIZE {
                for b in 0..CHUNK_SIZE {
                    let mut row = culled_solid_mask[(
                        (a + b * CHUNK_SIZE) * 2 
                        + face as i32 & 1 
                        + (face as i32 / 3) * CHUNK_AREA
                    ) as usize];

                    while row > 0 {
                        let c = row.trailing_zeros() as i32;

                        let (x, y, z) = match face {
                            Face::Top | Face::Bottom => (a, c, b),
                            Face::Right | Face::Left => (c, a, b),
                            Face::Front | Face::Back => (a, c, b)
                        };

                        let voxel_id = self.get_voxel(x,y,z);
                        row = row & (row-1);
                        let plane = greedy_meshing_planes[face as usize]
                            .entry(voxel_id)
                            .or_default()
                            .entry(c)
                            .or_default();
                        plane[a as usize] |= 1 << b as u32;
                    }
                }
            }
        }

        for face in Face::iter() {
            for (&voxel_id, planes) in &greedy_meshing_planes[face as usize] {
                for (&axis_pos, plane) in planes {
                    let new_data = Chunk::greedy_mesh_plane(plane, axis_pos, voxel_id, face);
                    mesh.vertices.extend(new_data);
                }
            }
        }

        mesh
    }


    fn greedy_mesh_plane(
        plane: &mut [u32], 
        depth: i32, 
        voxel_id: VOXELS, 
        face_id: Face
    ) -> Vec<u32> {
        let mut vertices = Vec::<u32>::new();
        let mut quads = Vec::<[u32; 4]>::new();
        for i0 in 0..plane.len() {
            let row = plane[i0];
            if row == 0 {continue;}
            let b0 = row.trailing_zeros();
            let mut b1 = b0 + (row >> b0).trailing_ones();
            let mut mask = u32::checked_shl(1, b1).map_or(!0u32, |n| n-1);
            let mut i1 = i0;
            while mask > 0 {
                mask &= plane[i1];
                b1 = b0 + (mask >> b0).trailing_zeros();
                i1 += 1;
                quads.push([i0 as u32, i1 as u32, b0 as u32, b1 as u32])
            }

            for i in i0..i1 {
                plane[i] &= !mask;
            }

        }

        vertices
    }


    pub fn get_vertex_data(&self, chunk_cluster:ChunkCluster) -> ChunkMeshData {
        let mut vertex_data: Vec<u32> = Vec::new();
        if self.status == ChunkStatus::Empty {
            return ChunkMeshData::new(self.pos, vertex_data);
        }
         
        let mut entities = vec![Vec::<u32>::new(); NUM_ENTITIES];
        let mut masks: Vec<Vec<u64>> = vec![
            vec![0;CHUNK_AREA as usize],
            vec![0;CHUNK_AREA as usize],
            vec![0;CHUNK_AREA as usize],
            vec![0;CHUNK_AREA as usize],
            vec![0;CHUNK_AREA as usize],
            vec![0;CHUNK_AREA as usize]
        ];

        (0..CHUNK_SIZE).flat_map(|x| {
            (0..CHUNK_SIZE).flat_map(move |y| {
                (0..CHUNK_SIZE).map(move |z| {
                    (x,y,z)
                })
            })
        }).for_each(|(x, y, z)| {
            let voxel_id = self.get_voxel(x, y, z);
            if voxel_id == VOXELS::EMPTY {return}
            entities[voxel_id as usize].push(self.compress_position((x,y,z)));
        });


        VOXELS::iter().map(|voxel_id| {
            (voxel_id, entities[voxel_id as usize].iter())
        }).for_each(|(voxel_id, data)| {

            self.generate_masks(&mut masks, &chunk_cluster, voxel_id, data);

            for i in [0, 2, 4] {
                for row in masks[i].iter_mut() {
                    *row = (!*row >> 1) & *row;
                    *row >>= 1;
                    *row &= 0xFFFFFFFF;
                }
            }

            for i in [1, 3, 5] {
                for row in masks[i].iter_mut() {
                    *row = (!*row << 1) & *row;
                    *row >>= 1;
                    *row &= 0xFFFFFFFF;
                }
            }

            vertex_data.extend(masks[0].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); y = 0;
                while *row > 0 {
                    x = (i & 0x1F) as i32;
                    let r = row.trailing_zeros();
                    y += r as i32;
                    z = (i >> 5) as i32;
                    res.push(self.compress_data(x, y+1, z+1, Face::Top, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z+1, Face::Top, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z, Face::Top, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z, Face::Top, voxel_id));
                    res.push(self.compress_data(x, y+1, z, Face::Top, voxel_id));
                    res.push(self.compress_data(x, y+1, z+1, Face::Top, voxel_id));
                    *row >>= r; *row >>= 1; y += 1;
                }
                res
            }));

            vertex_data.extend(masks[1].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); y = 0;
                while *row > 0 {
                    x = (i & 0x1F) as i32;
                    let r = row.trailing_zeros();
                    y += r as i32;
                    z = (i >> 5) as i32;
                    res.push(self.compress_data(x, y, z, Face::Bottom, voxel_id));
                    res.push(self.compress_data(x+1, y, z, Face::Bottom, voxel_id));
                    res.push(self.compress_data(x+1, y, z+1, Face::Bottom, voxel_id));
                    res.push(self.compress_data(x+1, y, z+1, Face::Bottom, voxel_id));
                    res.push(self.compress_data(x, y, z+1, Face::Bottom, voxel_id));
                    res.push(self.compress_data(x, y, z, Face::Bottom, voxel_id));
                    *row >>= r; *row >>= 1; y += 1;
                }
                res
            }));

            vertex_data.extend(masks[2].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); x = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x += r as i32;
                    y = (i & 0x1F) as i32;
                    z = (i >> 5) as i32;
                    res.push(self.compress_data(x+1, y, z+1, Face::Right, voxel_id));
                    res.push(self.compress_data(x+1, y, z, Face::Right, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z, Face::Right, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z, Face::Right, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z+1, Face::Right, voxel_id));
                    res.push(self.compress_data(x+1, y, z+1, Face::Right, voxel_id));
                    *row >>= r; *row >>= 1; x+=1;
                }
                res
            }));

            vertex_data.extend(masks[3].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); x = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x += r as i32;
                    y = (i & 0x1F) as i32;
                    z = (i >> 5) as i32;
                    res.push(self.compress_data(x, y, z, Face::Left, voxel_id));
                    res.push(self.compress_data(x, y, z+1, Face::Left, voxel_id));
                    res.push(self.compress_data(x, y+1, z+1, Face::Left, voxel_id));
                    res.push(self.compress_data(x, y+1, z+1, Face::Left, voxel_id));
                    res.push(self.compress_data(x, y+1, z, Face::Left, voxel_id));
                    res.push(self.compress_data(x, y, z, Face::Left, voxel_id));
                    *row >>= r; *row >>= 1; x+=1;
                }
                res
            }));

            vertex_data.extend(masks[4].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); z = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x = (i & 0x1F) as i32;
                    y = (i >> 5) as i32;
                    z += r as i32;
                    res.push(self.compress_data(x, y, z+1, Face::Front, voxel_id));
                    res.push(self.compress_data(x+1, y, z+1, Face::Front, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z+1, Face::Front, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z+1, Face::Front, voxel_id));
                    res.push(self.compress_data(x, y+1, z+1, Face::Front, voxel_id));
                    res.push(self.compress_data(x, y, z+1, Face::Front, voxel_id));
                    *row >>= r; *row >>= 1; z+=1;
                }
                res
            }));

            vertex_data.extend(masks[5].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); z = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x = (i & 0x1F) as i32;
                    y = (i >> 5) as i32;
                    z += row.trailing_zeros() as i32;
                    res.push(self.compress_data(x+1, y, z, Face::Back, voxel_id));
                    res.push(self.compress_data(x, y, z, Face::Back, voxel_id));
                    res.push(self.compress_data(x, y+1, z, Face::Back, voxel_id));
                    res.push(self.compress_data(x, y+1, z, Face::Back, voxel_id));
                    res.push(self.compress_data(x+1, y+1, z, Face::Back, voxel_id));
                    res.push(self.compress_data(x+1, y, z, Face::Back, voxel_id));
                    *row >>= r; *row >>= 1; z+=1;
                }
                res
            }));
        });

        //face id order
        
        ChunkMeshData::new(self.pos, vertex_data)
    }

    fn generate_masks<'a>(
        &self, 
        masks: &mut Vec<Vec<u64>>, 
        chunk_cluster: &ChunkCluster, 
        voxel_id: VOXELS, 
        data: impl Iterator<Item=&'a u32>
    ) {
        Face::iter().flat_map(|f| {
            (0..CHUNK_SIZE).flat_map(move |a| {
                (0..CHUNK_SIZE).map(move |b| {
                    (f,a,b)
                })
            })
        }).for_each(|(f,a,b)| {
            match f {
                Face::Top => {
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= (chunk_cluster.get_voxel(a,-1,b) == voxel_id) as u64;
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= ((chunk_cluster.get_voxel(a,CHUNK_SIZE,b) == voxel_id) as u64) << 33;
                },
                Face::Bottom => {
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= (chunk_cluster.get_voxel(a,-1,b) == voxel_id) as u64;
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= ((chunk_cluster.get_voxel(a,CHUNK_SIZE,b) == voxel_id) as u64) << 33;
                }
                Face::Right => {
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= (chunk_cluster.get_voxel(-1,a,b) == voxel_id) as u64;
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= ((chunk_cluster.get_voxel(CHUNK_SIZE,a,b) == voxel_id) as u64) << 33;
                }
                Face::Left => {
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= (chunk_cluster.get_voxel(-1,a,b) == voxel_id) as u64;
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= ((chunk_cluster.get_voxel(CHUNK_SIZE,a,b) == voxel_id) as u64) << 33;
                }
                Face::Front => {
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= (chunk_cluster.get_voxel(a,b,-1) == voxel_id) as u64;
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= ((chunk_cluster.get_voxel(a,b,CHUNK_SIZE) == voxel_id) as u64) << 33;
                }
                Face::Back => {
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= (chunk_cluster.get_voxel(a,b,-1) == voxel_id) as u64;
                    masks[f as usize][(a + b * CHUNK_SIZE) as usize] 
                        |= ((chunk_cluster.get_voxel(a,b,CHUNK_SIZE) == voxel_id) as u64) << 33;
                }
            }
        });

        data.for_each(|&pos| {
            let (x, y, z) = (
                (pos & 0xFF) as i32, 
                ((pos & 0xFF00) >> 8) as i32, 
                ((pos & 0xFF0000) >> 16) as i32
            );
            masks[0][(x + z * CHUNK_SIZE) as usize] |= 1 << y+1;
            masks[1][(x + z * CHUNK_SIZE) as usize] |= 1 << y+1;
            masks[2][(y + z * CHUNK_SIZE) as usize] |= 1 << x+1;
            masks[3][(y + z * CHUNK_SIZE) as usize] |= 1 << x+1;
            masks[4][(x + y * CHUNK_SIZE) as usize] |= 1 << z+1;
            masks[5][(x + y * CHUNK_SIZE) as usize] |= 1 << z+1;
        });
    }

    fn build_greedy_mesh_for_entity(&self, voxel_id: VOXELS, masks: Vec<Vec<u64>>) {
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


    fn compress_position(&self, (x, y, z):(i32, i32, i32)) -> u32 {
        x as u32 | (y as u32) << 8 | (z as u32) << 16
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


#[derive(Default)]
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

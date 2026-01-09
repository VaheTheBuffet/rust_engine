use std::collections::HashMap;
use crate::shader_program::{GlobalShaderProgram, UniformFn};
use crate::{math};
use crate::util::Noise;
use crate::world::{ChunkCluster, Face};
use crate::{settings::*};
use crate::renderer::{self, VertexArray};

pub struct Chunk<T: renderer::VertexArray> {
    pub pos: (i32, i32, i32),
    pub center: (f32, f32, f32),
    pub voxels: Vec<VOXELS>,
    pub vertex_count: i32,
    pub status: ChunkStatus,
    pub vertex_array: Option<T>
}

impl<T: VertexArray> Chunk<T> {
    pub fn new(x:i32, y:i32, z:i32) -> Chunk<T> {
        Chunk{
            pos:(x, y, z), 
            center: (
                (x * CHUNK_SIZE + H_CHUNK_SIZE) as f32, 
                (y * CHUNK_SIZE + H_CHUNK_SIZE) as f32, 
                (z * CHUNK_SIZE + H_CHUNK_SIZE) as f32
            ),
            voxels:vec![VOXELS::EMPTY;CHUNK_VOL as usize], 
            vertex_count:0, 
            status:ChunkStatus::Empty,
            vertex_array:None
        }
    }


    #[cfg(debug_assertions)]
    pub fn filled(x:i32, y:i32, z:i32) -> Chunk<T> {
        let voxels = vec![VOXELS::WOOD;CHUNK_VOL as usize];
        Chunk{
            pos:(x, y, z), 
            center: (
                (x * CHUNK_SIZE + H_CHUNK_SIZE) as f32, 
                (y * CHUNK_SIZE + H_CHUNK_SIZE) as f32, 
                (z * CHUNK_SIZE + H_CHUNK_SIZE) as f32
            ),
            voxels,
            vertex_count:0, 
            status:ChunkStatus::Dirty,
            vertex_array:None
        }
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
        if let Some(v) = &self.vertex_array {
            v.bind();
            v.draw(0, self.vertex_count);
            v.unbind();
        }
    }


    pub fn update_uniforms(&self, program: &GlobalShaderProgram) {
        program.programs[0].set_uniform("m_model\0", math::get_model(self.pos));
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
                    self.set_voxel(x, y+1, z, VOXELS::WATER)?;
                    self.set_voxel(x, y+2, z, VOXELS::WATER)?;
                    self.set_voxel(x, y+3, z, VOXELS::WATER)?;
                    self.set_voxel(x, y+4, z, VOXELS::WATER)?;
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


    pub fn get_mesh(&self, chunk_cluster: ChunkCluster) -> ChunkMesh {
        let mut general_mesh = self.get_vertices_greedy(
            self.build_masks(&chunk_cluster, |v| v != VOXELS::EMPTY && v != VOXELS::WATER)
        );
        let water_mesh = self.get_vertices_water(
            self.build_masks(&chunk_cluster, |v| v == VOXELS::WATER)
        );

        general_mesh.vertices.extend(water_mesh.vertices);
        general_mesh
    }


    fn build_masks(
        &self, 
        chunk_cluster: &ChunkCluster, 
        condition: impl Fn(VOXELS) -> bool
    ) -> [u64; 3*CHUNK_AREA as usize] {
        let mut solid_mask = [0u64; 3*CHUNK_AREA as usize];

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if condition(self.get_voxel(x,y,z)) {
                        solid_mask[(x+z*CHUNK_SIZE) as usize] |= 1<<y+1;
                        solid_mask[(y+z*CHUNK_SIZE + CHUNK_AREA) as usize] |= 1<<x+1;
                        solid_mask[(x+y*CHUNK_SIZE + 2*CHUNK_AREA) as usize] |= 1<<z+1;
                    }
                }
            }
        }

        for b in 0..CHUNK_SIZE {
            for a in 0..CHUNK_SIZE {
                if condition(chunk_cluster.get_voxel(a, -1, b)) {
                    solid_mask[(a+b*CHUNK_SIZE) as usize] |= 1;
                }
                if condition(chunk_cluster.get_voxel(a, CHUNK_SIZE, b)) {
                    solid_mask[(a+b*CHUNK_SIZE) as usize] |= 1<<33;
                }
            }
        }

        for b in 0..CHUNK_SIZE {
            for a in 0..CHUNK_SIZE {
                if condition(chunk_cluster.get_voxel(-1, a, b)) {
                    solid_mask[(a+b*CHUNK_SIZE + CHUNK_AREA) as usize] |= 1;
                }
                if condition(chunk_cluster.get_voxel(CHUNK_SIZE, a, b)) {
                    solid_mask[(a+b*CHUNK_SIZE + CHUNK_AREA) as usize] |= 1<<33;
                }
            }
        }

        for b in 0..CHUNK_SIZE {
            for a in 0..CHUNK_SIZE {
                if condition(chunk_cluster.get_voxel(a, b, -1)) {
                    solid_mask[(a+b*CHUNK_SIZE + 2*CHUNK_AREA) as usize] |= 1;
                }
                if condition(chunk_cluster.get_voxel(a, b, CHUNK_SIZE)) {
                    solid_mask[(a+b*CHUNK_SIZE + 2*CHUNK_AREA) as usize] |= 1<<33;
                }
            }
        }

        solid_mask
    }


    fn get_vertices_water(&self, water_mask: [u64; 3* CHUNK_AREA as usize]) -> ChunkMesh {
        let mut mesh = ChunkMesh::default();
        let mut culled_solid_mask = [0u64; 2*CHUNK_AREA as usize];
        let mut greedy_meshing_planes: HashMap<u32, [u32; 32]>;
        greedy_meshing_planes = HashMap::new();

        for i in 0..CHUNK_AREA as usize {
            //cull row and remove outer bits
            let row = water_mask[i];
            let culll  = (row & !(row >> 1)) >> 1 & 0xFFFFFFFF;
            culled_solid_mask[2*i] = culll;
        }

        for a in 0..CHUNK_SIZE {
            for b in 0..CHUNK_SIZE {
                let mut row = culled_solid_mask[(
                    (a + b * CHUNK_SIZE) * 2 
                    + (Face::Top as i32 & 1)
                    + ((Face::Top as i32) >> 1) * CHUNK_AREA * 2
                ) as usize];

                while row > 0 {
                    let c = row.trailing_zeros() as i32;

                    row = row & (row-1);
                    let plane = greedy_meshing_planes
                        .entry(c as u32)
                        .or_default();
                    plane[a as usize] |= 1 << b as u32;
                }
            }
        }

        for (&axis_pos, plane) in greedy_meshing_planes.iter_mut() {
            let new_data = Chunk::<T>::greedy_mesh_plane(plane, axis_pos, VOXELS::WATER, Face::Top);
            mesh.vertices.extend(new_data);
        }

        mesh.pos = self.pos;
        mesh
    }


    pub fn get_vertices_greedy(&self, solid_mask: [u64; 3*CHUNK_AREA as usize]) -> ChunkMesh {
        let mut mesh = ChunkMesh::default();
        let mut culled_solid_mask = [0u64; 6 * CHUNK_AREA as usize];
        let mut greedy_meshing_planes: [HashMap<VOXELS, HashMap<u32, [u32; 32]>>; 6];
        greedy_meshing_planes = [
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new()
        ];

        for (i, &row) in solid_mask.iter().enumerate() {
            //cull row and remove outer bits
            let cullr  = (row & !(row << 1)) >> 1 & 0xFFFFFFFF;
            let culll  = (row & !(row >> 1)) >> 1 & 0xFFFFFFFF;
            culled_solid_mask[2*i] = culll;
            culled_solid_mask[2*i+1] = cullr;
        }

        for face in Face::iter() {
            for a in 0..CHUNK_SIZE {
                for b in 0..CHUNK_SIZE {
                    let mut row = culled_solid_mask[(
                        (a + b * CHUNK_SIZE) * 2 
                        + (face as i32 & 1)
                        + ((face as i32) >> 1) * CHUNK_AREA * 2
                    ) as usize];

                    while row > 0 {
                        let c = row.trailing_zeros() as i32;

                        let (x, y, z) = match face {
                            Face::Top | Face::Bottom => (a, c, b),
                            Face::Right | Face::Left => (c, a, b),
                            Face::Front | Face::Back => (a, b, c)
                        };

                        let voxel_id = self.get_voxel(x,y,z);
                        row = row & (row-1);
                        let plane = greedy_meshing_planes[face as usize]
                            .entry(voxel_id)
                            .or_default()
                            .entry(c as u32)
                            .or_default();
                        plane[a as usize] |= 1 << b as u32;
                    }
                }
            }
        }

        for face in Face::iter() {
            for (&voxel_id, planes) in &mut greedy_meshing_planes[face as usize] {
                for (&axis_pos, plane) in planes {
                    let new_data = Chunk::<T>::greedy_mesh_plane(plane, axis_pos, voxel_id, face);
                    mesh.vertices.extend(new_data);
                }
            }
        }

        mesh.pos = self.pos;
        mesh
    }


    fn greedy_mesh_plane(
        plane: &mut [u32; 32], 
        depth: u32, 
        voxel_id: VOXELS, 
        face: Face
    ) -> Vec<u32> {
        let mut vertices = Vec::<u32>::new();
        let mut quads = Vec::<[u32; 4]>::new();
        for x in 0..plane.len() {
            let mut y = plane[x].trailing_zeros();
            while y < 32 {
                let height:u32 = (plane[x] >> y).trailing_ones();
                let mask:u32 = u32::checked_shl(1, height).map_or(!0u32, |n| n-1) << y;

                plane[x] &= !mask;
                let mut width = 1;
                while mask > 0 && x + width < 32 && mask == plane[x+width] {
                    plane[x + width] &= !mask;
                    width += 1;
                }
                quads.push([x as u32, (x + width) as u32, y as u32, (y + height) as u32]);
                y = plane[x].trailing_zeros();
            }
        }

        match face {
            Face::Top => {
                for [u0, u1, v0, v1] in quads {
                    vertices.extend([
                        Chunk::<T>::compress_data(u0, depth+1, v1, face, voxel_id), 
                        Chunk::<T>::compress_data(u1, depth+1, v1, face, voxel_id),
                        Chunk::<T>::compress_data(u1, depth+1, v0, face, voxel_id),
                        Chunk::<T>::compress_data(u1, depth+1, v0, face, voxel_id),
                        Chunk::<T>::compress_data(u0, depth+1, v0, face, voxel_id),
                        Chunk::<T>::compress_data(u0, depth+1, v1, face, voxel_id)
                    ])
                }
            }
            Face::Bottom => {
                for [u0, u1, v0, v1] in quads {
                    vertices.extend([
                        Chunk::<T>::compress_data(u0, depth, v0, face, voxel_id),
                        Chunk::<T>::compress_data(u1, depth, v0, face, voxel_id),
                        Chunk::<T>::compress_data(u1, depth, v1, face, voxel_id),
                        Chunk::<T>::compress_data(u1, depth, v1, face, voxel_id),
                        Chunk::<T>::compress_data(u0, depth, v1, face, voxel_id),
                        Chunk::<T>::compress_data(u0, depth, v0, face, voxel_id)
                    ])
                }
           }
            Face::Right => {
                for [u0, u1, v0, v1] in quads {
                    vertices.extend([
                        Chunk::<T>::compress_data(depth+1, u0, v1, face, voxel_id),
                        Chunk::<T>::compress_data(depth+1, u0, v0, face, voxel_id),
                        Chunk::<T>::compress_data(depth+1, u1, v0, face, voxel_id),
                        Chunk::<T>::compress_data(depth+1, u1, v0, face, voxel_id),
                        Chunk::<T>::compress_data(depth+1, u1, v1, face, voxel_id),
                        Chunk::<T>::compress_data(depth+1, u0, v1, face, voxel_id),
                    ])
                }
            }
            Face::Left => {
                for [u0, u1, v0, v1] in quads {
                    vertices.extend([
                        Chunk::<T>::compress_data(depth, u0, v0, face, voxel_id),
                        Chunk::<T>::compress_data(depth, u0, v1, face, voxel_id),
                        Chunk::<T>::compress_data(depth, u1, v1, face, voxel_id),
                        Chunk::<T>::compress_data(depth, u1, v1, face, voxel_id),
                        Chunk::<T>::compress_data(depth, u1, v0, face, voxel_id),
                        Chunk::<T>::compress_data(depth, u0, v0, face, voxel_id)
                    ])
                }
            }
            Face::Front => {
                for [u0, u1, v0, v1] in quads {
                    vertices.extend([
                        Chunk::<T>::compress_data(u0, v0, depth+1, face, voxel_id),
                        Chunk::<T>::compress_data(u1, v0, depth+1, face, voxel_id),
                        Chunk::<T>::compress_data(u1, v1, depth+1, face, voxel_id),
                        Chunk::<T>::compress_data(u1, v1, depth+1, face, voxel_id),
                        Chunk::<T>::compress_data(u0, v1, depth+1, face, voxel_id),
                        Chunk::<T>::compress_data(u0, v0, depth+1, face, voxel_id)
                    ])
                }
            }
            Face::Back => {
                for [u0, u1, v0, v1] in quads {
                    vertices.extend([
                        Chunk::<T>::compress_data(u1, v0, depth, face, voxel_id),
                        Chunk::<T>::compress_data(u0, v0, depth, face, voxel_id),
                        Chunk::<T>::compress_data(u0, v1, depth, face, voxel_id),
                        Chunk::<T>::compress_data(u0, v1, depth, face, voxel_id),
                        Chunk::<T>::compress_data(u1, v1, depth, face, voxel_id),
                        Chunk::<T>::compress_data(u1, v0, depth, face, voxel_id),
                    ])
                }
            }
        }
        vertices
    }


    pub fn get_vertex_data(&self, chunk_cluster:ChunkCluster) -> ChunkMesh {
        let mut mesh = ChunkMesh::default();
        if self.status == ChunkStatus::Empty {
            return mesh;
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

        fn compress_position((x, y, z):(i32, i32, i32)) -> u32 {
            x as u32 | (y as u32) << 8 | (z as u32) << 16
        }

        (0..CHUNK_SIZE).flat_map(|x| {
            (0..CHUNK_SIZE).flat_map(move |y| {
                (0..CHUNK_SIZE).map(move |z| {
                    (x,y,z)
                })
            })
        }).for_each(|(x, y, z)| {
            let voxel_id = self.get_voxel(x, y, z);
            if voxel_id == VOXELS::EMPTY {return}
            entities[voxel_id as usize].push(compress_position((x,y,z)));
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

            mesh.vertices.extend(masks[0].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); y = 0;
                while *row > 0 {
                    x = i as u32 & 0x1F;
                    let r = row.trailing_zeros();
                    y += r;
                    z = i as u32 >> 5;
                    res.push(Chunk::<T>::compress_data(x, y+1, z+1, Face::Top, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z+1, Face::Top, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z, Face::Top, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z, Face::Top, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z, Face::Top, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z+1, Face::Top, voxel_id));
                    *row >>= r; *row >>= 1; y += 1;
                }
                res
            }));

            mesh.vertices.extend(masks[1].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); y = 0;
                while *row > 0 {
                    x = i as u32 & 0x1F;
                    let r = row.trailing_zeros();
                    y += r;
                    z = i as u32 >> 5;
                    res.push(Chunk::<T>::compress_data(x, y, z, Face::Bottom, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z, Face::Bottom, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z+1, Face::Bottom, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z+1, Face::Bottom, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y, z+1, Face::Bottom, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y, z, Face::Bottom, voxel_id));
                    *row >>= r; *row >>= 1; y += 1;
                }
                res
            }));

            mesh.vertices.extend(masks[2].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); x = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x += r;
                    y = i as u32 & 0x1F;
                    z = i as u32 >> 5;
                    res.push(Chunk::<T>::compress_data(x+1, y, z+1, Face::Right, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z, Face::Right, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z, Face::Right, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z, Face::Right, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z+1, Face::Right, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z+1, Face::Right, voxel_id));
                    *row >>= r; *row >>= 1; x+=1;
                }
                res
            }));

            mesh.vertices.extend(masks[3].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); x = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x += r;
                    y = i as u32 & 0x1F;
                    z = i as u32 >> 5;
                    res.push(Chunk::<T>::compress_data(x, y, z, Face::Left, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y, z+1, Face::Left, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z+1, Face::Left, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z+1, Face::Left, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z, Face::Left, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y, z, Face::Left, voxel_id));
                    *row >>= r; *row >>= 1; x+=1;
                }
                res
            }));

            mesh.vertices.extend(masks[4].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); z = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x = i as u32 & 0x1F;
                    y = i as u32 >> 5;
                    z += r;
                    res.push(Chunk::<T>::compress_data(x, y, z+1, Face::Front, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z+1, Face::Front, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z+1, Face::Front, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z+1, Face::Front, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z+1, Face::Front, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y, z+1, Face::Front, voxel_id));
                    *row >>= r; *row >>= 1; z+=1;
                }
                res
            }));

            mesh.vertices.extend(masks[5].iter_mut().enumerate().flat_map(|(i, row)| {
                let mut res = Vec::<u32>::new();
                let (mut x, mut y, mut z); z = 0;
                while *row > 0 {
                    let r = row.trailing_zeros();
                    x = i as u32 & 0x1F;
                    y = i as u32 >> 5;
                    z += r;
                    res.push(Chunk::<T>::compress_data(x+1, y, z, Face::Back, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y, z, Face::Back, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z, Face::Back, voxel_id));
                    res.push(Chunk::<T>::compress_data(x, y+1, z, Face::Back, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y+1, z, Face::Back, voxel_id));
                    res.push(Chunk::<T>::compress_data(x+1, y, z, Face::Back, voxel_id));
                    *row >>= r; *row >>= 1; z+=1;
                }
                res
            }));
        });

        //face id order
        
        mesh.pos = self.pos;
        mesh
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


    pub fn compress_data(x:u32, y:u32, z:u32, face:Face, voxel_id:VOXELS) -> u32 {
        static COORD_STRIDE:u32 = 6;
        //static FACE_ID_STRIDE:usize = 3;
        static VOXEL_ID_STRIDE:u32 = 4;

        let mut res = 0;

        res |= face as u32;
        res <<= COORD_STRIDE; res |= x; 
        res <<= COORD_STRIDE; res |= y; 
        res <<= COORD_STRIDE; res |= z; 
        res <<= VOXEL_ID_STRIDE; res |= voxel_id as u32;

        res as u32
    }



}

impl Chunk<renderer::GLVertexArray> {
    pub fn build_mesh(&mut self, chunk_mesh_data: ChunkMesh) {
        let (len, vertex_data) = (chunk_mesh_data.vertices.len(), chunk_mesh_data.vertices);
        self.vertex_count = len as i32;
        self.status = if len > 0 {ChunkStatus::Clean} else {ChunkStatus::Empty};

        let mut layout = renderer::BufferLayout::default();
        layout.add(renderer::BufferElement{
            element_type: renderer::BufferElementType::I32,
            quantity: 1,
            normalized: false
        });

        let buf = renderer::VertexBuffer(vertex_data);
        self.vertex_array = Some(renderer::GLVertexArray::new(buf, layout));
    }
}


#[derive(Default)]
pub struct ChunkMesh {
    pub pos:(i32, i32, i32),
    pub vertices:Vec<u32>,
    //pub vertices_water:Vec<u32>
}

impl ChunkMesh {
    pub fn new(
        pos: (i32, i32, i32), 
        vertices: Vec<u32>, 
        //vertices_water: Vec<u32>
    ) -> ChunkMesh {
        ChunkMesh{pos, vertices}
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
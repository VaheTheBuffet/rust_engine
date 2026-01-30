use crate::*;
use std::{collections::HashMap};


pub struct World 
{
    pub chunks:HashMap<(i32,i32,i32), Arc<chunk::Chunk>>,
    pub render_range: Vec<Arc<chunk::Chunk>>,
    pub noise: util::Noise,
}

impl World {
    pub fn new() -> Self 
    {
        let noise = util::Noise::new(SEED);
        World{
            chunks:HashMap::new(),
            render_range: Vec::new(),
            noise}
    }


    pub fn chunk_build_task(&self, (x,y,z):(i32,i32,i32)) -> chunk::Chunk 
    {
        let mut chunk = chunk::Chunk::new(x, y, z);
        chunk.status = chunk::ChunkStatus::Dirty;
        chunk.build_voxels(&self.noise)
    }


    pub fn entity_build_task(&self, (x,y,z):(i32,i32,i32)) -> Vec<(i32,i32,i32,ENTITIES)> 
    {
        self.chunks.get(&(x,y,z)).unwrap().generate_entities()
    }


    pub fn mesh_build_task(&self, (x, y, z):(i32, i32, i32)) -> chunk::ChunkMesh 
    {
        let chunk_cluster = ChunkCluster::new(&self, x, y, z);
        self.chunks.get(&(x,y,z)).unwrap().get_mesh(chunk_cluster)
    }


    pub fn get_voxel(&self, global_x:i32, global_y:i32, global_z:i32) -> VOXELS
    {
        let cx = (global_x as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cy = (global_y as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cz = (global_z as f32 * INV_CHUNK_SIZE).floor() as i32;
        let lx = global_x % CHUNK_SIZE;
        let ly = global_y % CHUNK_SIZE;
        let lz = global_z % CHUNK_SIZE;

        self.chunks.get(&(cx,cy,cz)).unwrap().get_voxel(lx, ly, lz)
    }


    pub fn set_voxel(&mut self, global_x:i32, global_y:i32, global_z:i32, voxel:VOXELS) -> Result<(), ()> 
    {
        let cx = (global_x as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cy = (global_y as f32 * INV_CHUNK_SIZE).floor() as i32;
        let cz = (global_z as f32 * INV_CHUNK_SIZE).floor() as i32;
        let lx = global_x.rem_euclid(CHUNK_SIZE);
        let ly = global_y.rem_euclid(CHUNK_SIZE);
        let lz = global_z.rem_euclid(CHUNK_SIZE);

        let chunk = Arc::try_unwrap(self.chunks.remove(&(cx,cy,cz)).expect(&format!(
            "no chunk at {}, {}, {}\n", cx, cy, cz)));

        if let Ok(mut inner) = chunk {
            inner = inner.set_voxel(lx, ly, lz, voxel).unwrap();
            self.chunks.insert((cx, cy, cz), Arc::new(inner));
        }

        Ok(())
    }


    pub fn promote_chunks(&self, player:& camera::Player) -> (
        Vec<(i32, i32, i32)>, Vec<(i32, i32, i32)>, Vec<(i32, i32, i32)>
    ) {
        let (px, py, pz) = (player.chunk_x, player.chunk_y, player.chunk_z);
        let mut build_positions = Vec::<(i32, i32, i32)>::new();
        let mut dirty_positions = Vec::<(i32, i32, i32)>::new();
        let mut terrain_positions = Vec::<(i32, i32, i32)>::new();

        for pos in util::render_range((px, py, pz)) {
            if let Some(chunk) = self.chunks.get(&pos) {
                match chunk.status {
                    chunk::ChunkStatus::Dirty => {
                        dirty_positions.push(pos);
                    }
                    chunk::ChunkStatus::Terrain => {
                        terrain_positions.push(pos);
                    }
                    _ => {}
                }
            } else {
                build_positions.push(pos);
                dirty_positions.push(pos);
                terrain_positions.push(pos);
            }
        }

        //handle edge chunks
        build_positions.extend(util::border_range((px, py, pz))
            .filter(|&pos| {
                if let None = self.chunks.get(&pos) {true} else {false}})
            .collect::<Vec<_>>());
        (build_positions, terrain_positions, dirty_positions)
    }


    pub fn build_tree_at(&mut self, (x,y,z):(i32,i32,i32)) -> Result<(), ()> 
    {
        static HASH_LEN:usize = math::HASH.len();
        let key1 = (x*13+y*3+z*3) as usize % HASH_LEN;

        let height = math::HASH[key1].rem_euclid(4) + 4;
        for ty in 0..height 
        {
            self.set_voxel(x,y+ty,z, VOXELS::WOOD)?;
        }

        self.set_voxel(x, y+height, z, VOXELS::LEAF)?;

        for ty in 0..=3 
        {
            let key2 = (x*5+ty*11+z*13) as usize %HASH_LEN;
            let stride = (3-ty)+math::HASH[key2] % 3;
            for tx in -stride..=stride 
            {
                for tz in -stride..=stride 
                {
                    self.set_voxel(x+tx, y+height+ty, z+tz, VOXELS::LEAF)?;
                }
            }
        }

        Ok(())
    }


    pub fn decorate(&mut self, entities: Vec<Vec<(i32, i32, i32, ENTITIES)>>) -> Result<(), ()> 
    {
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

impl Face 
{
    pub fn id(&self) -> i32 
    {
        match self 
        {
            Face::Top => 0,
            Face::Bottom => 1,
            Face::Right => 2,
            Face::Left => 3,
            Face::Front => 4,
            Face::Back => 5
        }
    }

    pub fn offset(&self) -> (i32, i32, i32) 
    {
        match self 
        {
            Face::Top => (0,1,0),
            Face::Bottom => (0,-1,0),
            Face::Right => (1,0,0),
            Face::Left => (-1,0,0),
            Face::Front => (0,0,1),
            Face::Back => (0,0,-1)
        }
    }

    #[inline]
    pub fn iter() -> impl Iterator<Item = Face> 
    {
        [
            Face::Top, Face::Bottom, Face::Right, 
            Face::Left, Face::Front, Face::Back
        ].into_iter()
    }

    pub fn coords(&self) -> [[i32; 3]; 6] 
    {
        match self 
        {
            Face::Top => [[0,1,1], [1,1,1], [1,1,0], [1,1,0], [0,1,0], [0,1,1]],
            Face::Bottom => [[0,0,0], [1,0,0], [1,0,1], [1,0,1], [0,0,1], [0,0,0]],
            Face::Right => [[1,0,1], [1,0,0], [1,1,0], [1,1,0], [1,1,1], [1,0,1]],
            Face::Left =>[[0,0,0], [0,0,1], [0,1,1], [0,1,1], [0,1,0], [0,0,0]],
            Face::Front => [[0,0,1], [1,0,1], [1,1,1], [1,1,1], [0,1,1], [0,0,1]],
            Face::Back => [[1,0,0], [0,0,0], [0,1,0], [0,1,0], [1,1,0], [1,0,0]]
        }
    }
}


#[derive(Clone)]
pub struct ChunkCluster 
{
    pub center:Option<Arc<chunk::Chunk>>,
    top:Option<Arc<chunk::Chunk>>,
    bottom:Option<Arc<chunk::Chunk>>,
    right:Option<Arc<chunk::Chunk>>,
    left:Option<Arc<chunk::Chunk>>,
    front:Option<Arc<chunk::Chunk>>,
    back:Option<Arc<chunk::Chunk>>
}

impl ChunkCluster 
{
    pub fn new(world:&World, x:i32, y:i32, z:i32) -> Self 
    { 
        ChunkCluster 
        {
            center: Some(Arc::clone(world.chunks.get(&(x,y,z)).expect("Center must be valid"))),
            top:    world.chunks.get(&(x,y+1,z)).map(|chunk| Arc::clone(chunk)),
            bottom: world.chunks.get(&(x,y-1,z)).map(|chunk| Arc::clone(chunk)),
            right:  world.chunks.get(&(x+1,y,z)).map(|chunk| Arc::clone(chunk)),
            left:   world.chunks.get(&(x-1,y,z)).map(|chunk| Arc::clone(chunk)),
            front:  world.chunks.get(&(x,y,z+1)).map(|chunk| Arc::clone(chunk)),
            back:   world.chunks.get(&(x,y,z-1)).map(|chunk| Arc::clone(chunk)),
        }
    }


    pub fn get_voxel(&self, local_x:i32, local_y:i32, local_z:i32) -> VOXELS 
    {
        if let Some(chunk) = match (
            local_x.div_euclid(CHUNK_SIZE),
            local_y.div_euclid(CHUNK_SIZE),
            local_z.div_euclid(CHUNK_SIZE)
        ) {
            (0,0,0) => self.center.as_ref(),
            (0,1,0) => self.top.as_ref(),
            (0,-1,0) => self.bottom.as_ref(),
            (1,0,0) => self.right.as_ref(),
            (-1,0,0) => self.left.as_ref(),
            (0,0,1) => self.front.as_ref(),
            (0,0,-1) => self.back.as_ref(),
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


    pub fn is_solid(&self, local_x:i32, local_y:i32, local_z:i32) -> bool 
    {
        self.get_voxel(local_x, local_y, local_z) != VOXELS::EMPTY
    }


    pub fn is_face_visible(&self, x:i32, y:i32, z:i32, face:Face) -> bool 
    {
        let (dx, dy, dz) = face.offset();
        self.get_voxel(x+dx, y+dy, z+dz) == VOXELS::EMPTY
    }
}
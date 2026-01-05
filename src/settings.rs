pub const INV_ASPECT_RATIO:f32 = 9.0/16.0;
pub const WIDTH:u32 = 1280;
pub const HEIGHT:u32 = (INV_ASPECT_RATIO * (WIDTH as f32)) as u32;

pub const NEAR:f32 = 0.1;
pub const FAR:f32 = 1000.0;
pub const VFOV:f32 = 0.5;
pub const INV_VFOV:f32 = 1.0 / VFOV;
pub const DEPTH:f32 = FAR - NEAR;
pub const INV_DEPTH:f32 = 1.0 / DEPTH;

pub const CHUNK_SIZE:i32 = 32;
pub const INV_CHUNK_SIZE:f32 = 1.0 / CHUNK_SIZE as f32;
pub const H_CHUNK_SIZE:i32 = CHUNK_SIZE / 2;
pub const CHUNK_AREA:i32 = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL:i32 = CHUNK_AREA * CHUNK_SIZE;

pub const P_CHUNK_SIZE:i32 = 34;
pub const P_CHUNK_AREA:i32 = P_CHUNK_SIZE * P_CHUNK_SIZE - 4;

pub const RENDER_DISTANCE:i32 = 8;
//pub const RENDER_VOL:i32 = (2*RENDER_DISTANCE+1)*(2*RENDER_DISTANCE+1)*(2*RENDER_DISTANCE+1);

pub const SEED:u32 = 1;

pub const START_X:f32 = 0.0;
pub const START_Y:f32 = 0.0;
pub const START_Z:f32 = 0.0;

pub const START_CHUNK_X:i32 = (START_X * INV_CHUNK_SIZE) as i32;
pub const START_CHUNK_Y:i32 = (START_Y * INV_CHUNK_SIZE) as i32;
pub const START_CHUNK_Z:i32 = (START_Z * INV_CHUNK_SIZE) as i32;


pub const NUM_ENTITIES:usize = 8;
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VOXELS {
    EMPTY,
    SAND,
    GRASS,
    DIRT,
    COBBLESTONE,
    SNOW,
    LEAF,
    WOOD
}
impl VOXELS {
    pub fn iter() -> impl Iterator<Item = VOXELS> {
        [
            VOXELS::SAND,
            VOXELS::GRASS,
            VOXELS::DIRT,
            VOXELS::COBBLESTONE,
            VOXELS::SNOW,
            VOXELS::LEAF,
            VOXELS::WOOD
        ].into_iter()
    }
}




#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ENTITIES {
    SEED
}
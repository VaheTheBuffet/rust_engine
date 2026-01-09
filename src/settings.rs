use crate::math;

//SCREEN
pub const ASPECT_RATIO:f32 = 16.0/9.0;
pub const INV_ASPECT_RATIO:f32 = 1.0 / ASPECT_RATIO;
pub const WIDTH:u32 = 1280;
pub const HEIGHT:u32 = (INV_ASPECT_RATIO * (WIDTH as f32)) as u32;

//CAMERA
pub const NEAR:f32 = 0.1;
pub const FAR:f32 = 1000.0;
pub const VFOV_TAN:f32 = 0.5;
pub const HFOV_TAN:f32 = 0.888888888889;//VFOV_TAN * ASPECT_RATIO;
pub const VFOV_SEC:f32 = 1.3379549532;//1.0 + 0.5 * VFOV_TAN*VFOV_TAN;
pub const HFOV_SEC:f32 = 1.67459702313;//1.0 + 0.5 * HFOV_TAN*HFOV_TAN;
pub const INV_VFOV:f32 = 1.0 / VFOV_TAN;
pub const INV_HFOV:f32 = 1.0 / HFOV_TAN;
pub const DEPTH:f32 = FAR - NEAR;
pub const INV_DEPTH:f32 = 1.0 / DEPTH;
pub const FRUSTUM_Y: f32 = VFOV_SEC * CHUNK_RADIUS;
pub const FRUSTUM_X: f32 = HFOV_SEC * CHUNK_RADIUS;
pub const PROJECTION: [f32; 16] = [
    INV_HFOV,      0.0,                  0.0,                      0.0,
         0.0, INV_VFOV,                  0.0,                      0.0,
         0.0,      0.0,-(FAR+NEAR)*INV_DEPTH,-2.0*(FAR*NEAR)*INV_DEPTH,
         0.0,      0.0,                 -1.0,                      0.0 
];

//CHUNK
pub const CHUNK_SIZE:i32 = 32;
pub const INV_CHUNK_SIZE:f32 = 1.0 / CHUNK_SIZE as f32;
pub const H_CHUNK_SIZE:i32 = CHUNK_SIZE / 2;
pub const CHUNK_AREA:i32 = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL:i32 = CHUNK_AREA * CHUNK_SIZE;
pub const CHUNK_RADIUS: f32 = 27.7128129211;//CHUNK_SIZE as f32 * math::ROOT_3 / 2.0;

//GAME SETTINGS
pub const RENDER_DISTANCE:i32 = 8;
//pub const RENDER_VOL:i32 = (2*RENDER_DISTANCE+1)*(2*RENDER_DISTANCE+1)*(2*RENDER_DISTANCE+1);

pub const SEED:u32 = 1;

pub const START_X:f32 = 0.0;
pub const START_Y:f32 = 0.0;
pub const START_Z:f32 = 0.0;

pub const START_CHUNK_X:i32 = (START_X * INV_CHUNK_SIZE) as i32;
pub const START_CHUNK_Y:i32 = (START_Y * INV_CHUNK_SIZE) as i32;
pub const START_CHUNK_Z:i32 = (START_Z * INV_CHUNK_SIZE) as i32;

//ENTITY SETTINGS
pub const NUM_ENTITIES:usize = 8;
pub const NUM_TEXTURES:i32 = NUM_ENTITIES as i32 + 1;
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
    WOOD,
    WATER
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
            VOXELS::WOOD,
            VOXELS::WATER
        ].into_iter()
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ENTITIES {
    SEED
}
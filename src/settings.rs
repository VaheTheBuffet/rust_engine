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
pub const CHUNK_AREA:i32 = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL:i32 = CHUNK_AREA * CHUNK_SIZE;


pub const WORLD_W:i32 = 15;
pub const WORLD_H:i32 = 3;
pub const WORLD_A:i32 = WORLD_W * WORLD_W;
pub const WORLD_V:i32 = WORLD_A * WORLD_H;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum VOXELS {
    EMPTY = 0,
    SOLID
}
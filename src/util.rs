use noise::{NoiseFn, Perlin};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use crate::settings::*;

pub struct Noise {
    perlin: Perlin,
}

impl Noise {
    pub fn new(seed:u32) -> Self {
        Self{perlin: Perlin::new(seed)}
    }

    pub fn get_height(&self, x:f64, z:f64) -> i32 {
        let h1 = (self.perlin.get([x / 100.1, z / 100.1]) * CHUNK_SIZE as f64) as i32;
        let h2 = (self.perlin.get([x / 200.1, z / 200.1]) * CHUNK_SIZE as f64) as i32;
        let h3 = (self.perlin.get([x / 400.1, z / 400.1]) * CHUNK_SIZE as f64) as i32;
        let h4 = (self.perlin.get([x / 800.1, z / 800.1]) * CHUNK_SIZE as f64) as i32;

        H_CHUNK_SIZE + h1 - h2 + h3 - h4 
    }
}


#[inline(always)]
pub fn render_range((px,py,pz):(i32,i32,i32)) -> impl ParallelIterator<Item = (i32,i32,i32)> {
    (-RENDER_DISTANCE+px..=RENDER_DISTANCE+px).into_par_iter().flat_map(move |x| {
        (-RENDER_DISTANCE+py..=RENDER_DISTANCE+py).into_par_iter().flat_map(move |y| {
            (-RENDER_DISTANCE+pz..=RENDER_DISTANCE+pz).into_par_iter().map(move |z| {
                (x,y,z)
            })
        })
    }).into_par_iter()
} 


#[inline(always)]
pub fn border_range((px,py,pz):(i32,i32,i32)) -> impl ParallelIterator<Item = (i32,i32,i32)> {
    (-RENDER_DISTANCE-1..=RENDER_DISTANCE+1).into_par_iter().flat_map(move |t1| {
        (-RENDER_DISTANCE-1..=RENDER_DISTANCE+1).into_par_iter().flat_map(move |t2| {
            [
                (px-RENDER_DISTANCE-1,py+t1,pz+t2), (px+RENDER_DISTANCE+1,py+t1,pz+t2),
                (px+t1,py-RENDER_DISTANCE-1,pz+t2), (px+t1,py+RENDER_DISTANCE+1,pz+t2),
                (px+t1,py+t2,pz-RENDER_DISTANCE-1), (px+t1,py+t2,pz+RENDER_DISTANCE+1)
            ]
        })
    })
} 


#[inline(always)]
pub fn _outer_border_range((px,py,pz):(i32,i32,i32)) -> impl Iterator<Item = (i32,i32,i32)> {
    (-RENDER_DISTANCE-2..=RENDER_DISTANCE+2).flat_map(move |t1| {
        (-RENDER_DISTANCE-2..=RENDER_DISTANCE+2).flat_map(move |t2| {
            [
                (px-RENDER_DISTANCE-2,py+t1,pz+t2), (px+RENDER_DISTANCE+2,py+t1,pz+t2),
                (px+t1,py-RENDER_DISTANCE-2,pz+t2), (px+t1,py+RENDER_DISTANCE+2,pz+t2),
                (px+t1,py+t2,pz-RENDER_DISTANCE-2), (px+t1,py+t2,pz+RENDER_DISTANCE+2)
            ]
        })
    })
} 
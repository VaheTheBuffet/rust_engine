use crate::math::{dot, H_PI};
use crate::{settings::*};
use core::f32;


pub struct Player {
    pub pitch:f32,
    pub yaw:f32,
    pub x:f32,
    pub y:f32,
    pub z:f32,
    pub chunk_x:i32,
    pub chunk_y:i32,
    pub chunk_z:i32,
    pub speed:f32,
    pub sensitivity:f32,
    pub forward:[f32; 3],
    pub right:[f32; 3],
    pub up:[f32; 3],
}


impl Player{
    pub fn new() -> Player {
        Player {
            pitch:0.0,
            yaw:0.0,
            x:START_X,
            y:START_Y,
            z:START_Z,
            chunk_x:START_CHUNK_X,
            chunk_y:START_CHUNK_Y,
            chunk_z:START_CHUNK_Z,
            speed:10.0,
            sensitivity:0.01,
            forward: [0.0, 0.0, -1.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0]
        }
    }
}


pub trait Camera{
    fn move_up(&mut self, dt:f32);
    fn move_down(&mut self, dt:f32);
    fn move_right(&mut self, dt:f32);
    fn move_left(&mut self, dt:f32);
    fn move_forward(&mut self, dt:f32);
    fn move_backward(&mut self, dt:f32);
    fn rot_yaw(&mut self, mouse_dx:f32);
    fn rot_pitch(&mut self, mouse_dy:f32);
    fn update(&mut self);
    fn get_view_mat(&self)->[f32;16];
    fn get_proj_mat(&self)->[f32;16];
    fn is_in_frustum(&self, pos: (f32, f32, f32))->bool;
}


impl Camera for Player {
    fn move_up(self:&mut Player, delta_time:f32) {
        let [x,y,z] = self.up;
        self.x += x*self.speed*delta_time;
        self.y += y*self.speed*delta_time;
        self.z += z*self.speed*delta_time;
    }

    fn move_down(&mut self, delta_time:f32) {
        let [x,y,z] = self.up;
        self.x -= x*self.speed*delta_time;
        self.y -= y*self.speed*delta_time;
        self.z -= z*self.speed*delta_time;
    }

    fn move_right(&mut self, delta_time:f32) {
        let [x,y,z] = self.right;
        self.x += x*self.speed*delta_time;
        self.y += y*self.speed*delta_time;
        self.z += z*self.speed*delta_time;
    }

    fn move_left(&mut self, delta_time:f32) {
        let [x,y,z] = self.right;
        self.x -= x*self.speed*delta_time;
        self.y -= y*self.speed*delta_time;
        self.z -= z*self.speed*delta_time;
    }

    fn move_forward(&mut self, delta_time:f32) {
        let [x,y,z] = self.forward;
        self.x -= x*self.speed*delta_time;
        self.y -= y*self.speed*delta_time;
        self.z -= z*self.speed*delta_time;
    }

    fn move_backward(&mut self, delta_time:f32) {
        let [x,y,z] = self.forward;
        self.x += x*self.speed*delta_time;
        self.y += y*self.speed*delta_time;
        self.z += z*self.speed*delta_time;
    }

    fn rot_pitch(&mut self, dy:f32) {
        self.pitch = (self.pitch - self.sensitivity*dy).clamp(-H_PI, H_PI);
    }

    fn rot_yaw(&mut self, dx:f32) {
        self.yaw -= self.sensitivity*dx;
    }

    fn update(&mut self) {
        let sinyaw = self.yaw.sin();
        let cosyaw = self.yaw.cos();
        let sinpitch = self.pitch.sin();
        let cospitch = self.pitch.cos();

        self.right = [cosyaw, 0.0, -sinyaw];
        self.up = [sinyaw*sinpitch, cospitch, cosyaw*sinpitch];
        self.forward = [sinyaw*cospitch, -sinpitch, cospitch*cosyaw];

        self.chunk_x = (self.x * INV_CHUNK_SIZE).floor() as i32;
        self.chunk_y = (self.y * INV_CHUNK_SIZE).floor() as i32;
        self.chunk_z = (self.z * INV_CHUNK_SIZE).floor() as i32;
    }

    #[inline(always)]
    fn get_view_mat(&self) -> [f32;16] {
        [
            self.right[0],   self.right[1],   self.right[2],      -self.x*self.right[0]-self.y*self.right[1]-self.z*self.right[2],
               self.up[0],      self.up[1],      self.up[2],               -self.x*self.up[0]-self.y*self.up[1]-self.z*self.up[2],
          self.forward[0], self.forward[1], self.forward[2],-self.x*self.forward[0]-self.y*self.forward[1]-self.z*self.forward[2],
                      0.0,             0.0,             0.0,                                                                  1.0
        ]
    }

    #[inline(always)]
    fn get_proj_mat(&self) -> [f32;16] {
        PROJECTION
    }

    fn is_in_frustum(&self, (x, y, z): (f32, f32, f32)) -> bool {
        let dv = [x-self.x,y-self.y,z-self.z];
        let depth = dot(self.forward, dv);
        if depth > -NEAR + CHUNK_RADIUS || depth < -FAR - CHUNK_RADIUS {return false;}

        let vertical = dot(self.up, dv).abs();
        if vertical > FRUSTUM_Y - depth * VFOV_TAN {return false;}

        let horizontal = dot(self.right, dv).abs();
        if horizontal > FRUSTUM_X - depth * HFOV_TAN {return false;}

        true
    }
}
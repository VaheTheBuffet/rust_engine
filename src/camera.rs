use crate::math::{cross, normalize, H_PI};
use crate::{settings::*};
use core::f32;


pub struct Camera {
    pub pitch:f32,
    pub yaw:f32,
    pub x:f32,
    pub y:f32,
    pub z:f32,
    pub speed:f32,
    pub sensitivity:f32,
    pub forward:[f32; 3],
    pub right:[f32; 3],
    pub up:[f32; 3],
}


impl Camera{
    pub fn new() -> Camera {
        Camera {
            pitch:0.0,
            yaw:-H_PI,
            x:16.0,
            y:50.0,
            z:50.0,
            speed:4.0,
            sensitivity:0.01,
            forward: [0.0, 0.0, -1.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0]
        }
    }
}


pub trait HasCamera{
    fn move_up(&mut self, dt:&f32);
    fn move_down(&mut self, dt:&f32);
    fn move_right(&mut self, dt:&f32);
    fn move_left(&mut self, dt:&f32);
    fn move_forward(&mut self, dt:&f32);
    fn move_backward(&mut self, dt:&f32);
    fn rot_yaw(&mut self, mouse_dx:&f32);
    fn rot_pitch(&mut self, mouse_dy:&f32);
    fn update_forward(&mut self);
    fn update_right(&mut self);
    fn update_up(&mut self);
    fn update(&mut self);
    fn get_view_mat(&self)->[f32;16];
    fn get_proj_mat(&self)->[f32;16];
}


impl HasCamera for Camera {
    fn move_up(self:&mut Camera, delta_time:&f32) {
        let [x,y,z] = self.up;
        self.x += x*self.speed*delta_time;
        self.y += y*self.speed*delta_time;
        self.z += z*self.speed*delta_time;
    }

    fn move_down(&mut self, delta_time:&f32) {
        let [x,y,z] = self.up;
        self.x -= x*self.speed*delta_time;
        self.y -= y*self.speed*delta_time;
        self.z -= z*self.speed*delta_time;
    }

    fn move_right(&mut self, delta_time:&f32) {
        let [x,y,z] = self.right;
        self.x += x*self.speed*delta_time;
        self.y += y*self.speed*delta_time;
        self.z += z*self.speed*delta_time;
    }

    fn move_left(&mut self, delta_time:&f32) {
        let [x,y,z] = self.right;
        self.x -= x*self.speed*delta_time;
        self.y -= y*self.speed*delta_time;
        self.z -= z*self.speed*delta_time;
    }

    fn move_forward(&mut self, delta_time:&f32) {
        let [x,y,z] = self.forward;
        self.x += x*self.speed*delta_time;
        self.y += y*self.speed*delta_time;
        self.z += z*self.speed*delta_time;
    }

    fn move_backward(&mut self, delta_time:&f32) {
        let [x,y,z] = self.forward;
        self.x -= x*self.speed*delta_time;
        self.y -= y*self.speed*delta_time;
        self.z -= z*self.speed*delta_time;
    }

    fn rot_pitch(&mut self, dy:&f32) {
        self.pitch = (self.pitch - self.sensitivity*dy).clamp(-H_PI, H_PI);
    }

    fn rot_yaw(&mut self, dx:&f32) {
        self.yaw =self.yaw + self.sensitivity*dx;
    }

    fn update_forward(&mut self) {
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();
        self.forward[0] = cy * cp;
        self.forward[1] = sp;
        self.forward[2] = sy * cp;
    }

    fn update_right(&mut self) {
        self.right = normalize(&mut cross(&self.forward, &[0.0,1.0,0.0]));
    }

    fn update_up(&mut self) {
        self.up = cross(&self.right, &self.forward);
    }

    fn update(&mut self) {
        self.update_forward();
        self.update_right();
        self.update_up();
    }

    fn get_view_mat(&self) -> [f32;16] {
        [
            self.right[0],   self.right[1],   self.right[2],      -self.x*self.right[0]-self.y*self.right[1]-self.z*self.right[2],
               self.up[0],      self.up[1],      self.up[2],               -self.x*self.up[0]-self.y*self.up[1]-self.z*self.up[2],
          self.forward[0], self.forward[1], self.forward[2],-self.x*self.forward[0]-self.y*self.forward[1]-self.z*self.forward[2],
                      0.0,             0.0,             0.0,                                                                  1.0
        ]
    }

    fn get_proj_mat(&self) -> [f32;16] {
        [
            INV_VFOV*INV_ASPECT_RATIO,      0.0,                  0.0,                      0.0,
                                  0.0, INV_VFOV,                  0.0,                      0.0,
                                  0.0,      0.0, (FAR+NEAR)*INV_DEPTH,-2.0*(FAR*NEAR)*INV_DEPTH,
                                  0.0,      0.0,                  1.0,                      0.0 
        ]
    }
}
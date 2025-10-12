use crate::math::{cross, normalize, IDENTITY};
use crate::settings::*;
use std::f32::consts::{PI};
pub struct Camera {
    pitch:f32,
    yaw:f32,
    x:f32,
    y:f32,
    z:f32,
    speed:f32,
    forward:[f32; 3],
    right:[f32; 3],
    up:[f32; 3],
}

impl Camera{
    pub fn new() ->Camera {
        Camera {
            pitch:0.0,
            yaw:PI/2.0,
            x:0.0,
            y:0.0,
            z:0.0,
            speed:5.0,
            forward: [0.0, 0.0, -1.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0]
        }
    }

    pub fn move_up(&mut self) {
        self.y += self.speed;
    }

    pub fn move_down(&mut self) {
        self.y -= self.speed;
    }

    pub fn move_right(&mut self) {
        self.x += self.speed;
    }

    pub fn move_left(&mut self) {
        self.x -= self.speed;
    }

    pub fn move_forward(&mut self) {
        self.z -= self.speed;
    }

    pub fn move_backward(&mut self) {
        self.z += self.speed;
    }

    pub fn rot_pitch(&mut self) {
        self.pitch += self.speed;
    }

    pub fn rot_yaw(&mut self) {
        self.yaw += self.speed;
    }

    pub fn update_forward(&mut self) {
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();
        self.forward[0] = cy * cp;
        self.forward[1] = sp;
        self.forward[2] = sy * cp;
    }

    pub fn update_right(&mut self) {
        self.right = normalize(&mut cross(&self.forward, &[0.0,1.0,0.0]));
    }

    pub fn update_up(&mut self) {
        self.up = cross(&self.right, &self.forward);
    }

    pub fn update_vectors(&mut self) {
        self.update_forward();
        self.update_right();
        self.update_up();
    }

    pub fn get_view_mat(&mut self) -> [f32;16] {
        [
            self.right[0], self.right[1], self.right[2],-self.x,
            self.up[0], self.up[1], self.up[2],-self.y,
            self.forward[0], self.forward[1], self.forward[2],-self.z,
                      0.0,        0.0,             0.0,    1.0
        ]
    }

    pub fn get_proj_mat(&mut self) -> [f32;16] {
        [
            NEAR,        0.0,                   0.0,                     0.0,
            0.0,        NEAR,                   0.0,                     0.0,
            0.0,         0.0, (FAR+NEAR)/(FAR-NEAR),-2.0*FAR*NEAR/(FAR-NEAR),
            0.0,         0.0,                   1.0,                     0.0
        ]
    }
}
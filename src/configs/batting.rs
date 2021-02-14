// use super::_interfaces::{Batting, BattingState, HitInfo};
use crate::bezier::bezier_scalar;
use crate::game_state::batting::{BatCoord, BattingConfig};
use crate::impls::vector::{Cross, Normalize};
use crate::log;

use std::f32::consts::PI;
use webgl_matrix::{Mat4, Matrix, Vec3, Vector};

pub struct BattingConfigImpl {
    swing_duration: f32,
    swing_active: (f32, f32),
    batting_area_center: Vec3,
    batting_area_u_axis: Vec3,
    batting_area_v_axis: Vec3,
    batting_area_rect: (f32, f32),
    arm_rot_pivot: Vec3,
    arm_rot_axis: Vec3,
    arm_angle_range: (f32, f32, f32, f32),
    bat_rot_pivot: Vec3,
    bat_rot_axis: Vec3,
    bat_angle_range: (f32, f32, f32, f32),
    bat_center: Vec3,
    bat_length: f32,
    bat_width: f32,
}

impl BattingConfigImpl {
    pub fn init(&mut self) {
        self.arm_rot_axis = self.arm_rot_axis.normalize();
        self.bat_rot_axis = self.bat_rot_axis.normalize();
        self.batting_area_u_axis = self.batting_area_u_axis.normalize();
        self.batting_area_v_axis = self.batting_area_v_axis.normalize();
    }
}

impl BattingConfig for BattingConfigImpl {
    fn normalized_time(&self, time: f32) -> f32 {
        time / self.swing_duration
    }

    fn constrain_batting_area(&self, batter_position: &Vec3) -> Vec3 {
        let batter = batter_position.sub(&self.batting_area_center);
        let batting_area_w_axis = self
            .batting_area_u_axis
            .cross(&self.batting_area_v_axis)
            .normalize();

        let u = self
            .batting_area_u_axis
            .dot(&batter)
            .clamp(-self.batting_area_rect.0, self.batting_area_rect.0);
        let v = self
            .batting_area_v_axis
            .dot(&batter)
            .clamp(-self.batting_area_rect.1, self.batting_area_rect.1);
        let w = batting_area_w_axis.dot(&batter);

        self.batting_area_center
            .add(&batting_area_w_axis.scale(w))
            .add(&self.batting_area_u_axis.scale(u))
            .add(&self.batting_area_v_axis.scale(v))
    }

    fn is_active_swing_time(&self, t: f32) -> bool {
        self.swing_active.0 < t && t < self.swing_active.1
    }

    fn bat_coord(&self, t: f32) -> BatCoord {
        let arm_rotation = rotate_matrix_bezier_angle(t, &self.arm_rot_axis, self.arm_angle_range);
        let bat_rotation = rotate_matrix_bezier_angle(t, &self.bat_rot_axis, self.bat_angle_range);

        let hand = rotate_around(&self.bat_rot_pivot, &self.arm_rot_pivot, &arm_rotation);
        let origin = rotate_around(
            &rotate_around(&self.bat_center, &self.bat_rot_pivot, &bat_rotation),
            &self.arm_rot_pivot,
            &arm_rotation,
        );

        let y_axis = arm_rotation.mul_vector(&self.bat_rot_axis);
        let y_axis = [y_axis[0], y_axis[1], y_axis[2]].scale(1. / y_axis[3]);

        let x_axis = hand.sub(&origin);
        let x_axis = x_axis.scale(-1. / x_axis.mag());

        let z_axis = y_axis.cross(&x_axis);

        BatCoord {
            origin,
            x_axis,
            y_axis,
            z_axis,
        }
    }

    fn is_valid_meet(&self, [x, y]: [f32; 2]) -> bool {
        x.abs() <= self.bat_length * 0.5 && y.abs() <= self.bat_width * 0.5
    }
}

fn rotate_matrix_bezier_angle(t: f32, axis: &Vec3, angle_range: (f32, f32, f32, f32)) -> Mat4 {
    let angle = bezier_scalar(t, angle_range);
    let mut mat = Mat4::identity();
    mat.rotate(angle, axis);
    mat
}

fn rotate_around(point: &Vec3, origin: &Vec3, mat: &Mat4) -> Vec3 {
    let p = mat.mul_vector(&point.sub(origin));
    let w = 1. / p[3];
    [p[0] * w, p[1] * w, p[2] * w].add(origin)
}

// Concrete Instances

impl BattingConfigImpl {
    pub fn default() -> Self {
        Self {
            swing_duration: 400.,
            batting_area_center: [0., 0., 0.8],
            batting_area_rect: (0.3, 0.5),
            batting_area_u_axis: [1., 0., 0.],
            batting_area_v_axis: [0., 0., 1.],
            swing_active: (0.3, 0.8),
            arm_rot_pivot: [0., 1., 0.],
            arm_rot_axis: [0., 1., 1.],
            arm_angle_range: (0., 0.333 * PI, 0.666 * PI, PI),
            bat_rot_pivot: [0., 1., -0.2],
            bat_rot_axis: [1., 0., 0.],
            bat_angle_range: (0., 0.2 * PI, 0.4 * PI, 0.6 * PI),
            bat_center: [0., 1.5, -0.2],
            bat_length: 0.5,
            bat_width: 0.2,
        }
    }
}

use super::_interfaces::{Batting, BattingState, HitInfo};
use crate::log;

use webgl_matrix::{Vec3, Vector};

pub struct BatCoord {
    pub origin: Vec3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
}

pub trait BattingConfig {
    fn normalized_time(&self, time: f32) -> f32;
    fn constrain_batting_area(&self, batter_position: &Vec3) -> Vec3;
    fn is_active_swing_time(&self, t: f32) -> bool;
    fn bat_coord(&self, t: f32) -> BatCoord;
    fn is_valid_meet(&self, meet: [f32; 2]) -> bool;
}

pub struct BattingImpl<C>
where
    C: BattingConfig,
{
    is_swinging: bool,
    swang_at: f32,
    batter_position: Vec3,
    last_ball_position_local: Vec3,
    config: C,
}

impl<C> Batting for BattingImpl<C>
where
    C: BattingConfig,
{
    type Config = C;
    fn new(config: C) -> Self {
        Self {
            is_swinging: false,
            swang_at: f32::MAX,
            batter_position: [0., 0., 0.],
            last_ball_position_local: [0., 0., 0.],
            config,
        }
    }

    fn set_batter_position(&mut self, position: Vec3) {
        if !self.is_swinging {
            self.batter_position = position;
        }
    }

    fn swing(&mut self, swang_at: f32) {
        if !self.is_swinging {
            self.swang_at = swang_at;
            self.is_swinging = true;
        }
    }

    fn update(&mut self, time: f32, ball_position: Option<Vec3>) -> BattingState {
        let t = self.config.normalized_time(time - self.swang_at);
        let batter_position = self.config.constrain_batting_area(&self.batter_position);
        if t < 0. || 1. < t || ball_position.is_none() {
            self.is_swinging = false;
            return BattingState::Idle {
                batter: batter_position,
            };
        }

        let swinging = BattingState::Swinging {
            batter: batter_position,
            swing_degree: t,
        };
        if !self.config.is_active_swing_time(t) {
            return swinging;
        }

        let ball_position = ball_position.unwrap();
        let bat_coord = self.config.bat_coord(t);
        let point = ball_position.sub(&batter_position).sub(&bat_coord.origin);
        let x = point.dot(&bat_coord.x_axis);
        let y = point.dot(&bat_coord.y_axis);
        let z = point.dot(&bat_coord.z_axis);

        if z > 0. {
            self.last_ball_position_local = [x, y, z];
            return swinging;
        } else if !self.config.is_valid_meet([x, y]) {
            return swinging;
        }
        // TODO: stop following frame update

        let meet = self.last_ball_position_local.sub(&[x, y, z]);
        let z = -z / meet[2];
        let meet = [z * meet[0] + x, z * meet[1] + y];

        BattingState::Hit(HitInfo {
            x_axis: bat_coord.x_axis,
            y_axis: bat_coord.y_axis,
            z_axis: bat_coord.z_axis,
            origin: bat_coord.origin,
            meet_position: meet,
        })
    }
}

use super::{Frame, Renderable};
use crate::bezier::BezierTrajectory;

use webgl_matrix::{Mat4, Matrix};

pub struct ThrownBall {
    trajectory: BezierTrajectory,
    model: Mat4,
}

impl ThrownBall {
    pub fn new(trajectory: BezierTrajectory) -> Self {
        Self {
            trajectory,
            model: Mat4::identity(),
        }
    }

    pub fn next(&mut self) -> Result<(), String> {
        self.trajectory.set_loop(true);
        let p = self.trajectory.next().unwrap();
        self.model = [
            1., 0., 0., 0., // x
            0., 1., 0., 0., // y
            0., 0., 1., 0., // z
            p[0], p[1], p[2], 1.,
        ];
        Ok(())
    }
}

impl Renderable for ThrownBall {
    const FRAMES: [Option<(f32, Frame)>; 16] = [
        Some((
            1.,
            Frame {
                uv_offset: [0., 0.],
                uv_scale: [0.05, 0.05],
                pos_offset: [0., 0.],
            },
        )),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];
    fn get_parameter(&self, _: f32) -> f32 {
        0.
    }
    fn model(&self) -> Mat4 {
        self.model
    }
    fn set_model(&mut self, model: Mat4) {
        self.model = model;
    }
}

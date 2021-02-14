use super::{Frame, Renderable};

use webgl_matrix::{Mat4, Matrix};

pub struct ThrownBall {
    model: Mat4,
}

impl ThrownBall {
    pub fn new() -> Self {
        Self {
            model: Mat4::identity(),
        }
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
    fn model(&self) -> Mat4 {
        self.model
    }
    fn set_model(&mut self, model: Mat4) {
        self.model = model;
    }
}

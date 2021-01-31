use super::{Frame, Renderable};

use webgl_matrix::{Mat4, Matrix};

pub struct SampleEntity {
    duration: f32,
    start_at: f32,
    model: Mat4,
}

const HEIGHT: f32 = 0.2197265625;

impl SampleEntity {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            start_at: f32::MAX,
            model: Mat4::identity(),
        }
    }

    pub fn start(&mut self, time: f32) {
        self.start_at = time;
    }
}

/// sample_entity_0.png
impl Renderable for SampleEntity {
    const FRAMES: [Option<(f32, Frame)>; 16] = [
        Some((
            0.2,
            Frame {
                uv_offset: [0.01, 1. - HEIGHT],
                uv_scale: [0.126953125, HEIGHT],
                pos_offset: [0.1, 0.],
            },
        )),
        Some((
            0.4,
            Frame {
                uv_offset: [0.13916015625, 1. - HEIGHT],
                uv_scale: [0.126953125, HEIGHT],
                pos_offset: [-0.1, 0.],
            },
        )),
        Some((
            0.6,
            Frame {
                uv_offset: [0.4345703125, 1. - HEIGHT],
                uv_scale: [0.2099609375, HEIGHT],
                pos_offset: [0.6, 0.],
            },
        )),
        Some((
            3.0,
            Frame {
                uv_offset: [0.27099609375, 1. - HEIGHT],
                uv_scale: [0.1611328125, HEIGHT],
                pos_offset: [-0.3, 0.],
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
    ];
    fn get_parameter(&self, time: f32) -> f32 {
        (time - self.start_at) / self.duration
    }
    fn model(&self) -> Mat4 {
        self.model
    }
    fn set_model(&mut self, model: Mat4) {
        self.model = model;
    }
}

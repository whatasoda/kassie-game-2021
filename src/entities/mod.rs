pub mod sample_batter;

use crate::shaders::entity_shader::Instance;

use webgl_matrix::Mat4;

#[derive(Clone)]
pub struct Frame {
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
    pos_offset: [f32; 2],
}

pub trait Renderable {
    const FRAMES: [Option<(f32, Frame)>; 16];

    fn get_parameter(&self, time: f32) -> f32;
    fn model(&self) -> Mat4;
    fn set_model(&mut self, model: Mat4);
}

pub fn get_current_instance_value<T>(target: &T, time: f32) -> Instance
where
    T: Renderable,
{
    let t = target.get_parameter(time);
    let frames = T::FRAMES;
    let mut frames = frames.iter();
    let mut frame: Option<Frame> = None;
    while let Some(Some((threshold, curr))) = frames.next() {
        if t < *threshold {
            frame = Some(curr.clone());
            break;
        }
    }

    let Frame {
        uv_offset,
        uv_scale,
        pos_offset,
    } = frame.unwrap_or_else(|| {
        T::FRAMES
            .get(0)
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .1
            .clone()
    });
    Instance {
        model: target.model(),
        uv_offset,
        uv_scale,
        pos_offset,
    }
}

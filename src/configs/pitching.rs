use crate::bezier::ParametricCurveSequence;
use crate::game_state::pitching::PitchingConfig;
use crate::log;

use webgl_matrix::Vec3;

pub struct PitchingConfigImpl<C> {
    curve: C,
    idle_duration: f32,
    pitching_duration: f32,
    throws_at: f32,
    ball_duration: f32,
    pitcher_position: Vec3,
}

impl<C> PitchingConfig for PitchingConfigImpl<C>
where
    C: ParametricCurveSequence,
{
    fn next_idle_break(&self, offset: f32, timestamp: f32) -> f32 {
        let duration = self.idle_duration;
        ((timestamp - offset) / duration).ceil() * duration + offset
    }

    fn pre_idle_parameter(&self, offset: f32, time: f32) -> f32 {
        ((time - offset) / self.idle_duration).fract()
    }

    fn post_idle_parameter(&self, time: f32) -> f32 {
        let t = time - self.pitching_duration;
        if t < 0. {
            0.
        } else {
            (t / self.idle_duration).fract()
        }
    }

    fn pitching_parameter(&self, time: f32) -> f32 {
        time / self.pitching_duration
    }

    fn ball_parameter(&self, time: f32) -> f32 {
        (time - self.throws_at) / self.ball_duration
    }

    fn ball_position(&self, t: f32) -> Option<Vec3> {
        // log::log_f32(t);
        self.curve.calc_point(self.curve.duration() * t, false)
    }
}

use crate::bezier::{BezierSequence, Curve};

impl PitchingConfigImpl<BezierSequence> {
    pub fn default() -> Self {
        Self {
            curve: BezierSequence::new(vec![Curve {
                t_duration: 333.0,
                t_p: None,
                p_0: [0., -0.6, 2.],
                p_1: [0., -0.6, 3.],
                p_2: [0., -0.6, 4.],
                p_3: [0., -0.6, 5.],
            }]),
            idle_duration: 300.,
            pitching_duration: 500.,
            throws_at: 200.,
            ball_duration: 1000.,
            pitcher_position: [0., 0., 2.],
        }
    }
}

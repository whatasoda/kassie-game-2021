use super::_interfaces::{PitcherState, Pitching, PitchingState};

use webgl_matrix::Vec3;

pub trait PitchingConfig {
    fn next_idle_break(&self, offset: f32, timestamp: f32) -> f32;
    fn pre_idle_parameter(&self, offset: f32, time: f32) -> f32;
    fn post_idle_parameter(&self, time: f32) -> f32;
    fn pitching_parameter(&self, time: f32) -> f32;
    fn ball_parameter(&self, time: f32) -> f32;
    fn ball_position(&self, t: f32) -> Option<Vec3>;
}

pub struct PitchingImpl<C>
where
    C: PitchingConfig,
{
    config: C,
    idle_offset: f32,
    pitched_at: f32,
    is_pitching: bool,
}

impl<C> Pitching for PitchingImpl<C>
where
    C: PitchingConfig,
{
    type Config = C;

    fn new(config: Self::Config) -> Self {
        Self {
            config,
            idle_offset: 0.,
            pitched_at: f32::MAX,
            is_pitching: false,
        }
    }

    fn reset_idle(&mut self, timestamp: f32) {
        self.idle_offset = timestamp;
    }

    fn pitch(&mut self, timestamp: f32) {
        if !self.is_pitching {
            self.pitched_at = self.config.next_idle_break(self.idle_offset, timestamp);
            self.is_pitching = true;
        }
    }

    fn end(&mut self) {
        self.pitched_at = f32::MAX;
    }

    fn update(&mut self, time: f32) -> PitchingState {
        let time = time - self.pitched_at;
        if time < 0. {
            return PitchingState {
                pitcher: PitcherState::Idle(self.config.pre_idle_parameter(self.idle_offset, time)),
                ball_position: None,
            };
        }

        let ball_parameter = self.config.ball_parameter(time);
        let pitching_parameter = self.config.pitching_parameter(time);

        let ball_position = if ball_parameter < 0. || 1. < ball_parameter {
            None
        } else {
            self.config.ball_position(ball_parameter)
        };

        let pitcher = if pitching_parameter < 1. {
            PitcherState::Pitching(pitching_parameter)
        } else {
            PitcherState::Idle(self.config.post_idle_parameter(time))
        };

        match (&ball_position, &pitcher) {
            (None, PitcherState::Idle(_)) => {
                self.is_pitching = false;
            }
            _ => {}
        }

        PitchingState {
            pitcher,
            ball_position,
        }
    }
}

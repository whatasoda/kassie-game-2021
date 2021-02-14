use super::_interfaces::{HitBall, HitBallState, HitInfo, HitResult};

use webgl_matrix::{Vec3, Vector};

pub struct Config {
    duration: f32,
    ground_height: f32,
    gravity: f32,
    sector: Sector,
}

pub struct Sector {
    position: Vec3,
    direction: Vec3,
    angle_cos: f32,
    radius: f32,
}

pub struct Parabola {
    start: Vec3,
    initial_velocity: Vec3,
}

pub struct HitBallImpl {
    config: Config,
    hit_at: f32,
    parabola: Option<Parabola>,
    curr_result: Option<(f32, HitResult)>,
    last_ball_position: Vec3,
}

impl HitBall for HitBallImpl {
    type Config = Config;
    fn new(config: Self::Config) -> Self {
        Self {
            config,
            hit_at: f32::MAX,
            parabola: None,
            curr_result: None,
            last_ball_position: [0., 0., 0.],
        }
    }

    fn hit(&mut self, timestamp: f32, info: HitInfo) {
        self.hit_at = timestamp;
        self.parabola = Some(Parabola {
            start: [0., 0., 0.],
            initial_velocity: [0., 0., 0.],
        });
    }

    fn update(&mut self, time: f32) -> HitBallState {
        let t = (time - self.hit_at) / self.config.duration;
        if t < 0. || 1. < t {
            self.curr_result = None;
            return HitBallState::Idle {};
        }
        match &self.parabola {
            None => HitBallState::Idle {},
            Some(parabola) => {
                let position = parabola.initial_velocity.scale(t).add(&[
                    0.,
                    self.config.gravity * t * t * 0.5,
                    0.,
                ]);

                if let Some((judged_at, result)) = self.curr_result.as_ref() {
                    return HitBallState::Result {
                        position,
                        result: result.clone(),
                        judged_at: *judged_at,
                    };
                }

                // TODO: consider air resistance
                let p = position.sub(&self.config.sector.position);

                let p_u = p.scale(1. / p.mag());
                let angle = self.config.sector.direction.dot(&p_u);

                if self.config.sector.angle_cos < angle {
                    self.last_ball_position = position;
                    self.curr_result = Some((time, HitResult::Foul));
                    return HitBallState::Result {
                        result: HitResult::Foul,
                        position,
                        judged_at: time,
                    };
                }

                if p[1] > self.config.ground_height {
                    return HitBallState::Frying { position };
                }

                // TODO: do some calcuration to align ball position with the ground

                let frying_distance = [p[0], 0., p[2]].mag();
                self.last_ball_position = position;

                let result = if frying_distance < self.config.sector.radius {
                    HitResult::SafeHit
                } else {
                    HitResult::HomeRun
                };

                self.last_ball_position = position;
                self.curr_result = Some((time, result.clone()));
                HitBallState::Result {
                    result,
                    position,
                    judged_at: time,
                }
            }
        }
    }
}

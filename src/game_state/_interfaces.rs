use webgl_matrix::Vec3;

// Shared Models

pub struct HitInfo {
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
    pub origin: Vec3,
    pub meet_position: [f32; 2],
}

// Traits

pub enum PitcherState {
    Idle(f32),
    Pitching(f32),
}

pub struct PitchingState {
    pub pitcher: PitcherState,
    pub ball_position: Option<Vec3>,
}

pub trait Pitching {
    type Config;
    fn new(config: Self::Config) -> Self;
    fn reset_idle(&mut self, timestamp: f32);
    fn pitch(&mut self, timestamp: f32);
    fn end(&mut self);
    fn update(&mut self, time: f32) -> PitchingState;
}

pub enum BattingState {
    Idle { batter: Vec3 },
    Swinging { batter: Vec3, swing_degree: f32 },
    Hit(HitInfo),
}

pub trait Batting {
    type Config;
    fn new(config: Self::Config) -> Self;
    fn set_batter_position(&mut self, position: Vec3);
    fn swing(&mut self, timestamp: f32);
    // TODO: fix `get_ball_position`
    fn update(&mut self, time: f32, ball_position: Option<Vec3>) -> BattingState;
}

pub enum HitBallState {
    Idle {},
    Frying {
        position: Vec3,
    },
    Result {
        position: Vec3,
        result: HitResult,
        judged_at: f32,
    },
}

#[derive(Clone)]
pub enum HitResult {
    Foul,
    HomeRun,
    SafeHit,
}

pub trait HitBall {
    type Config;
    fn new(config: Self::Config) -> Self;
    fn hit(&mut self, timestamp: f32, info: HitInfo);
    fn update(&mut self, time: f32) -> HitBallState;
}

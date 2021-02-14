mod sample;
mod test;

pub use sample::{SampleScene, SampleSceneContext};
pub use test::{TestScene, TestSceneContext};

pub enum SceneType {
    Batting,
    Test,
}

pub struct SceneManager {
    pub type_: SceneType,
}
impl SceneManager {
    pub fn set_scene(&mut self, type_: SceneType) {
        self.type_ = type_;
    }
}

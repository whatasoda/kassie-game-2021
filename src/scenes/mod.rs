mod sample;
mod test;

pub use sample::{SampleScene, SampleSceneContext};
pub use test::{TestScene, TestSceneContext};

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

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

pub struct Scenes {
    pub scene_manager: Rc<RefCell<SceneManager>>,
    pub batting: SampleScene,
    pub test: TestScene,
}

impl Scenes {
    pub fn render(&mut self, time: f32) -> Result<(), JsValue> {
        match self.scene_manager.borrow().type_ {
            SceneType::Test => self.test.render(time)?,
            SceneType::Batting => self.batting.render(time)?,
        }
        Ok(())
    }
}

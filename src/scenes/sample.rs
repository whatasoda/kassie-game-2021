use crate::camera::CameraController;
use crate::entities::get_current_instance_value;
use crate::entities::sample_batter::SampleEntity;
use crate::input::InputState;
use crate::scenes::SceneManager;
use crate::shader::SharedContext;
use crate::shaders::entity_shader::EntityShader;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use webgl_matrix::{Mat4, Matrix};

pub struct SampleSceneContext {
    pub scene_manager: Rc<RefCell<SceneManager>>,
    pub entity_shader: Rc<RefCell<EntityShader>>,
    pub camera: Rc<RefCell<CameraController>>,
    pub input: Rc<RefCell<InputState>>,
    pub shared: Rc<RefCell<SharedContext>>,
}

pub struct SampleScene {
    context: SampleSceneContext,
    batter: SampleEntity,
}

impl SampleScene {
    pub fn new(context: SampleSceneContext) -> Self {
        Self {
            context,
            batter: SampleEntity {
                start_at: 0.,
                duration: 600.,
                model: *Mat4::identity().scale(500.).translate(&[0., 0., -0.2]),
            },
        }
    }

    pub fn render(&mut self, time: f32) -> Result<(), JsValue> {
        let shared = self.context.shared.borrow();
        let input = self.context.input.borrow();
        let mut camera = self.context.camera.borrow_mut();
        let mut entity_shader = self.context.entity_shader.borrow_mut();

        camera.view.position = [0., 0., 0.];
        camera.view.direction = [0., 0., -1.];
        camera.refresh();
        unsafe {
            shared.uniform_buffer_data("camera", &camera.camera)?;
        }

        entity_shader.clear();
        if let Some(click) = &input.clicked {
            self.batter.start_at = click.timestamp;
        }
        self.batter.model = *Mat4::identity().scale(500.).translate(&[
            input.curr_coord.0 * 2.35 - 1.4,
            -input.curr_coord.1 * 2.3 - 0.,
            -0.2,
        ]);
        entity_shader
            .instances
            .push(get_current_instance_value(&self.batter, time));
        entity_shader.draw(time)?;

        Ok(())
    }
}

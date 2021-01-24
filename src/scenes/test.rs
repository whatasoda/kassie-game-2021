use crate::shader::SharedContext;
use crate::shaders::test::TestShader;
use crate::Uniform;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

pub struct TestSceneContext {
    pub test_shader: Rc<RefCell<TestShader>>,
    pub test_uniform: Rc<RefCell<Uniform>>,
    pub shared: Rc<RefCell<SharedContext>>,
}

pub struct TestScene {
    context: TestSceneContext,
}

impl TestScene {
    pub fn new(context: TestSceneContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, time: f32) -> Result<(), JsValue> {
        let shared = self.context.shared.borrow();
        let mut test_shader = self.context.test_shader.borrow_mut();
        let mut test_uniform = self.context.test_uniform.borrow_mut();

        test_uniform.size0 = time / 2000.0;
        unsafe {
            shared.uniform_buffer_data("uniforms_", &*test_uniform)?;
        }
        test_shader.draw(time)?;

        Ok(())
    }
}

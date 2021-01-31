use crate::bezier::{BezierTrajectory, Curve};
use crate::camera::CameraController;
use crate::entities::sample_batter::SampleEntity;
use crate::entities::thrown_ball::ThrownBall;
use crate::entities::{get_current_instance_value, Renderable};
use crate::input::InputState;
use crate::log;
use crate::scenes::SceneManager;
use crate::shader::SharedContext;
use crate::shaders::background_shader::{Background, BackgroundShader};
use crate::shaders::entity_shader::EntityShader;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use webgl_matrix::{Mat4, Matrix, MulVectorMatrix, Vector};

pub struct SampleSceneContext {
    pub scene_manager: Rc<RefCell<SceneManager>>,
    pub entity_shader: Rc<RefCell<EntityShader>>,
    pub background_shader: Rc<RefCell<BackgroundShader>>,
    pub camera: Rc<RefCell<CameraController>>,
    pub input: Rc<RefCell<InputState>>,
    pub shared: Rc<RefCell<SharedContext>>,
}

pub struct SampleScene {
    context: SampleSceneContext,
    batter: SampleEntity,
    background: Background,
    ball: ThrownBall,
    vp_inv: Mat4,
}

impl SampleScene {
    pub fn new(context: SampleSceneContext) -> Self {
        Self {
            context,
            batter: SampleEntity::new(600.),
            background: Background {
                model: [
                    9., 0., 0., 0., //
                    0., 9., 0., 0., //
                    0., 0., -10., 0., //
                    0., -1., -3., 1., //
                ],
            },
            ball: ThrownBall::new(BezierTrajectory::new(
                true,
                vec![Curve {
                    rate_coef: 0.1,
                    rate_1: 0.3333,
                    rate_2: 0.6666,
                    p_0: [0., -0.5, -0.2],
                    p_1: [0., -0.8, 2.],
                    p_2: [0., -0.6, 2.5],
                    p_3: [0., -0.5, 2.8],
                }],
            )),
            vp_inv: Mat4::zeros(),
        }
    }

    pub fn render(&mut self, time: f32) -> Result<(), JsValue> {
        let shared = self.context.shared.borrow();
        let input = self.context.input.borrow();
        let mut camera = self.context.camera.borrow_mut();
        let mut entity_shader = self.context.entity_shader.borrow_mut();
        let mut background_shader = self.context.background_shader.borrow_mut();

        unsafe {
            shared.uniform_buffer_data("background", &self.background)?;
        }

        let theta = PI / 10.;
        camera.view.position = [0., 0., 3.];
        camera.view.direction = [0., -theta.sin(), -theta.cos()];
        camera.refresh();
        unsafe {
            shared.uniform_buffer_data("camera", &camera.camera)?;
        }

        background_shader.clear();
        background_shader.draw(time)?;

        entity_shader.clear();
        self.ball.next()?;

        if let Some(click) = &input.clicked {
            self.batter.start(click.timestamp);
        }

        self.vp_inv = {
            let mut mat = camera.camera.vp_matrix.clone();
            mat.inverse();
            mat
        };
        let r = [input.curr_coord.0, input.curr_coord.1, -1.0, 1.0];
        let r = r.mul_matrix(&self.vp_inv);
        let r = r.scale(1. / r[3]);
        let r = [r[0], r[1], r[2]];
        let r = r.sub(&camera.view.position);
        let r = r.scale(1. / r.mag());
        let r = r.scale((-0.2 - camera.view.position[1]) / r[1]);
        let r = r.add(&camera.view.position);
        let r = r.add(&[-0.05, 0.005, 0.]);

        self.batter.set_model([
            0.2, 0., 0., 0., //
            0., 0.2, 0., 0., //
            0., 0., 0.2, 0., //
            r[0], r[1], r[2], 1., //
        ]);
        {
            let mut instances = entity_shader.instances_mut();
            instances.push(get_current_instance_value(&self.batter, time));
            instances.push(get_current_instance_value(&self.ball, time));
        }
        entity_shader.draw(time)?;

        Ok(())
    }
}

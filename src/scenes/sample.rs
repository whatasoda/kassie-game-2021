use crate::camera::CameraController;
use crate::entities::sample_batter::SampleEntity;
use crate::entities::thrown_ball::ThrownBall;
use crate::entities::{get_current_instance_value, Renderable};
use crate::game_state::{Batting, BattingState, GameStateBatting, GameStatePitching, Pitching};
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

pub struct SampleScene<G>
where
    G: GameStateBatting + GameStatePitching,
{
    context: SampleSceneContext,
    game_state: G,
    batter: SampleEntity,
    background: Background,
    ball: ThrownBall,
    vp_inv: Mat4,
}

impl<G> SampleScene<G>
where
    G: GameStateBatting + GameStatePitching,
{
    pub fn new(context: SampleSceneContext, game_state: G) -> Self {
        Self {
            context,
            game_state,
            batter: SampleEntity::new(),
            background: Background {
                model: [
                    9., 0., 0., 0., //
                    0., 9., 0., 0., //
                    0., 0., -10., 0., //
                    0., -1., -3., 1., //
                ],
            },
            ball: ThrownBall::new(),
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
        let r = r.scale((-0.8 - camera.view.position[1]) / r[1]);
        let r = r.add(&camera.view.position);
        // let r = r.add(&[-0.05, 0.005, 0.]);

        let mut pitching = self.game_state.pitching_mut();
        if time % 800. < 10. {
            pitching.pitch(time);
        }
        let pitching_state = pitching.update(time);
        let ball = pitching_state.ball_position;

        let mut batting = self.game_state.batting_mut();
        batting.set_batter_position(r);
        if let Some(click) = &input.clicked {
            batting.swing(click.timestamp);
        }
        let batting_state = batting.update(time, ball);

        let (batter, swing_degree) = match batting_state {
            BattingState::Idle { batter } => (batter, 0.),
            BattingState::Swinging {
                batter,
                swing_degree,
            } => (batter, swing_degree),
            BattingState::Hit(_) => {
                log::log("aa");
                return Ok(());
            }
        };
        // log::log_f32(swing_degree);

        self.batter.set_model([
            0.8, 0., 0., 0., //
            0., 0.8, 0., 0., //
            0., 0., 0.8, 0., //
            batter[0], batter[1], batter[2], 1., //
        ]);
        if let Some([x, y, z]) = ball {
            self.ball.set_model([
                0.8, 0., 0., 0., //
                0., 0.8, 0., 0., //
                0., 0., 0.8, 0., //
                x, y, z, 1., //
            ]);
        }
        {
            let mut instances = entity_shader.instances_mut();
            instances.push(get_current_instance_value(&self.batter, swing_degree));
            instances.push(get_current_instance_value(&self.ball, 0.));
        }
        entity_shader.draw(time)?;

        Ok(())
    }
}

mod bezier;
mod camera;
mod entities;
mod impls;
mod input;
mod log;
mod scenes;
mod scheduler;
mod shader;
mod shaders;
mod utils;

use crate::input::set_input_handler;
use crate::scenes::{SampleScene, SampleSceneContext, Scenes, TestScene, TestSceneContext};
use crate::scenes::{SceneManager, SceneType};
use crate::scheduler::start_loop;
use crate::shader::{ConvertArrayView, ShaderController, SharedContext};
use crate::shaders::background_shader::BackgroundShader;
use crate::shaders::entity_shader::EntityShader;
use crate::shaders::test::TestShader;

use num_traits::cast::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Performance, WebGl2RenderingContext};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut PERFORMANCE: Option<Rc<Performance>> = None;

#[repr(C)]
pub struct Uniform {
    size0: f32,
    size1: f32,
    _pad0: [u32; 2],
}
impl ConvertArrayView for Uniform {}

pub fn now() -> f32 {
    unsafe {
        PERFORMANCE
            .as_ref()
            .and_then(|p| p.now().to_f32())
            .unwrap_or(0.)
    }
}

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    let window = Rc::new(web_sys::window().unwrap());
    let doc = Rc::new(window.document().unwrap());
    let canvas = Rc::new(
        doc.get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?,
    );
    let ctx = Rc::new(
        canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?,
    );
    let performance = Rc::new(window.performance().unwrap());
    unsafe {
        PERFORMANCE = Some(performance.clone());
    }

    let scene_manager = Rc::new(RefCell::new(SceneManager {
        type_: SceneType::Test,
    }));

    let shared = SharedContext::new(doc.clone(), ctx.clone());
    shared
        .borrow_mut()
        .init_uniform_buffers(vec!["uniforms_", "camera", "background"])?;

    let camera = Rc::new(RefCell::new(camera::CameraController::default()));
    let input = set_input_handler(canvas.clone());

    let test_uniform = Rc::new(RefCell::new(Uniform {
        size0: 0.01,
        size1: 0.5,
        _pad0: [0, 0],
    }));

    let test_shader = Rc::new(RefCell::new(TestShader::new(ShaderController::new(
        shared.clone(),
    ))?));
    test_shader.borrow_mut().init().await?;

    let entity_shader = EntityShader::new(shared.clone())?;
    entity_shader.borrow_mut().init_textures().await?;

    let background_shader = BackgroundShader::new(shared.clone())?;
    background_shader.borrow_mut().init_textures().await?;

    let mut scenes = Scenes {
        scene_manager: scene_manager.clone(),
        batting: SampleScene::new(SampleSceneContext {
            scene_manager: scene_manager.clone(),
            entity_shader: entity_shader.clone(),
            background_shader: background_shader.clone(),
            camera: camera.clone(),
            input: input.clone(),
            shared: shared.clone(),
        }),
        test: TestScene::new(TestSceneContext {
            test_shader: test_shader.clone(),
            test_uniform: test_uniform.clone(),
            shared: shared.clone(),
        }),
    };

    ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
    ctx.depth_func(WebGl2RenderingContext::LEQUAL);
    ctx.enable(WebGl2RenderingContext::BLEND);
    ctx.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    start_loop(window.clone(), move |time| {
        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear_depth(1.0);
        ctx.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        scene_manager.borrow_mut().type_ = SceneType::Batting;
        // if (time / 10000.0) % 2. < 1. {
        // } else {
        //     scene_manager.borrow_mut().type_ = SceneType::Test;
        // }

        scenes.render(time)?;
        input.borrow_mut().resolve();
        Ok(())
    })?;

    Ok(())
}

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

use crate::entities::get_current_instance_value;
use crate::entities::sample_batter::SampleEntity;
use crate::input::set_input_handler;
use crate::scheduler::start_loop;
use crate::shader::{ConvertArrayView, ShaderController, SharedContext};
use crate::shaders::entity_shader::EntityShader;
use crate::shaders::test::TestShader;

use core::cell::RefCell;
use num_traits::cast::ToPrimitive;
// use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Performance, WebGl2RenderingContext};
use webgl_matrix::{Mat4, Matrix};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut PERFORMANCE: Option<Rc<Performance>> = None;

#[repr(C)]
struct Uniform {
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

    let shared = SharedContext::new(doc.clone(), ctx.clone());
    shared
        .borrow_mut()
        .init_uniform_buffers(vec!["uniforms_", "camera"])?;

    let camera = Rc::new(RefCell::new(camera::CameraController::default()));
    let input = set_input_handler(canvas.clone());

    let test = Rc::new(RefCell::new(SampleEntity {
        start_at: 0.,
        duration: 600.,
        model: *Mat4::identity().scale(500.).translate(&[0., 0., -0.2]),
    }));

    let mut uniform = Uniform {
        size0: 0.01,
        size1: 0.5,
        _pad0: [0, 0],
    };

    let mut test_shader = TestShader::new(ShaderController::new(shared.clone()))?;
    test_shader.init().await?;

    let entity_shader = EntityShader::new(shared.clone())?;
    entity_shader.borrow_mut().init_textures().await?;

    ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
    ctx.depth_func(WebGl2RenderingContext::LEQUAL);
    ctx.enable(WebGl2RenderingContext::BLEND);
    ctx.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );
    start_loop(window.clone(), move |now| {
        let shared = shared.borrow();
        let mut input = input.borrow_mut();
        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear_depth(1.0);
        ctx.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        uniform.size0 = now / 2000.0;
        unsafe {
            shared.uniform_buffer_data("uniforms_", &uniform)?;
        }
        // test_shader.draw(now)?;

        let mut camera = camera.borrow_mut();
        camera.view.position = [0., 0., 0.];
        // let t = ((now % 20000.0) / 20000.0) * 2. * PI;
        // camera.view.direction = [t.cos(), 0., t.sin()];
        camera.view.direction = [0., 0., -1.];
        camera.refresh();
        unsafe {
            shared.uniform_buffer_data("camera", &camera.camera)?;
        }

        let mut shader = entity_shader.borrow_mut();
        shader.clear();
        let mut test = test.borrow_mut();
        if let Some(click) = &input.clicked {
            test.start_at = click.timestamp;
        }
        test.model = *Mat4::identity().scale(500.).translate(&[
            input.curr_coord.0 * 2.35 - 1.4,
            -input.curr_coord.1 * 2.3 - 0.,
            -0.2,
        ]);
        shader
            .instances
            .push(get_current_instance_value(&*test, now));
        shader.draw(now)?;

        input.resolve();
        Ok(())
    })?;

    Ok(())
}

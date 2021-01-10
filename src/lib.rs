mod log;
mod scheduler;
mod shader;
mod shaders;
mod utils;

use crate::scheduler::start_loop;
use crate::shader::buffer_data::ConvertArrayView;
use crate::shader::Shader;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, WebGl2RenderingContext};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[repr(C)]
struct Uniform {
    size0: f32,
    size1: f32,
    _pad0: [u32; 2],
}
impl ConvertArrayView for Uniform {}

static mut DOC: Option<Document> = None;
static mut CTX: Option<WebGl2RenderingContext> = None;

fn init_static() -> Result<(&'static Document, &'static WebGl2RenderingContext), JsValue> {
    let doc: &'static Document = unsafe {
        DOC = Some(web_sys::window().unwrap().document().unwrap());
        DOC.as_ref().unwrap()
    };
    let ctx: &'static WebGl2RenderingContext = unsafe {
        let canvas = doc.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        CTX = Some(
            canvas
                .get_context("webgl2")?
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()?,
        );
        CTX.as_ref().unwrap()
    };
    Ok((doc, ctx))
}

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    let (doc, ctx) = init_static()?;

    let mut uniform = Uniform {
        size0: 0.01,
        size1: 0.5,
        _pad0: [0, 0],
    };

    let mut test_shader = shaders::test::TestShader::new(Shader::new(doc, ctx))?;
    test_shader.init().await?;

    start_loop(move |now| {
        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        uniform.size0 = now / 2000.0;
        unsafe {
            test_shader
                .shader
                .uniform_buffer_data("uniforms_", &uniform)?;
        }
        test_shader.draw(now)?;
        Ok(())
    })?;

    Ok(())
}

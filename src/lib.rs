mod geometry;
mod shader;
mod utils;

use crate::shader::ext::{AngleInstancedArrays, ExtensionGetter};
use crate::shader::Shader;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;
use webgl_matrix::Vec4;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("カッシーおはよう！");
}

#[repr(C)]
struct Vertex {
    position: Vec4,
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let ctx = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let mut shader = Shader::<Vertex, ()>::new(&ctx);
    shader.compile(
        vec!["position"],
        vec![],
        r#"
        attribute vec4 position;
        void main() {
            gl_Position = position;
        }
        "#,
        r#"
        void main() {
            gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
        "#,
    )?;
    let ext_angle = AngleInstancedArrays::get(&ctx)?;
    shader.set_vertex_layout(&ext_angle, vec![("position", 4)])?;

    let vertices = vec![
        Vertex {
            position: [-0.7, -0.7, 0.5, 1.0],
        },
        Vertex {
            position: [0.7, -0.7, 0.5, 1.0],
        },
        Vertex {
            position: [0.0, 0.7, 0.5, 1.0],
        },
    ];
    unsafe {
        shader.vertex_buffer_data(&vertices);
    }

    ctx.clear_color(0.0, 0.0, 0.0, 1.0);
    ctx.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    ctx.use_program(shader.program.program.as_ref());
    ctx.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, vertices.len() as i32);

    Ok(())
}

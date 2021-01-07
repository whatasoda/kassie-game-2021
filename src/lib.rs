mod scheduler;
mod shader;
mod utils;

use crate::scheduler::start_loop;
use crate::shader::buffer::ConvertArrayView;
use crate::shader::Shader;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;
use webgl_matrix::Vec3;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[repr(C)]
struct Vertex {
    position: Vec3,
    uv: [f32; 2],
}

#[repr(C)]
struct Uniform {
    size0: f32,
    size1: f32,
    _pad0: [u32; 2],
}
impl ConvertArrayView for Uniform {}

static mut CTX: Option<WebGl2RenderingContext> = None;

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    unsafe {
        CTX = Some(
            canvas
                .get_context("webgl2")?
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()?,
        );
    }

    let ctx: &'static WebGl2RenderingContext = unsafe { CTX.as_ref().unwrap() };

    let mut shader = Shader::<Vertex, (), Uniform>::new(ctx);
    shader.compile(
        vec!["position", "uv"],
        r#"#version 300 es
        layout (location = 0) in vec3 position;
        layout (location = 1) in vec2 uv;

        out vec2 v_uv;

        void main() {
            gl_Position = vec4(position, 1.0);
            v_uv = uv;
        }
        "#,
        r#"#version 300 es
        precision highp float;
        layout (std140) uniform uniforms_ {
            float size0;
            float size1;
        };
        uniform sampler2D tex0;

        in vec2 v_uv;

        out vec4 outColor;

        void main() {
            vec4 tex_color = texture(tex0, v_uv);
            vec3 rainbow = min((1.0 - tex_color.a) * vec3(gl_FragCoord.xy / size0 / size1, 0.0), vec3(1.0));
            outColor = vec4((tex_color.xyz * tex_color.a) + rainbow, 1.0);
        }
        "#,
    )?;
    shader.init_buffers()?;
    shader.set_vertex_layout(vec![("position", 3), ("uv", 2)])?;

    let vertices = vec![
        Vertex {
            position: [-0.7, -0.7, 0.5],
            uv: [0.0, 1.0],
        },
        Vertex {
            position: [0.7, -0.7, 0.5],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [-0.7, 0.7, 0.5],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [-0.7, 0.7, 0.5],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [0.7, -0.7, 0.5],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [0.7, 0.7, 0.5],
            uv: [1.0, 0.0],
        },
    ];
    let mut uniform = Uniform {
        size0: 200.0,
        size1: 0.5,
        _pad0: [0, 0],
    };

    ctx.use_program(shader.program.program.as_ref());
    shader
        .create_texture(&document, "sample_texture.png")
        .await?;

    start_loop(move |now| {
        ctx.use_program(shader.program.program.as_ref());
        shader.bind_texture(0, "sample_texture.png")?;
        uniform.size0 = now / 20.0;
        unsafe {
            shader.vertex_buffer_data(&vertices)?;
            shader.uniform_buffer_data(&uniform)?;
        }
        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        shader.prepare_draw()?;
        ctx.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);
        Ok(())
    })?;

    Ok(())
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

// // Multiple arguments too!
// #[wasm_bindgen(js_namespace = console, js_name = log)]
// fn log_many(a: &str, b: &str);
}

mod log;
mod scheduler;
mod shader;
mod utils;

use crate::scheduler::start_loop;
use crate::shader::buffer_data::ConvertArrayView;
use crate::shader::Shader;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;
use webgl_matrix::{Mat4, Matrix, Vec3};

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
struct Instance {
    model: Mat4,
    mask: Vec3,
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

    let ctx: &'static WebGl2RenderingContext = unsafe {
        CTX = Some(
            canvas
                .get_context("webgl2")?
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()?,
        );
        CTX.as_ref().unwrap()
    };

    let mut shader = Shader::<Vertex, Instance>::new(ctx);
    shader.compile(
        r#"#version 300 es
        layout (location = 0) in vec3 position;
        layout (location = 1) in vec2 uv;
        layout (location = 2) in mat4 model;
        layout (location = 6) in vec3 mask;

        out vec2 v_uv;
        out vec3 v_mask;

        void main() {
            gl_Position = model * vec4(position, 1.0);
            v_uv = uv;
            v_mask = mask;
        }
        "#,
        r#"#version 300 es
        precision highp float;
        uniform sampler2D tex0;
        layout (std140) uniform uniforms_ {
            float size0;
            float size1;
        };

        in vec2 v_uv;
        in vec3 v_mask;

        out vec4 outColor;

        void main() {
            vec4 tex_color = texture(tex0, v_uv);
            vec3 rainbow = min((1.0 - tex_color.a) * vec3(v_uv / size0 / size1, 0.0), vec3(1.0));
            outColor = vec4(((tex_color.xyz * tex_color.a) + rainbow) * v_mask, 1.0);
        }
        "#,
    )?;
    shader.bind_uniform_blocks(vec!["uniforms_"])?;
    shader.set_vertex_layout(vec![("position", 3), ("uv", 2)])?;
    shader.set_instance_layout(vec![("model", 16), ("mask", 3)])?;

    let vertices = vec![
        Vertex {
            position: [-0.4, -0.4, 0.5],
            uv: [0.0, 1.0],
        },
        Vertex {
            position: [0.4, -0.4, 0.5],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [-0.4, 0.4, 0.5],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [-0.4, 0.4, 0.5],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [0.4, -0.4, 0.5],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [0.4, 0.4, 0.5],
            uv: [1.0, 0.0],
        },
    ];
    let instances = vec![
        Instance {
            model: *Mat4::identity().translate(&[-0.4, -0.4, 0.1]),
            mask: [1.0, 0.0, 0.0],
        },
        Instance {
            model: *Mat4::identity().translate(&[0.4, 0.4, 0.1]),
            mask: [0.0, 1.0, 0.0],
        },
        Instance {
            model: *Mat4::identity().translate(&[-0.4, 0.4, 0.1]),
            mask: [0.0, 0.0, 1.0],
        },
        Instance {
            model: *Mat4::identity().translate(&[0.4, -0.4, 0.1]),
            mask: [1.0, 1.0, 1.0],
        },
    ];
    let mut uniform = Uniform {
        size0: 0.01,
        size1: 0.5,
        _pad0: [0, 0],
    };
    shader.activate();

    shader
        .create_texture(&document, "sample_texture.png")
        .await?;
    unsafe {
        shader.vertex_buffer_data(&vertices)?;
        shader.instance_buffer_data(&instances)?;
    }

    start_loop(move |now| {
        shader.activate();
        shader.bind_texture(0, "sample_texture.png")?;
        shader.attach_texture(0, 0)?;
        uniform.size0 = now / 2000.0;
        unsafe {
            shader.uniform_buffer_data("uniforms_", &uniform)?;
        }
        shader.prepare_array_buffers()?;
        shader.preapre_uniform_blocks()?;
        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        ctx.draw_arrays_instanced(
            WebGl2RenderingContext::TRIANGLES,
            0,
            vertices.len() as i32,
            instances.len() as i32,
        );
        Ok(())
    })?;

    Ok(())
}

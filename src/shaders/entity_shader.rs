use crate::shader::{Shader, ShaderController, ShaderImpl};
use crate::ConvertArrayView;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;
use webgl_matrix::Mat4;

pub type EntityShader = Shader<EntityShaderImpl, Instance>;
pub struct EntityShaderImpl {}

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
pub struct Instance {
    pub model: Mat4,
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub pos_offset: [f32; 2],
}

impl ConvertArrayView for [Vertex; 6] {}
static VERTICES: [Vertex; 6] = [
    Vertex {
        position: [-1., -1.],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1., -1.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-1., 1.],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-1., 1.],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1., -1.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1., 1.],
        uv: [1.0, 0.0],
    },
];

const VERT: &'static str = r#"#version 300 es
layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

layout (location = 2) in mat4 model;
layout (location = 6) in vec2 uv_offset;
layout (location = 7) in vec2 uv_scale;
layout (location = 8) in vec2 pos_offset;

out vec2 v_uv;

layout (std140) uniform camera {
    mat4 vpMatrix;
};

void main() {
    v_uv = uv_scale * uv + uv_offset;
    mat4 mvp = vpMatrix * model;
    gl_Position = mvp * vec4((position + pos_offset) * uv_scale, 0.0, 1.0);
}
"#;

const FRAG: &'static str = r#"#version 300 es
precision highp float;
uniform sampler2D tex0;

in vec2 v_uv;

out vec4 outColor;

void main() {
    vec4 tex_color = texture(tex0, v_uv);
    outColor = tex_color;
    float alpha = tex_color.a;
    float depth = gl_FragCoord.z;
    // TODO: correct formula
    gl_FragDepth = (alpha * depth) + (1.0 - alpha);
}
"#;

impl ShaderImpl<Instance> for EntityShaderImpl {
    const INSTANCE_CAPACITY: Option<usize> = None;

    fn new() -> Self {
        Self {}
    }

    fn get_static_instances(&self) -> Option<Vec<Instance>> {
        None
    }

    fn init(&self, shader: &mut ShaderController) -> Result<(), JsValue> {
        shader.compile(VERT, FRAG)?;
        shader.bind_uniform_blocks(vec!["camera"])?;
        shader.layout_buffer::<Vertex>("vertex", 0, vec![("position", 2), ("uv", 2)])?;
        shader.layout_buffer::<Instance>(
            "instance",
            1,
            vec![
                ("model", 16),
                ("uv_offset", 2),
                ("uv_scale", 2),
                ("pos_offset", 2),
            ],
        )?;
        unsafe {
            shader.buffer_data_static("vertex", &VERTICES)?;
        }
        Ok(())
    }

    fn get_texture_map(&self) -> Vec<(u32, u32, &'static str)> {
        vec![(0, 0, "entities0.png")]
    }

    fn draw(&self, ctx: &WebGl2RenderingContext, _: f32, instance_len: i32) {
        ctx.draw_arrays_instanced(
            WebGl2RenderingContext::TRIANGLES,
            0,
            VERTICES.len() as i32,
            instance_len,
        );
    }
}

use crate::shader::{ConvertArrayView, Shader, ShaderController, ShaderImpl};
use webgl_matrix::Mat4;

use num_traits::ToPrimitive;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

pub type BackgroundShader = Shader<BackgroundShaderImpl, Instance>;
pub struct BackgroundShaderImpl {}

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
pub struct Instance {
    pos_offset: [f32; 2],
    uv_offset: [f32; 2],
}

#[repr(C)]
pub struct Background {
    pub model: Mat4,
}
impl ConvertArrayView for Background {}

const UNIT: f32 = 0.1;

impl ConvertArrayView for [Vertex; 6] {}
static VERTICES: [Vertex; 6] = [
    Vertex {
        position: [0., 0.],
        uv: [0.0, UNIT],
    },
    Vertex {
        position: [UNIT, 0.],
        uv: [UNIT, UNIT],
    },
    Vertex {
        position: [0., UNIT],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0., UNIT],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [UNIT, 0.],
        uv: [UNIT, UNIT],
    },
    Vertex {
        position: [UNIT, UNIT],
        uv: [UNIT, 0.0],
    },
];

const VERT: &'static str = r#"#version 300 es
layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

layout (location = 2) in vec2 pos_offset;
layout (location = 3) in vec2 uv_offset;

out vec2 v_uv;

layout (std140) uniform background {
    mat4 model;
};

layout (std140) uniform camera {
    mat4 vpMatrix;
};

void main() {
    v_uv = uv + uv_offset;
    vec2 pos = position + pos_offset;
    gl_Position = vpMatrix * model * vec4(pos.x, 0.0, pos.y, 1.0);
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
}
"#;

impl ShaderImpl<Instance> for BackgroundShaderImpl {
    const INSTANCE_CAPACITY: Option<usize> = Some(100);

    fn new() -> Self {
        Self {}
    }

    fn get_static_instances(&self) -> Option<Vec<Instance>> {
        let mut instances = Vec::<Instance>::with_capacity(100);
        for x in 0..10 {
            let u = x.to_f32().unwrap() * UNIT;
            let x = (x - 5).to_f32().unwrap() * UNIT;
            for z in 0..10 {
                let v = (9 - z).to_f32().unwrap() * UNIT;
                let z = (z - 5).to_f32().unwrap() * UNIT;
                instances.push(Instance {
                    pos_offset: [x, z],
                    uv_offset: [u, v],
                });
            }
        }
        Some(instances)
    }

    fn init(&self, shader: &mut ShaderController) -> Result<(), JsValue> {
        shader.compile(VERT, FRAG)?;
        shader.bind_uniform_blocks(vec!["camera", "background"])?;
        shader.layout_buffer::<Vertex>("vertex", 0, vec![("position", 2), ("uv", 2)])?;
        shader.layout_buffer::<Instance>(
            "instance",
            1,
            vec![("pos_offset", 2), ("uv_offset", 2)],
        )?;
        unsafe {
            shader.buffer_data_static("vertex", &VERTICES)?;
        }
        Ok(())
    }

    fn get_texture_map(&self) -> Vec<(u32, u32, &'static str)> {
        vec![(0, 0, "background.png")]
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

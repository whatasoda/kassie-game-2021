use crate::shader::Shader;
use crate::ConvertArrayView;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;
use webgl_matrix::{Mat4, Matrix};

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
struct Instance {
    model: Mat4,
}

pub struct EntityShader {
    pub shader: Shader,
    instances: Option<Vec<Instance>>,
}

impl ConvertArrayView for [Vertex; 6] {}
static VERTICES: [Vertex; 6] = [
    Vertex {
        position: [-1., 0.],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1., 0.],
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
        position: [1., 0.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1., 1.],
        uv: [1.0, 0.0],
    },
];

impl EntityShader {
    const VERT: &'static str = r#"#version 300 es
layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;
layout (location = 2) in mat4 model;

out vec2 v_uv;

layout (std140) uniform camera {
    mat4 vpMatrix;
};

void main() {
    v_uv = uv;
    gl_Position = vpMatrix * (model * vec4(position, 0.0, 1.0));
}
"#;
    const FRAG: &'static str = r#"#version 300 es
precision highp float;
uniform sampler2D tex0;
layout (std140) uniform uniforms_ {
    float size0;
    float size1;
};

in vec2 v_uv;

out vec4 outColor;

void main() {
    vec4 tex_color = texture(tex0, v_uv);
    outColor = tex_color.rgba;
}
"#;

    pub fn new(mut shader: Shader) -> Result<Self, JsValue> {
        VERTICES.as_ptr();
        shader.compile(Self::VERT, Self::FRAG)?;
        shader.bind_uniform_blocks(vec!["uniforms_", "camera"])?;
        shader.layout_buffer::<Vertex>("vertex", 0, vec![("position", 2), ("uv", 2)])?;
        shader.layout_buffer::<Instance>("instance", 1, vec![("model", 16)])?;
        unsafe {
            shader.buffer_data_static("vertex", &VERTICES)?;
        }

        Ok(Self {
            shader,
            instances: None,
        })
    }

    pub async fn init(&mut self) -> Result<(), JsValue> {
        let shader = &mut self.shader;
        self.instances = Some(vec![
            Instance {
                model: *Mat4::identity().translate(&[0., 0., -100.3]).scale(10.),
            },
            Instance {
                model: *Mat4::identity().translate(&[0., -10., 0.]),
            },
            Instance {
                model: *Mat4::identity().translate(&[-0.4, 0.4, 12.1]),
            },
            Instance {
                model: *Mat4::identity().translate(&[0.4, -0.4, 0.1]),
            },
        ]);

        shader.activate();
        shader.create_texture("sample_texture.png").await?;
        unsafe {
            shader.buffer_data_dynamic("instance", self.instances.as_ref().unwrap())?;
        }
        Ok(())
    }

    pub fn draw(&mut self, _: f32) -> Result<(), JsValue> {
        let shader = &mut self.shader;
        shader.activate();
        shader.bind_texture(0, "sample_texture.png")?;
        shader.attach_texture(0, 0)?;

        shader.prepare_array_buffers()?;
        shader.preapre_uniform_blocks()?;
        shader.ctx.draw_arrays_instanced(
            WebGl2RenderingContext::TRIANGLES,
            0,
            VERTICES.len() as i32,
            self.instances.as_ref().unwrap().len() as i32,
        );
        Ok(())
    }
}

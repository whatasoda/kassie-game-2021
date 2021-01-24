use crate::shader::ConvertArrayView;
use crate::shader::Shader;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;
use webgl_matrix::{Mat4, Matrix, Vec3};

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

pub struct TestShader {
    pub shader: Shader,
    instances: Option<Vec<Instance>>,
}

impl ConvertArrayView for [Vertex; 6] {}
static VERTICES: [Vertex; 6] = [
    Vertex {
        position: [-0.4, -0.4, 0.],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.4, -0.4, 0.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-0.4, 0.4, 0.],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-0.4, 0.4, 0.],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0.4, -0.4, 0.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.4, 0.4, 0.],
        uv: [1.0, 0.0],
    },
];

impl TestShader {
    const VERT: &'static str = r#"#version 300 es
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
"#;
    const FRAG: &'static str = r#"#version 300 es
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
"#;
    pub fn new(mut shader: Shader) -> Result<Self, JsValue> {
        VERTICES.as_ptr();
        shader.compile(Self::VERT, Self::FRAG)?;
        shader.bind_uniform_blocks(vec!["uniforms_"])?;
        shader.layout_buffer::<Vertex>("vertex", 0, vec![("position", 3), ("uv", 2)])?;
        shader.layout_buffer::<Instance>("instance", 1, vec![("model", 16), ("mask", 3)])?;
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
                model: *Mat4::identity().translate(&[-0.4, -0.4, 0.999]),
                mask: [1.0, 0.0, 0.0],
            },
            Instance {
                model: *Mat4::identity().translate(&[0.4, 0.4, 0.999]),
                mask: [0.0, 1.0, 0.0],
            },
            Instance {
                model: *Mat4::identity().translate(&[-0.4, 0.4, 0.999]),
                mask: [0.0, 0.0, 1.0],
            },
            Instance {
                model: *Mat4::identity().translate(&[0.4, -0.4, 0.999]),
                mask: [1.0, 1.0, 1.0],
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
        // shader.shared.borrow().ctx.draw_arrays_instanced(
        //     WebGl2RenderingContext::TRIANGLES,
        //     0,
        //     VERTICES.len() as i32,
        //     self.instances.as_ref().unwrap().len() as i32,
        // );
        Ok(())
    }
}

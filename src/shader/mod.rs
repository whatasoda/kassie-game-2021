#[macro_use]
mod macros;

mod array_buffer;
pub mod buffer_data;
mod compile;
mod texture;
mod uniform_buffer;

use web_sys::WebGl2RenderingContext;

pub struct Shader {
    ctx: &'static WebGl2RenderingContext,
    program: compile::Program,
    arrays: array_buffer::ArrayBuffers,
    uniforms: uniform_buffer::UniformBlocks,
    textures: texture::Textures,
}

impl Shader {
    pub fn new(ctx: &'static WebGl2RenderingContext) -> Self {
        Shader {
            ctx,
            program: compile::Program::empty(),
            arrays: array_buffer::ArrayBuffers::empty(),
            uniforms: uniform_buffer::UniformBlocks::empty(),
            textures: texture::Textures::empty(),
        }
    }
}

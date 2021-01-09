#[macro_use]
mod macros;

mod array_buffer;
pub mod buffer_data;
mod compile;
mod texture;
mod uniform_buffer;

use web_sys::WebGl2RenderingContext;

pub struct Shader<'ctx, V, I>
where
    V: Sized,
    I: Sized,
{
    ctx: &'ctx WebGl2RenderingContext,
    program: compile::Program,
    buffers: array_buffer::ArrayBuffers<V, I>,
    uniforms: uniform_buffer::UniformBlocks,
    textures: texture::Textures,
}

impl<'ctx, V, I> Shader<'ctx, V, I> {
    pub fn new(ctx: &'ctx WebGl2RenderingContext) -> Self {
        Shader {
            ctx,
            program: compile::Program::empty(),
            buffers: array_buffer::ArrayBuffers::empty(),
            uniforms: uniform_buffer::UniformBlocks::empty(),
            textures: texture::Textures::empty(),
        }
    }
}

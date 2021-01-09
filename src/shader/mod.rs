#[macro_use]
mod macros;

pub mod buffer;
mod compile;
mod texture;

use web_sys::WebGl2RenderingContext;

pub struct Shader<'ctx, V, I, U>
where
    V: Sized,
    I: Sized,
    U: Sized,
{
    ctx: &'ctx WebGl2RenderingContext,
    pub program: compile::Program,
    buffers: buffer::Buffers<V, I, U>,
    textures: texture::Textures,
}

impl<'ctx, V, I, U> Shader<'ctx, V, I, U> {
    pub fn new(ctx: &'ctx WebGl2RenderingContext) -> Self {
        Shader {
            ctx,
            program: compile::Program::empty(),
            buffers: buffer::Buffers::empty(),
            textures: texture::Textures::empty(),
        }
    }
}

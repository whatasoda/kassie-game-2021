mod buffer;
mod compile;
// pub mod ext;
mod texture;

use web_sys::WebGl2RenderingContext;

pub struct Shader<'ctx, V, I>
where
    V: Sized,
    I: Sized,
{
    ctx: &'ctx WebGl2RenderingContext,
    pub program: compile::Program,
    buffers: buffer::Buffers<V, I>,
    textures: texture::Textures,
}

impl<'ctx, V, I> Shader<'ctx, V, I> {
    pub fn new(ctx: &'ctx WebGl2RenderingContext) -> Self {
        Shader {
            ctx,
            program: compile::Program::empty(),
            buffers: buffer::Buffers::empty(),
            textures: texture::Textures::empty(),
        }
    }
}

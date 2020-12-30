mod buffer;
mod compile;
pub mod ext;

use web_sys::WebGlRenderingContext;

pub struct Shader<'ctx, V, I>
where
    V: Sized,
    I: Sized,
{
    ctx: &'ctx WebGlRenderingContext,
    pub program: compile::Program,
    pub buffers: buffer::Buffers<V, I>,
}

impl<'ctx, V, I> Shader<'ctx, V, I> {
    pub fn new(ctx: &'ctx WebGlRenderingContext) -> Self {
        Shader {
            ctx,
            program: compile::Program::empty(),
            buffers: buffer::Buffers::empty(),
        }
    }
}

#[macro_use]
mod macros;

mod array_buffer;
pub mod buffer_data;
mod compile;
mod texture;
mod uniform_buffer;

use std::rc::Rc;
use web_sys::{Document, WebGl2RenderingContext};

pub struct Shader {
    doc: Rc<Document>,
    pub ctx: Rc<WebGl2RenderingContext>,
    program: compile::Program,
    arrays: array_buffer::ArrayBuffers,
    uniforms: uniform_buffer::UniformBlocks,
    textures: texture::Textures,
}

impl Shader {
    pub fn new(doc: Rc<Document>, ctx: Rc<WebGl2RenderingContext>) -> Self {
        Shader {
            doc,
            ctx,
            program: compile::Program::empty(),
            arrays: array_buffer::ArrayBuffers::empty(),
            uniforms: uniform_buffer::UniformBlocks::empty(),
            textures: texture::Textures::empty(),
        }
    }
}

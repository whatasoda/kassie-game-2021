use super::buffer_data::{buffer_data, ConvertArrayView};
use super::Shader;
use wasm_bindgen::JsValue;

use std::collections::HashMap;

use web_sys::{WebGl2RenderingContext, WebGlBuffer};

static mut UNIFORM_BUFFERS: Option<HashMap<&'static str, WebGlBuffer>> = None;

pub struct UniformBlocks {
    bindings: HashMap<&'static str, u32>,
    buffers: &'static mut HashMap<&'static str, WebGlBuffer>,
}

impl UniformBlocks {
    pub fn empty() -> Self {
        let buffers = unsafe {
            if UNIFORM_BUFFERS.is_none() {
                UNIFORM_BUFFERS = Some(HashMap::new());
            }
            UNIFORM_BUFFERS.as_mut().unwrap()
        };
        Self {
            bindings: HashMap::new(),
            buffers,
        }
    }
}

impl<V, I> Shader<'_, V, I> {
    fn get_uniform_buffer_by_name(&self, name: &'static str) -> Result<&WebGlBuffer, JsValue> {
        Ok(self
            .uniforms
            .buffers
            .get(name)
            .ok_or("uniform buffer not found")?)
    }

    pub fn preapre_uniform_blocks(&self) -> Result<(), JsValue> {
        let mut bindings = self.uniforms.bindings.iter();
        while let Some((name, binding)) = bindings.next() {
            let buffer = self.get_uniform_buffer_by_name(name)?;
            self.ctx.bind_buffer_base(
                WebGl2RenderingContext::UNIFORM_BUFFER,
                *binding,
                Some(buffer),
            );
        }
        Ok(())
    }

    pub fn bind_uniform_blocks(&mut self, blocks: Vec<&'static str>) -> Result<(), String> {
        // TODO: avoid being called over twice
        let mut blocks = blocks.into_iter();
        let mut curr_binding = 0;
        while let Some(name) = blocks.next() {
            let program = self.program.program.as_ref().unwrap();
            let index = self.ctx.get_uniform_block_index(program, name);
            if index == 0xffffffff {
                return Err(format!("uniform block {} not found", name));
            }
            let binding = curr_binding;
            self.ctx.uniform_block_binding(program, index, binding);
            self.uniforms.bindings.insert(name, binding);

            if !self.uniforms.buffers.contains_key(name) {
                let buffer = self.ctx.create_buffer().ok_or("failed to create buffer")?;
                self.uniforms.buffers.insert(name, buffer);
            }
            curr_binding += 1;
        }
        Ok(())
    }

    pub unsafe fn uniform_buffer_data<T>(&self, name: &'static str, data: &T) -> Result<(), JsValue>
    where
        T: ConvertArrayView,
    {
        let buffer = self.get_uniform_buffer_by_name(name)?;
        buffer_data(
            &self.ctx,
            WebGl2RenderingContext::UNIFORM_BUFFER,
            Some(buffer),
            data,
            true,
        );
        Ok(())
    }
}

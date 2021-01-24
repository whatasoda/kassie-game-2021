use super::buffer_data::{buffer_data, ConvertArrayView};
use super::{ShaderController, SharedContext};

use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

pub type UniformBuffers = HashMap<&'static str, WebGlBuffer>;

impl SharedContext {
    fn_ensure_hashmap!(
        [fn ensure_uniform_buffer],
        [uniform_buffers],
        if_none: "uniform buffer uninitialized",
        if_some: "uniform buffer already exists",
    );

    pub fn init_uniform_buffers(&mut self, names: Vec<&'static str>) -> Result<(), JsValue> {
        for name in names {
            if !self.uniform_buffers.contains_key(name) {
                let buffer = self.ctx.create_buffer().ok_or("failed to create buffer")?;
                self.uniform_buffers.insert(name, buffer);
            }
        }
        Ok(())
    }

    pub unsafe fn uniform_buffer_data<T>(&self, name: &'static str, data: &T) -> Result<(), JsValue>
    where
        T: ConvertArrayView,
    {
        self.ensure_uniform_buffer(name, Some(()))?;
        let buffer = self.uniform_buffers.get(name).unwrap();
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

pub struct UniformBlocks {
    is_ready: bool,
    bindings: HashMap<&'static str, u32>,
}

impl UniformBlocks {
    pub fn empty() -> Self {
        Self {
            is_ready: false,
            bindings: HashMap::new(),
        }
    }
}

impl ShaderController {
    pub fn preapre_uniform_blocks(&self) -> Result<(), JsValue> {
        let shared = self.shared.borrow();
        let mut bindings = self.uniforms.bindings.iter();
        while let Some((name, binding)) = bindings.next() {
            shared.ensure_uniform_buffer(name, Some(()))?;
            let buffer = shared.uniform_buffers.get(name).unwrap();
            shared.ctx.bind_buffer_base(
                WebGl2RenderingContext::UNIFORM_BUFFER,
                *binding,
                Some(buffer),
            );
        }
        Ok(())
    }

    pub fn bind_uniform_blocks(&mut self, blocks: Vec<&'static str>) -> Result<(), String> {
        if self.uniforms.is_ready {
            return Err(String::from("cannot bind uniform blocks over twice"));
        }
        let shared = self.shared.borrow();
        let mut blocks = blocks.into_iter();
        let mut curr_binding = 0;
        while let Some(name) = blocks.next() {
            shared.ensure_uniform_buffer(name, Some(()))?;
            let program = self.program.program.as_ref().unwrap();
            let index = shared.ctx.get_uniform_block_index(program, name);
            if index == 0xffffffff {
                return Err(format!("uniform block {} not found", name));
            }
            let binding = curr_binding;
            shared.ctx.uniform_block_binding(program, index, binding);
            self.uniforms.bindings.insert(name, binding);

            curr_binding += 1;
        }
        self.uniforms.is_ready = true;
        Ok(())
    }
}

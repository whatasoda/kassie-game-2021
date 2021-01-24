use super::buffer_data::{buffer_data, ConvertArrayView};
use super::ShaderController;

use std::cmp::min;
use std::collections::HashMap;
use std::mem;
use web_sys::WebGlProgram;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

pub struct ArrayBuffers {
    vao: Option<WebGlVertexArrayObject>,
    buffers: HashMap<&'static str, WebGlBuffer>,
}

impl ArrayBuffers {
    pub fn empty() -> Self {
        ArrayBuffers {
            vao: None,
            buffers: HashMap::new(),
        }
    }
}

impl ShaderController {
    fn_ensure_option!(
        [fn ensure_vao],
        arrays.vao,
        if_none: "VAO uninitialized",
        if_some: "VAO already exists",
    );
    fn_ensure_hashmap!(
        [fn ensure_array_buffer],
        [arrays.buffers],
        if_none: "array buffer uninitialized",
        if_some: "array buffer already exists",
    );

    pub(super) fn init_buffers(&mut self) -> Result<(), String> {
        self.ensure_vao(None)?;
        let shared = self.shared.borrow();

        self.arrays.vao = Some(
            shared
                .ctx
                .create_vertex_array()
                .ok_or("failed to create vao")?,
        );
        Ok(())
    }

    pub fn prepare_array_buffers(&self) -> Result<(), String> {
        self.ensure_vao(Some(()))?;
        let shared = self.shared.borrow();

        shared.ctx.bind_vertex_array(self.arrays.vao.as_ref());
        Ok(())
    }

    pub fn layout_buffer<T>(
        &mut self,
        name: &'static str,
        divisor: u32,
        layout: Vec<(&'static str, i32)>,
    ) -> Result<(), String>
    where
        T: Sized,
    {
        self.ensure_vao(Some(()))?;
        self.ensure_program(Some(()))?;
        self.ensure_array_buffer(name, None)?;
        let shared = self.shared.borrow();

        shared.ctx.bind_vertex_array(self.arrays.vao.as_ref());
        let buffer = create_buffer_with_layout::<T>(
            &shared.ctx,
            &self.program.program.as_ref().unwrap(),
            divisor,
            layout,
        )?;
        shared.ctx.bind_vertex_array(None);
        self.arrays.buffers.insert(name, buffer);
        Ok(())
    }

    pub unsafe fn buffer_data_static<T>(&self, name: &'static str, data: &T) -> Result<(), String>
    where
        T: ConvertArrayView,
    {
        self.buffer_data(name, data, false)
    }

    pub unsafe fn buffer_data_dynamic<T>(&self, name: &'static str, data: &T) -> Result<(), String>
    where
        T: ConvertArrayView,
    {
        self.buffer_data(name, data, true)
    }

    unsafe fn buffer_data<T>(
        &self,
        name: &'static str,
        data: &T,
        is_dynamic: bool,
    ) -> Result<(), String>
    where
        T: ConvertArrayView,
    {
        self.ensure_vao(Some(()))?;
        self.ensure_array_buffer(name, Some(()))?;
        let shared = self.shared.borrow();

        shared.ctx.bind_vertex_array(self.arrays.vao.as_ref());
        let buffer = self.arrays.buffers.get(name).unwrap();
        buffer_data(
            &shared.ctx,
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(buffer),
            data,
            is_dynamic,
        );
        shared.ctx.bind_vertex_array(None);
        Ok(())
    }
}

fn create_buffer_with_layout<T>(
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
    divisor: u32,
    layout: Vec<(&str, i32)>,
) -> Result<WebGlBuffer, String>
where
    T: Sized,
{
    let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    let byte_length = mem::size_of::<T>() as i32;
    let mut byte_offset = 0;
    let mut iter = layout.into_iter();
    while let Some((name, total_stride)) = iter.next() {
        let loc = match ctx.get_attrib_location(&program, name) {
            -1 => return Err(format!("attribute '{}' was not defined with program", name)),
            loc => loc as u32,
        };
        let mut remaining = total_stride;
        let mut loc_offset = 0;
        while remaining > 0 {
            let stride = min(remaining, 4);
            remaining -= stride;
            let loc = loc + loc_offset;
            ctx.enable_vertex_attrib_array(loc);
            ctx.vertex_attrib_pointer_with_i32(
                loc,
                stride,
                WebGl2RenderingContext::FLOAT,
                false,
                byte_length,
                byte_offset,
            );
            if divisor != 0 {
                ctx.vertex_attrib_divisor(loc, divisor);
            }
            byte_offset += stride * 4;
            loc_offset += 1;
        }
    }
    if byte_offset != byte_length {
        return Err(String::from("invalid byte length"));
    }
    Ok(buffer)
}

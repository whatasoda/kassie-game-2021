use super::buffer_data::{buffer_data, ConvertArrayView};
use super::Shader;

use std::cmp::min;
use std::marker::PhantomData;
use std::mem;
use web_sys::WebGlProgram;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

const DIVISOR: u32 = 1;

pub(super) struct ArrayBuffers<V, I, U>
where
    V: Sized,
    I: Sized,
    U: Sized,
{
    _phantom: PhantomData<(V, I, U)>,
    vao: Option<WebGlVertexArrayObject>,
    vertex: Option<WebGlBuffer>,
    instance: Option<WebGlBuffer>,
    uniform: Option<WebGlBuffer>,
}

impl<V, I, U> ArrayBuffers<V, I, U>
where
    V: Sized,
    I: Sized,
{
    pub fn empty() -> Self {
        ArrayBuffers {
            _phantom: PhantomData {},
            vao: None,
            vertex: None,
            instance: None,
            uniform: None,
        }
    }
}

impl<V, I, U> Shader<'_, V, I, U> {
    fn_ensure_option!(
        [fn ensure_vao],
        buffers.vao,
        if_none: "VAO uninitialized",
        if_some: "VAO already exists",
    );
    fn_ensure_option!(
        [fn ensure_vertex],
        buffers.vertex,
        if_none: "vertex layout uninitialized",
        if_some: "vertex layout already exists",
    );
    fn_ensure_option!(
        [fn ensure_instance],
        buffers.instance,
        if_none: "instance layout uninitialized",
        if_some: "instance layout already exists",
    );
    fn_ensure_option!(
        [fn ensure_uniform],
        buffers.uniform,
        if_none: "uniform uninitialized",
        if_some: "uniform already exists",
    );

    pub(super) fn init_buffers(&mut self) -> Result<(), String> {
        self.ensure_vao(None)?;

        self.buffers.vao = Some(
            self.ctx
                .create_vertex_array()
                .ok_or("failed to create vao")?,
        );
        self.buffers.uniform = Some(create_uniform_buffer(self.ctx)?);
        Ok(())
    }

    pub fn prepare_draw(&self) -> Result<(), String> {
        self.ensure_vao(Some(()))?;

        self.ctx.bind_buffer_base(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            0,
            self.buffers.uniform.as_ref(),
        );
        self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
        Ok(())
    }

    pub unsafe fn vertex_buffer_data(&self, data: &Vec<V>) -> Result<(), String> {
        self.ensure_vao(Some(()))?;
        self.ensure_vertex(Some(()))?;

        self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
        buffer_data(
            &self.ctx,
            WebGl2RenderingContext::ARRAY_BUFFER,
            self.buffers.vertex.as_ref(),
            data,
            false,
        );
        self.ctx.bind_vertex_array(None);
        Ok(())
    }

    pub unsafe fn instance_buffer_data(&self, data: &Vec<I>) -> Result<(), String> {
        self.ensure_vao(Some(()))?;
        self.ensure_instance(Some(()))?;

        self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
        buffer_data(
            &self.ctx,
            WebGl2RenderingContext::ARRAY_BUFFER,
            self.buffers.instance.as_ref(),
            data,
            true,
        );
        self.ctx.bind_vertex_array(None);
        Ok(())
    }

    pub unsafe fn uniform_buffer_data(&self, data: &U) -> Result<(), String>
    where
        U: ConvertArrayView,
    {
        buffer_data(
            &self.ctx,
            WebGl2RenderingContext::DYNAMIC_DRAW,
            self.buffers.uniform.as_ref(),
            data,
            true,
        );
        Ok(())
    }

    pub fn set_vertex_layout(&mut self, layout: Vec<(&'static str, i32)>) -> Result<(), String> {
        self.ensure_vao(Some(()))?;
        self.ensure_vertex(None)?;
        self.ensure_program(Some(()))?;

        self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
        self.buffers.vertex = Some(create_buffer_with_layout::<V>(
            self.ctx,
            &self.program.program.as_ref().unwrap(),
            layout,
            false,
        )?);
        self.ctx.bind_vertex_array(None);
        Ok(())
    }

    pub fn set_instance_layout(&mut self, layout: Vec<(&'static str, i32)>) -> Result<(), String> {
        self.ensure_vao(Some(()))?;
        self.ensure_instance(None)?;
        self.ensure_program(Some(()))?;

        self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
        self.buffers.instance = Some(create_buffer_with_layout::<I>(
            self.ctx,
            &self.program.program.as_ref().unwrap(),
            layout,
            true,
        )?);
        self.ctx.bind_vertex_array(None);
        Ok(())
    }
}

fn create_buffer_with_layout<T>(
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
    layout: Vec<(&str, i32)>,
    is_instanced: bool,
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
            if is_instanced {
                ctx.vertex_attrib_divisor(loc, DIVISOR);
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

fn create_uniform_buffer(ctx: &WebGl2RenderingContext) -> Result<WebGlBuffer, String> {
    let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
    Ok(buffer)
}

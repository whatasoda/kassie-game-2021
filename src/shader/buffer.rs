use super::Shader;
use js_sys::{Float32Array, WebAssembly};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

const DIVISOR: u32 = 1;

pub(super) struct Buffers<V, I, U>
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

impl<V, I, U> Shader<'_, V, I, U> {
    pub fn init_buffers(&mut self) -> Result<(), String> {
        match self.buffers.vao {
            Some(_) => Err(String::from("VAO already exists")),
            None => {
                self.buffers.vao = Some(
                    self.ctx
                        .create_vertex_array()
                        .ok_or("failed to create vao")?,
                );
                self.buffers.uniform =
                    Some(create_uniform_buffer(self.ctx, self.buffers.vao.as_ref())?);
                Ok(())
            }
        }
    }

    pub fn prepare_draw(&self) -> Result<(), String> {
        match self.buffers.vao {
            Some(_) => {
                self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
                self.ctx.bind_buffer_base(
                    WebGl2RenderingContext::UNIFORM_BUFFER,
                    0,
                    self.buffers.uniform.as_ref(),
                );
            }
            None => return Err(String::from("VAO uninitialized")),
        }
        Ok(())
    }

    pub unsafe fn vertex_buffer_data(&self, data: &Vec<V>) -> Result<(), String> {
        if self.buffers.vao.is_none() {
            return Err(String::from("VAO uninitialized"));
        }
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

    pub unsafe fn instance_buffer_data(&self, data: &Vec<V>) -> Result<(), String> {
        if self.buffers.vao.is_none() {
            return Err(String::from("VAO uninitialized"));
        }
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
        if self.buffers.vao.is_none() {
            return Err(String::from("VAO uninitialized"));
        }
        self.ctx.bind_vertex_array(self.buffers.vao.as_ref());
        buffer_data(
            &self.ctx,
            WebGl2RenderingContext::UNIFORM_BUFFER,
            self.buffers.uniform.as_ref(),
            data,
            true,
        );
        self.ctx.bind_vertex_array(None);
        Ok(())
    }

    pub fn set_vertex_layout(&mut self, layout: Vec<(&'static str, i32)>) -> Result<(), String> {
        if self.buffers.vao.is_none() {
            return Err(String::from("VAO uninitialized"));
        }
        if self.buffers.vertex.is_some() {
            return Err(String::from("Layout already exists"));
        }
        self.buffers.vertex = Some(create_buffer_with_layout::<V>(
            self.ctx,
            self.buffers.vao.as_ref(),
            &self.program.attrib_locations,
            layout,
            false,
        )?);
        Ok(())
    }

    pub fn set_instance_layout(&mut self, layout: Vec<(&'static str, i32)>) -> Result<(), String> {
        if self.buffers.vao.is_none() {
            return Err(String::from("VAO uninitialized"));
        }
        if self.buffers.instance.is_some() {
            return Err(String::from("Layout already exists"));
        }
        self.buffers.instance = Some(create_buffer_with_layout::<I>(
            self.ctx,
            self.buffers.vao.as_ref(),
            &self.program.attrib_locations,
            layout,
            true,
        )?);
        Ok(())
    }
}

impl<V, I, U> Buffers<V, I, U>
where
    V: Sized,
    I: Sized,
{
    pub fn empty() -> Self {
        Self {
            _phantom: PhantomData {},
            vao: None,
            vertex: None,
            instance: None,
            uniform: None,
        }
    }
}

fn create_buffer_with_layout<T>(
    ctx: &WebGl2RenderingContext,
    vao: Option<&WebGlVertexArrayObject>,
    locations: &HashMap<&str, u32>,
    layout: Vec<(&str, i32)>,
    is_instanced: bool,
) -> Result<WebGlBuffer, String>
where
    T: Sized,
{
    let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    let mut offset = 0;
    let byte_length = mem::size_of::<T>() as i32;
    let mut iter = layout.into_iter();
    ctx.bind_vertex_array(vao);
    while let Some((name, stride)) = iter.next() {
        let loc = locations
            .get(name)
            .ok_or_else(|| format!("'{}' was not found from attribute locations", name))?;
        ctx.enable_vertex_attrib_array(*loc);
        ctx.vertex_attrib_pointer_with_i32(
            *loc,
            stride,
            WebGl2RenderingContext::FLOAT,
            false,
            byte_length,
            offset,
        );
        if is_instanced {
            ctx.vertex_attrib_divisor(*loc, DIVISOR);
        }
        offset += stride << 2;
    }
    if offset != byte_length {
        return Err(String::from("invalid byte length"));
    }
    ctx.bind_vertex_array(None);
    Ok(buffer)
}

fn create_uniform_buffer(
    ctx: &WebGl2RenderingContext,
    vao: Option<&WebGlVertexArrayObject>,
) -> Result<WebGlBuffer, String> {
    let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_vertex_array(vao);
    ctx.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
    ctx.bind_vertex_array(None);
    Ok(buffer)
}

unsafe fn buffer_data<T>(
    ctx: &WebGl2RenderingContext,
    buffer_type: u32,
    buffer: Option<&WebGlBuffer>,
    vertices: &T,
    is_dynamic: bool,
) where
    T: ConvertArrayView,
{
    ctx.bind_buffer(buffer_type, buffer);
    ctx.buffer_data_with_array_buffer_view(
        buffer_type,
        &create_array_view(vertices),
        if is_dynamic {
            WebGl2RenderingContext::DYNAMIC_DRAW
        } else {
            WebGl2RenderingContext::STATIC_DRAW
        },
    );
}

// Note that `Float32Array::view` is somewhat dangerous (hence the
// `unsafe`!). This is creating a raw view into our module's
// `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
// (aka do a memory allocation in Rust) it'll cause the buffer to change,
// causing the `Float32Array` to be invalid.
//
// As a result, after `Float32Array::view` we have to be very careful not to
// do any memory allocations before it's dropped.
unsafe fn create_array_view<T>(vertices: &T) -> Float32Array
where
    T: ConvertArrayView,
{
    let buf = wasm_bindgen::memory();
    let mem = buf.unchecked_ref::<WebAssembly::Memory>();
    Float32Array::new_with_byte_offset_and_length(
        &mem.buffer(),
        vertices.byte_offset() as u32,
        vertices.byte_length(4) as u32,
    )
}

pub trait ConvertArrayView
where
    Self: Sized,
{
    fn byte_offset(&self) -> usize {
        self as *const Self as usize
    }
    fn byte_length(&self, unit: usize) -> usize {
        mem::size_of::<Self>() / unit
    }
}

impl<T> ConvertArrayView for Vec<T>
where
    T: Sized,
{
    fn byte_offset(&self) -> usize {
        self.as_ptr() as usize
    }
    fn byte_length(&self, unit: usize) -> usize {
        (self.len() * mem::size_of::<T>()) / unit
    }
}

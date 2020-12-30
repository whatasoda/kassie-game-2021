use super::ext::AngleInstancedArrays;
use super::Shader;
use js_sys::{Float32Array, WebAssembly};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

const DIVISOR: u32 = 1;

pub struct Buffers<V, I>
where
    V: Sized,
    I: Sized,
{
    _phantom: PhantomData<(V, I)>,
    vertex: Option<WebGlBuffer>,
    instance: Option<WebGlBuffer>,
}

impl<V, I> Shader<'_, V, I> {
    pub unsafe fn vertex_buffer_data(&self, data: &Vec<V>) {
        buffer_data(&self.ctx, self.buffers.vertex.as_ref(), data, false);
    }
    pub unsafe fn instance_buffer_data(&self, data: &Vec<V>) {
        buffer_data(&self.ctx, self.buffers.instance.as_ref(), data, true);
    }

    pub fn set_vertex_layout(
        &mut self,
        ext_angle: &AngleInstancedArrays,
        layout: Vec<(&'static str, i32)>,
    ) -> Result<(), String> {
        self.buffers.vertex = Some(create_buffer_with_layout::<V>(
            &self.ctx,
            &ext_angle,
            &self.program.attrib_locations,
            layout,
            false,
        )?);
        Ok(())
    }
    pub fn set_instance_layout(
        &mut self,
        ext_angle: &AngleInstancedArrays,
        layout: Vec<(&'static str, i32)>,
    ) -> Result<(), String> {
        self.buffers.instance = Some(create_buffer_with_layout::<I>(
            self.ctx,
            &ext_angle,
            &self.program.attrib_locations,
            layout,
            true,
        )?);
        Ok(())
    }
}

impl<V, I> Buffers<V, I>
where
    V: Sized,
    I: Sized,
{
    pub fn empty() -> Self {
        Self {
            _phantom: PhantomData {},
            vertex: None,
            instance: None,
        }
    }
}

fn create_buffer_with_layout<T>(
    ctx: &WebGlRenderingContext,
    ext_angle: &AngleInstancedArrays,
    locations: &HashMap<&str, u32>,
    layout: Vec<(&str, i32)>,
    is_instanced: bool,
) -> Result<WebGlBuffer, String>
where
    T: Sized,
{
    let buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    let mut offset = 0;
    let byte_length = mem::size_of::<T>() as i32;
    let mut iter = layout.into_iter();
    while let Some((name, stride)) = iter.next() {
        let loc = locations
            .get(name)
            .ok_or_else(|| format!("'{}' was not found with attribute locations", name))?;
        ctx.enable_vertex_attrib_array(*loc);
        ctx.vertex_attrib_pointer_with_i32(
            *loc,
            stride,
            WebGlRenderingContext::FLOAT,
            false,
            byte_length,
            offset,
        );
        if is_instanced {
            ext_angle.vertex_attrib_divisor_angle(*loc, DIVISOR);
        }
        offset += stride << 2;
    }
    if offset != byte_length {
        return Err(String::from("invalid byte length"));
    }
    Ok(buffer)
}

unsafe fn buffer_data<T>(
    ctx: &WebGlRenderingContext,
    buffer: Option<&WebGlBuffer>,
    vertices: &Vec<T>,
    is_dynamic: bool,
) {
    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, buffer);
    ctx.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &create_array_view(vertices),
        if is_dynamic {
            WebGlRenderingContext::DYNAMIC_DRAW
        } else {
            WebGlRenderingContext::STATIC_DRAW
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
unsafe fn create_array_view<V>(vertices: &Vec<V>) -> Float32Array
where
    V: Sized,
{
    let buf = wasm_bindgen::memory();
    let mem = buf.unchecked_ref::<WebAssembly::Memory>();
    Float32Array::new_with_byte_offset_and_length(
        &mem.buffer(),
        vertices.as_ptr() as u32,
        (vertices.len() as u32 * mem::size_of::<V>() as u32) >> 2,
    )
}

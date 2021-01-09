use js_sys::{Float32Array, WebAssembly};
use std::mem;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

pub(super) unsafe fn buffer_data<T>(
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

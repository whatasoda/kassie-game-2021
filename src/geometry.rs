use js_sys::{Float32Array, WebAssembly};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation,
};
use webgl_matrix::{Mat4, Vec4};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

// // Multiple arguments too!
// #[wasm_bindgen(js_namespace = console, js_name = log)]
// fn log_many(a: &str, b: &str);
}

#[repr(C)]
struct Vartex {
    position: Vec4,
    u: f32,
    v: f32,
}

struct Instance {
    texture: String,
    model_matrix: Mat4,
}

pub struct ShaderProgram<V, I>
where
    V: Sized,
    I: Sized,
{
    pub program: WebGlProgram,
    vert: WebGlShader,
    frag: WebGlShader,
    attrib_locations: HashMap<&'static str, u32>,
    uniform_locations: HashMap<&'static str, WebGlUniformLocation>,
    vartex_buffer: WebGlBuffer,
    instance_buffer: WebGlBuffer,
    vartex: PhantomData<V>,
    instance: PhantomData<I>,
}

impl<V, I> ShaderProgram<V, I>
where
    V: Sized,
    I: Sized,
{
    pub fn new(
        ctx: &WebGlRenderingContext,
        attributes: Vec<&'static str>,
        uniforms: Vec<&'static str>,
        vertex_layout: Vec<(&'static str, i32)>,
        instance_layout: Vec<(&'static str, i32)>,
        vert: &str,
        frag: &str,
    ) -> Result<Self, String> {
        let angle = ctx
            .get_extension("ANGLE_instanced_arrays")
            .map_err(|_| String::from("failed to get extension"))?
            .ok_or_else(|| String::from("extension unsupported"))?
            .unchecked_into::<AngleInstancedArrays>();

        let vert = compile_shader(ctx, WebGlRenderingContext::VERTEX_SHADER, vert)?;
        let frag = compile_shader(ctx, WebGlRenderingContext::FRAGMENT_SHADER, frag)?;
        let program = link_program(&ctx, &vert, &frag)?;
        ctx.use_program(Some(&program));

        let attrib_locations = collect_attrib_locations(&ctx, &program, &attributes)?;
        let uniform_locations = collect_uniform_locations(&ctx, &program, &uniforms)?;
        let vartex_buffer =
            create_buffer_with_layout::<V>(ctx, &angle, &attrib_locations, vertex_layout, 0)?;
        let instance_buffer =
            create_buffer_with_layout::<I>(ctx, &angle, &attrib_locations, instance_layout, 1)?;

        Ok(ShaderProgram {
            vert,
            frag,
            program,
            vartex_buffer,
            instance_buffer,
            attrib_locations,
            uniform_locations,
            vartex: PhantomData {},
            instance: PhantomData {},
        })
    }

    pub unsafe fn vertex_buffer_data(&self, ctx: &WebGlRenderingContext, vertices: &Vec<V>) {
        ctx.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.vartex_buffer),
        );
        ctx.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &create_array_view(vertices),
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
}

fn compile_shader(
    ctx: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = ctx
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    ctx.shader_source(&shader, source);
    ctx.compile_shader(&shader);

    if ctx
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(ctx
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    ctx: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = ctx
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    ctx.attach_shader(&program, vert_shader);
    ctx.attach_shader(&program, frag_shader);
    ctx.link_program(&program);

    if ctx
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(ctx
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn collect_attrib_locations<'a>(
    ctx: &WebGlRenderingContext,
    program: &WebGlProgram,
    attirbutes: &Vec<&'a str>,
) -> Result<HashMap<&'a str, u32>, String> {
    let mut locations = HashMap::<&'a str, u32>::new();
    let mut iter = attirbutes.iter();
    while let Some(name) = iter.next() {
        let loc = ctx.get_attrib_location(&program, name);
        if loc == -1 {
            return Err(format!("attribute '{}' was not defined with program", name));
        }
        locations.insert(name, loc as u32);
    }
    Ok(locations)
}

fn collect_uniform_locations<'a>(
    ctx: &WebGlRenderingContext,
    program: &WebGlProgram,
    uniforms: &Vec<&'a str>,
) -> Result<HashMap<&'a str, WebGlUniformLocation>, String> {
    let mut locations = HashMap::<&'a str, WebGlUniformLocation>::new();
    let mut iter = uniforms.iter();
    while let Some(name) = iter.next() {
        let loc = ctx
            .get_uniform_location(&program, name)
            .ok_or_else(|| format!("attribute '{}' was not defined with program", name))?;
        locations.insert(name, loc);
    }
    Ok(locations)
}

fn create_buffer_with_layout<T>(
    ctx: &WebGlRenderingContext,
    ext: &AngleInstancedArrays,
    locations: &HashMap<&str, u32>,
    layout: Vec<(&str, i32)>,
    divisor: u32,
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
        if divisor != 0 {
            ext.vertex_attrib_divisor_angle(*loc, divisor);
        }
        offset += stride << 2;
    }
    if offset != byte_length {
        return Err(String::from("invalid byte length"));
    }
    Ok(buffer)
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

// https://github.com/rustwasm/wasm-bindgen/issues/1428#issuecomment-480671336
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = ANGLEInstancedArrays)]
    type AngleInstancedArrays;

    #[wasm_bindgen(method, getter, js_name = VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE)]
    fn vertex_attrib_array_divisor_angle(this: &AngleInstancedArrays) -> i32;

    #[wasm_bindgen(method, catch, js_name = drawArraysInstancedANGLE)]
    fn draw_arrays_instanced_angle(
        this: &AngleInstancedArrays,
        mode: u32,
        first: i32,
        count: i32,
        primcount: i32,
    ) -> Result<(), JsValue>;

    // TODO offset should be i64
    #[wasm_bindgen(method, catch, js_name = drawElementsInstancedANGLE)]
    fn draw_elements_instanced_angle(
        this: &AngleInstancedArrays,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i32,
        primcount: i32,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_name = vertexAttribDivisorANGLE)]
    fn vertex_attrib_divisor_angle(this: &AngleInstancedArrays, index: u32, divisor: u32);
}

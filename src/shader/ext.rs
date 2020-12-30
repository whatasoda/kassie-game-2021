use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;

pub trait ExtensionGetter
where
    Self: JsCast,
{
    const EXT_NAME: &'static str;
    fn get_optional(ctx: &WebGlRenderingContext) -> Result<Option<Self>, String> {
        Ok(ctx
            .get_extension(Self::EXT_NAME)
            .map_err(|_| format!("failed to get extension '{}'", Self::EXT_NAME))?
            .and_then(|ext| Some(ext.unchecked_into::<Self>())))
    }
    fn get(ctx: &WebGlRenderingContext) -> Result<Self, String> {
        Ok(Self::get_optional(ctx)?
            .ok_or_else(|| format!("extension '{}' unsupported ", Self::EXT_NAME))?
            .unchecked_into::<Self>())
    }
}

impl ExtensionGetter for AngleInstancedArrays {
    const EXT_NAME: &'static str = "ANGLE_instanced_arrays";
}
// https://github.com/rustwasm/wasm-bindgen/issues/1428#issuecomment-480671336
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = ANGLEInstancedArrays)]
    pub type AngleInstancedArrays;

    #[wasm_bindgen(method, getter, js_name = VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE)]
    pub fn vertex_attrib_array_divisor_angle(this: &AngleInstancedArrays) -> i32;

    #[wasm_bindgen(method, catch, js_name = drawArraysInstancedANGLE)]
    pub fn draw_arrays_instanced_angle(
        this: &AngleInstancedArrays,
        mode: u32,
        first: i32,
        count: i32,
        primcount: i32,
    ) -> Result<(), JsValue>;

    // TODO offset should be i64
    #[wasm_bindgen(method, catch, js_name = drawElementsInstancedANGLE)]
    pub fn draw_elements_instanced_angle(
        this: &AngleInstancedArrays,
        mode: u32,
        count: i32,
        type_: u32,
        offset: i32,
        primcount: i32,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_name = vertexAttribDivisorANGLE)]
    pub fn vertex_attrib_divisor_angle(this: &AngleInstancedArrays, index: u32, divisor: u32);
}

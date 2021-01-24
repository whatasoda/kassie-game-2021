use super::ShaderController;
use web_sys::WebGlProgram;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlImageElement, WebGl2RenderingContext, WebGlTexture, WebGlUniformLocation};

pub struct Textures {
    uniforms: HashMap<u32, WebGlUniformLocation>,
    cache_tex: HashMap<&'static str, WebGlTexture>,
    cache_img: HashMap<&'static str, HtmlImageElement>,
}

impl Textures {
    pub fn empty() -> Self {
        Textures {
            uniforms: HashMap::new(),
            cache_tex: HashMap::new(),
            cache_img: HashMap::new(),
        }
    }
}

impl ShaderController {
    pub async fn create_texture<'a>(&'a mut self, src: &'static str) -> Result<(), JsValue> {
        if self.textures.cache_tex.contains_key(src) {
            return Ok(());
        }
        let shared = self.shared.borrow();
        if !self.textures.cache_img.contains_key(src) {
            self.textures
                .cache_img
                .insert(src, load_image(&shared.doc, src).await?);
        }
        let img = self.textures.cache_img.get(src);
        let img = img.as_ref().unwrap();
        let tex = shared
            .ctx
            .create_texture()
            .ok_or("failed to create texture")?;

        shared
            .ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tex));
        self.textures.cache_tex.insert(src, tex);
        shared
            .ctx
            .tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                img,
            )?;
        shared
            .ctx
            .generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        Ok(())
    }

    pub fn bind_texture(&self, tex_slot: u32, src: &str) -> Result<(), JsValue> {
        let tex = self.textures.cache_tex.get(src).ok_or("unknown texture")?;
        let shared = self.shared.borrow();
        if tex_slot > 31 {
            Err("texture slot out of range")?;
        }

        shared
            .ctx
            .active_texture(WebGl2RenderingContext::TEXTURE0 + tex_slot);
        shared
            .ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tex));

        Ok(())
    }

    pub fn attach_texture(&mut self, tex_id: u32, tex_slot: u32) -> Result<(), JsValue> {
        self.ensure_program(Some(()))?;
        if tex_slot > 31 {
            Err("texture slot out of range")?;
        }
        let shared = self.shared.borrow();

        let loc = get_tex_uniform_location(
            &shared.ctx,
            self.program.program.as_ref().unwrap(),
            &mut self.textures.uniforms,
            tex_id,
        )?;
        shared.ctx.uniform1i(Some(loc), tex_slot as i32);

        Ok(())
    }
}

const TEX_NAMES: [&'static str; 32] = [
    "tex0", "tex1", "tex2", "tex3", "tex4", "tex5", "tex6", "tex7", "tex8", "tex9", //
    "tex10", "tex11", "tex12", "tex13", "tex14", "tex15", "tex16", "tex17", "tex18", "tex19",
    "tex20", "tex21", "tex22", "tex23", "tex24", "tex25", "tex26", "tex27", "tex28", "tex29",
    "tex30", "tex31",
];
fn get_tex_uniform_location<'a>(
    ctx: &WebGl2RenderingContext,
    program: &WebGlProgram,
    acc: &'a mut HashMap<u32, WebGlUniformLocation>,
    tex_id: u32,
) -> Result<&'a WebGlUniformLocation, JsValue> {
    if !acc.contains_key(&tex_id) {
        let name = TEX_NAMES
            .get(tex_id as usize)
            .ok_or("texture id out of range")?;
        let loc = ctx
            .get_uniform_location(program, name)
            .ok_or(format!("failed to get uniform location tex{}", tex_id))?;
        acc.insert(tex_id, loc);
    }
    Ok(acc.get(&tex_id).unwrap())
}

async fn load_image(
    document: &web_sys::Document,
    src: &str,
) -> Result<web_sys::HtmlImageElement, JsValue> {
    let img = document
        .create_element("img")
        .unwrap()
        .dyn_into::<web_sys::HtmlImageElement>()?;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let onload = Closure::wrap(Box::new(move || {
            resolve.call0(&JsValue::NULL)?;
            Ok(())
        }) as Box<dyn FnMut() -> Result<(), JsValue>>);
        img.set_onload(Some(onload.as_ref().unchecked_ref()));
        img.set_src(src);
        onload.forget();
    });
    JsFuture::from(promise).await?;
    Ok(img)
}

use super::Shader;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, HtmlImageElement, WebGl2RenderingContext, WebGlTexture};

pub struct Textures {
    cache_tex: HashMap<&'static str, WebGlTexture>,
    cache_img: HashMap<&'static str, HtmlImageElement>,
}

impl Textures {
    pub fn empty() -> Self {
        Textures {
            cache_tex: HashMap::new(),
            cache_img: HashMap::new(),
        }
    }
}

impl<V, I, U> Shader<'_, V, I, U> {
    pub async fn create_texture<'a>(
        &'a mut self,
        document: &'a Document,
        src: &'static str,
    ) -> Result<(), JsValue> {
        if self.textures.cache_tex.contains_key(src) {
            return Ok(());
        }
        if !self.textures.cache_img.contains_key(src) {
            self.textures
                .cache_img
                .insert(src, load_image(&document, src).await?);
        }
        let img = self.textures.cache_img.get(src);
        let img = img.as_ref().unwrap();
        let tex = self
            .ctx
            .create_texture()
            .ok_or("failed to create texture")?;

        self.ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tex));
        self.textures.cache_tex.insert(src, tex);
        self.ctx
            .tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                img,
            )?;
        self.ctx.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        Ok(())
    }

    pub fn bind_texture(&self, tex_slot: u32, src: &str) -> Result<(), JsValue> {
        let tex = self.textures.cache_tex.get(src).ok_or("unknown texture")?;
        if tex_slot > 31 {
            Err("invalid texture slot")?;
        }

        self.ctx
            .active_texture(WebGl2RenderingContext::TEXTURE0 + tex_slot);
        self.ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tex));

        Ok(())
    }
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

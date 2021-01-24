#[macro_use]
mod macros;

mod array_buffer;
mod buffer_data;
mod compile;
mod texture;
mod uniform_buffer;

pub use buffer_data::ConvertArrayView;
use uniform_buffer::UniformBuffers;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{Document, WebGl2RenderingContext};

pub struct SharedContext {
    pub doc: Rc<Document>,
    pub ctx: Rc<WebGl2RenderingContext>,
    uniform_buffers: UniformBuffers,
}

impl SharedContext {
    pub fn new(doc: Rc<Document>, ctx: Rc<WebGl2RenderingContext>) -> Rc<RefCell<SharedContext>> {
        Rc::new(RefCell::new(SharedContext {
            doc,
            ctx,
            uniform_buffers: UniformBuffers::new(),
        }))
    }
}

pub struct ShaderController {
    pub shared: Rc<RefCell<SharedContext>>,
    program: compile::Program,
    arrays: array_buffer::ArrayBuffers,
    uniforms: uniform_buffer::UniformBlocks,
    textures: texture::Textures,
}

impl ShaderController {
    pub fn new(shared: Rc<RefCell<SharedContext>>) -> Self {
        Self {
            shared,
            program: compile::Program::empty(),
            arrays: array_buffer::ArrayBuffers::empty(),
            uniforms: uniform_buffer::UniformBlocks::empty(),
            textures: texture::Textures::empty(),
        }
    }
}

pub trait ShaderImpl {
    const INSTANCE_CAPACITY: Option<usize>;
    fn new() -> Self;
    fn init(&self, shader: &mut ShaderController) -> Result<(), JsValue>;
    fn get_texture_map(&self) -> Vec<(u32, u32, &'static str)>;
    fn draw(&self, ctx: &WebGl2RenderingContext, time: f32, instance_len: i32);
}

pub type Shader<T, I> = ShaderWrapper<T, I>;

pub struct ShaderWrapper<T, I>
where
    T: ShaderImpl,
    I: Sized,
{
    implementation: T,
    pub controller: ShaderController,
    pub instances: Vec<I>,
}

impl<T, I> ShaderWrapper<T, I>
where
    T: ShaderImpl,
    I: Sized,
{
    pub fn new(shared: Rc<RefCell<SharedContext>>) -> Result<Rc<RefCell<Shader<T, I>>>, JsValue> {
        let mut controller = ShaderController::new(shared);
        let implementation = T::new();
        implementation.init(&mut controller)?;
        Ok(Rc::new(RefCell::new(ShaderWrapper {
            implementation,
            controller,
            instances: match T::INSTANCE_CAPACITY {
                Some(capacity) => Vec::with_capacity(capacity),
                None => Vec::new(),
            },
        })))
    }

    pub async fn init_textures(&mut self) -> Result<(), JsValue> {
        self.controller.activate();
        for (_, _, filename) in self.implementation.get_texture_map() {
            self.controller.create_texture(filename).await?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn draw(&mut self, time: f32) -> Result<(), JsValue> {
        self.controller.activate();
        for (tex_id, tex_slot, filename) in self.implementation.get_texture_map() {
            self.controller.bind_texture(tex_slot, filename)?;
            self.controller.attach_texture(tex_id, tex_slot)?;
        }
        unsafe {
            self.controller
                .buffer_data_dynamic("instance", &self.instances)?;
        }
        self.controller.prepare_array_buffers()?;
        self.controller.preapre_uniform_blocks()?;
        self.implementation.draw(
            &self.controller.shared.borrow().ctx,
            time,
            self.instances.len() as i32,
        );
        Ok(())
    }
}

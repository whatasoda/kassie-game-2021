use crate::now;

use core::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub trait InputReceiver {
    fn onclick(&mut self, time: f32);
}

pub fn set_input_handler<T>(canvas: &web_sys::HtmlCanvasElement, receiver: Rc<RefCell<T>>)
where
    T: InputReceiver + 'static,
{
    let onclick = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        event;
        receiver.as_ref().borrow_mut().onclick(now());
        Ok(())
    })
        as Box<dyn FnMut(web_sys::MouseEvent) -> Result<(), JsValue>>);

    canvas.set_onclick(Some(onclick.as_ref().unchecked_ref()));
    onclick.forget();
}

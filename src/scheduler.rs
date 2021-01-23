use crate::now;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &JsValue);
}

fn request_animation_frame(window: &Window, f: &Closure<dyn FnMut()>) {
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn start_loop<T>(w: Rc<Window>, mut task: T) -> Result<(), JsValue>
where
    T: 'static + FnMut(f32) -> Result<(), JsValue>,
{
    let x = w.clone();
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if true {
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(w.as_ref(), f.borrow().as_ref().unwrap());
        } else {
            // Drop our handle to this closure so that it will get cleaned up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        if let Err(err) = task(now()) {
            error(&err);
        }
    }) as Box<dyn FnMut()>));

    request_animation_frame(x.as_ref(), g.borrow().as_ref().unwrap());
    Ok(())
}

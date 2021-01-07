use num_traits::cast::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &JsValue);
}

fn get_window() -> web_sys::Window {
    web_sys::window().expect("should have a window in this context")
}

fn request_animation_frame(window: &web_sys::Window, f: &Closure<dyn FnMut()>) {
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn start_loop<T>(mut task: T) -> Result<(), JsValue>
where
    T: 'static + FnMut(f32) -> Result<(), JsValue>,
{
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let window = get_window();
    let performance = window
        .performance()
        .expect("performance should be available");
    let time_origin = performance.now();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if true {
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(&window, f.borrow().as_ref().unwrap());
        } else {
            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        let now = performance.now() - time_origin;
        if let Err(err) = task(now.to_f32().unwrap()) {
            error(&err);
        }
    }) as Box<dyn FnMut()>));

    request_animation_frame(&get_window(), g.borrow().as_ref().unwrap());
    Ok(())
}

use crate::now;

use core::cell::RefCell;
use num_traits::ToPrimitive;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

type Element = HtmlCanvasElement;

pub struct ClickEvent {
    pub timestamp: f32,
    coord: (f32, f32),
}

pub struct InputState {
    pub clicked: Option<ClickEvent>,
    pub curr_coord: (f32, f32),
    prev_coord: (f32, f32),
}

impl InputState {
    pub fn resolve(&mut self) {
        self.clicked = None;
        self.prev_coord = self.curr_coord;
    }
}

pub fn set_input_handler(element: Rc<Element>) -> Rc<RefCell<InputState>> {
    let state = Rc::new(RefCell::new(InputState {
        clicked: None,
        curr_coord: (0., 0.),
        prev_coord: (0., 0.),
    }));

    let onclick = Closure::wrap(Box::new({
        let state = state.clone();
        let element = element.clone();
        move |event: web_sys::MouseEvent| {
            let mut state = state.borrow_mut();
            state.clicked = Some(ClickEvent {
                timestamp: now(),
                coord: get_mouse_coord(&event, &element),
            });
            Ok(())
        }
    })
        as Box<dyn FnMut(web_sys::MouseEvent) -> Result<(), JsValue>>);
    element.set_onclick(Some(onclick.as_ref().unchecked_ref()));
    onclick.forget();

    let onmousemove = Closure::wrap(Box::new({
        let state = state.clone();
        let element = element.clone();
        move |event: web_sys::MouseEvent| {
            let mut state = state.borrow_mut();
            state.curr_coord = get_mouse_coord(&event, &element);
            Ok(())
        }
    })
        as Box<dyn FnMut(web_sys::MouseEvent) -> Result<(), JsValue>>);
    element.set_onmousemove(Some(onmousemove.as_ref().unchecked_ref()));
    onmousemove.forget();

    state
}

fn get_mouse_coord(event: &web_sys::MouseEvent, element: &Element) -> (f32, f32) {
    let offset_x = event.offset_x().to_f32().unwrap();
    let offset_y = event.offset_y().to_f32().unwrap();
    let width = element.width().to_f32().unwrap();
    let height = element.height().to_f32().unwrap();
    let x = (2. * offset_x / width) - 1.;
    let y = 1. - (2. * offset_y / height);
    (x, y)
}

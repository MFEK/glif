// Select
use super::prelude::*;

pub fn mouse_moved<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
) -> bool {
    let mposition = update_mousepos(position, &v, false);
    v.borrow_mut().corner_two = Some(mposition);
    v.borrow().show_sel_box
}

pub fn mouse_button<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
    button: MouseButton,
) -> bool {
    false
}

pub fn mouse_pressed<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
    button: MouseButton,
) -> bool {
    v.borrow_mut().show_sel_box = true;
    let position = v.borrow().mousepos;
    let mposition = PhysicalPosition::from((position.x, position.y));
    v.borrow_mut().mousepos = mposition;
    if v.borrow().show_sel_box {
        v.borrow_mut().corner_one = Some(mposition);
    }
    false
}

pub fn mouse_released<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
    button: MouseButton,
) -> bool {
    v.borrow_mut().show_sel_box = false;
    true
}

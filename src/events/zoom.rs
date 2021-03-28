// Zoom
use super::prelude::*;

use log::debug;

pub fn zoom_in_factor<T>(_factor: f32, v: &RefCell<state::State<T>>) -> f32 {
    v.borrow().factor + SCALE_FACTOR
}

pub fn zoom_out_factor<T>(_factor: f32, v: &RefCell<state::State<T>>) -> f32 {
    let mut scale = v.borrow().factor;
    if scale >= 0.10 {
        scale += -SCALE_FACTOR;
    }
    scale
}

pub fn mouse_moved<T>(position: (f64, f64), v: &RefCell<state::State<T>>) -> bool {
    let _mposition = update_mousepos(position, &v, false);
    false
}

use sdl2::mouse::MouseButton;
pub fn mouse_released<T>(
    _position: (f64, f64),
    v: &RefCell<state::State<T>>,
    meta: MouseMeta,
) -> bool {
    let mut scale = v.borrow().factor;
    match meta.button {
        MouseButton::Left => {
            scale = zoom_in_factor(scale, &v);
        }
        MouseButton::Right => {
            scale = zoom_out_factor(scale, &v);
        }
        _ => {}
    }
    let mut offset = v.borrow().offset;
    let winsize = v.borrow().winsize;
    let position = v.borrow().absolute_mousepos;
    let center = (
        (winsize.0 as f32 / 2.) + offset.0,
        (winsize.1 as f32 / 2.) + offset.1,
    );
    offset.0 = -(position.0 as f32 - center.0);
    offset.1 = -(position.1 as f32 - center.1);
    update_viewport(Some(offset), Some(scale), &v);
    debug!(
        "Zoom triggered @ {}x{}, offset {:?}",
        position.0, position.1, offset
    );
    true
}

use crate::renderer::constants::*;
use crate::state;
use glutin::dpi::PhysicalPosition;
use glutin::event::MouseButton;
use reclutch::skia::{Canvas, Matrix};
use std::cell::RefCell;

pub fn update_viewport(
    offset: Option<(f32, f32)>,
    scale: Option<f32>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
) {
    let offset = match offset {
        None => v.borrow().offset,
        Some(offset) => offset,
    };
    let scale = match scale {
        None => v.borrow().factor,
        Some(scale) => scale,
    };
    let mut matrix = Matrix::new_identity();
    matrix.set_scale_translate((scale, scale), offset);
    canvas.set_matrix(&matrix);
    v.borrow_mut().factor = scale;
    v.borrow_mut().offset = offset;
}

pub fn update_mousepos(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    pan: bool,
) -> PhysicalPosition<f64> {
    let factor = 1. / v.borrow().factor as f64;
    let uoffset = v.borrow().offset;
    let offset = (uoffset.0 as f64, uoffset.1 as f64);

    let absolute_mposition = PhysicalPosition::from(((position.x).floor(), (position.y).floor()));
    let mposition = PhysicalPosition::from((
        ((position.x).floor() - offset.0) * factor,
        ((position.y).floor() - offset.1) * factor,
    ));

    v.borrow_mut().absolute_mousepos = absolute_mposition;
    v.borrow_mut().mousepos = mposition;
    if pan {
        absolute_mposition
    } else {
        mposition
    }
}

pub fn mouse_moved_select(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
) -> bool {
    let mposition = update_mousepos(position, &v, false);
    v.borrow_mut().corner_two = Some(mposition);
    v.borrow().show_sel_box
}

pub fn mouse_moved_move(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
) -> bool {
    let old_mposition = v.borrow().absolute_mousepos.clone();
    let mut offset = v.borrow().offset;
    let mposition = update_mousepos(position, &v, true);
    if !v.borrow().mousedown {
        return false;
    }
    offset.0 += (mposition.x - old_mposition.x).floor() as f32;
    offset.1 += (mposition.y - old_mposition.y).floor() as f32;
    //offset = (mposition.x as f32, mposition.y as f32);
    v.borrow_mut().offset = offset;
    update_viewport(None, None, &v, canvas);
    true
}

pub fn mouse_button_select(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
    button: MouseButton,
) -> bool {
    false
}

pub fn mouse_pressed_select(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
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

pub fn mouse_released_select(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
    button: MouseButton,
) -> bool {
    v.borrow_mut().show_sel_box = false;
    true
}

pub fn zoom_in_factor(factor: f32, v: &RefCell<state::State>) -> f32 {
    v.borrow().factor + SCALE_FACTOR
}

pub fn zoom_out_factor(factor: f32, v: &RefCell<state::State>) -> f32 {
    let mut scale = v.borrow().factor;
    if scale >= 0.10 {
        scale += -SCALE_FACTOR;
    }
    scale
}

pub fn mouse_moved_zoom(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
) -> bool {
    let mposition = update_mousepos(position, &v, false);
    false
}

pub fn mouse_released_zoom(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
    button: MouseButton,
) -> bool {
    let mut scale = v.borrow().factor;
    match button {
        MouseButton::Left => {
            scale = zoom_in_factor(scale, &v);
        }
        MouseButton::Right => {
            scale = zoom_out_factor(scale, &v);
        }
        _ => {}
    }
    update_viewport(None, Some(scale), &v, canvas);
    true
}

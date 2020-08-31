use crate::state;
use glutin::dpi::PhysicalPosition;
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
) -> PhysicalPosition<f64> {
    let factor = 1. / v.borrow().factor as f64;
    let uoffset = v.borrow().offset;
    let offset = (uoffset.0 as f64, uoffset.1 as f64);
    //let mposition = PhysicalPosition::from(((position.x - offset.0) * factor, (position.y - offset.1) * factor));
    let mposition =
        PhysicalPosition::from(((position.x).floor() * factor, (position.y).floor() * factor));
    v.borrow_mut().mousepos = mposition;
    mposition
}

pub fn mouse_moved_select(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
) -> bool {
    let mposition = update_mousepos(position, &v);
    v.borrow_mut().corner_two = Some(mposition);
    v.borrow().show_sel_box
}

pub fn mouse_moved_move(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State>,
    canvas: &mut Canvas,
) -> bool {
    let old_mposition = v.borrow().mousepos.clone();
    let mut offset = v.borrow().offset;
    let mposition = update_mousepos(position, &v);
    if !v.borrow().mousedown {
        return false;
    }
    offset.0 += (mposition.x - old_mposition.x).floor() as f32;
    offset.1 += (mposition.y - old_mposition.y).floor() as f32;
    //offset = (mposition.x as f32, mposition.y as f32);
    v.borrow_mut().offset = offset;
    println!(
        "C: {:?}",
        (mposition.x - old_mposition.x, mposition.y - old_mposition.y)
    );
    println!("O: {:?}", offset);
    update_viewport(None, None, &v, canvas);
    true
}

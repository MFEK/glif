use crate::renderer::constants::*;
use crate::renderer::points::calc::*;
use crate::state;
use crate::{PEN_DATA, STATE};
use state::{Mode, PenData, PointData};

use skulpin::skia_safe::{Canvas, Matrix};
use skulpin::winit;
use skulpin::winit::dpi::{PhysicalPosition, PhysicalSize};
use skulpin::winit::event::MouseButton;
use skulpin::winit::window::Window;

use std::cell::RefCell;
use std::mem;

// Generic events
pub fn center_cursor(winit_window: &Window) -> Result<(), winit::error::ExternalError> {
    let mut center = winit_window.outer_size();
    center.width /= 2;
    center.height /= 2;
    STATE.with(|v| {
        v.borrow_mut().absolute_mousepos = PhysicalPosition::from((center.width, center.height))
    });
    winit_window.set_cursor_position(winit::dpi::PhysicalPosition::new(
        center.width as i32,
        center.height as i32,
    ))
}

pub fn update_viewport<T>(
    offset: Option<(f32, f32)>,
    scale: Option<f32>,
    v: &RefCell<state::State<T>>,
) {
    let offset = match offset {
        None => v.borrow().offset,
        Some(offset) => offset,
    };
    let scale = match scale {
        None => v.borrow().factor,
        Some(scale) => scale,
    };
    v.borrow_mut().factor = scale;
    v.borrow_mut().offset = offset;
}

pub fn update_mousepos<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
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

// Pan

pub fn mouse_moved_move<T>(position: PhysicalPosition<f64>, v: &RefCell<state::State<T>>) -> bool {
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
    update_viewport(None, None, &v);
    true
}

// Select

pub fn mouse_moved_select<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
) -> bool {
    let mposition = update_mousepos(position, &v, false);
    v.borrow_mut().corner_two = Some(mposition);
    v.borrow().show_sel_box
}

pub fn mouse_button_select<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
    button: MouseButton,
) -> bool {
    false
}

pub fn mouse_pressed_select<T>(
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

pub fn mouse_released_select<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
    button: MouseButton,
) -> bool {
    v.borrow_mut().show_sel_box = false;
    true
}

// Zoom

pub fn zoom_in_factor<T>(factor: f32, v: &RefCell<state::State<T>>) -> f32 {
    v.borrow().factor + SCALE_FACTOR
}

pub fn zoom_out_factor<T>(factor: f32, v: &RefCell<state::State<T>>) -> f32 {
    let mut scale = v.borrow().factor;
    if scale >= 0.10 {
        scale += -SCALE_FACTOR;
    }
    scale
}

pub fn mouse_moved_zoom<T>(position: PhysicalPosition<f64>, v: &RefCell<state::State<T>>) -> bool {
    let mposition = update_mousepos(position, &v, false);
    false
}

pub fn mouse_released_zoom<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
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
    let mut offset = v.borrow().offset;
    let winsize = v.borrow().winsize;
    let position = v.borrow().absolute_mousepos;
    let mut center = (
        (winsize.width as f32 / 2.) + offset.0,
        (winsize.height as f32 / 2.) + offset.1,
    );
    offset.0 = -(position.x as f32 - center.0);
    offset.1 = -(position.y as f32 - center.1);
    update_viewport(Some(offset), Some(scale), &v);
    debug!(
        "Zoom triggered @ {}x{}, offset {:?}",
        position.x, position.y, offset
    );
    true
}

// Pen

use glifparser::{self, Contour, Handle, Outline, Point, PointType};

// $e of type RefCell<State<T>>
macro_rules! get_outline_mut {
    ($e:expr) => {
        $e.borrow_mut()
            .glyph
            .as_mut()
            .unwrap()
            .glif
            .outline
            .as_mut()
            .unwrap()
    };
}
macro_rules! get_outline {
    ($e:expr) => {
        $e.borrow()
            .glyph
            .as_ref()
            .unwrap()
            .glif
            .outline
            .as_ref()
            .unwrap()
    };
}

pub fn mode_switched(from: Mode, to: Mode) {
    assert!(from != to);
    PEN_DATA.with(|v| v.borrow_mut().contour = None);
}

pub fn mouse_moved_pen(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<Option<PointData>>>,
) -> bool {
    let mposition = update_mousepos(position, &v, false);

    PEN_DATA.with(|vv| {
        let contour = vv.borrow().contour;
        match contour {
            Some(idx) => {
                if v.borrow().mousedown {
                    let mut last_point = get_outline!(v)[idx].last().unwrap().clone();

                    let pos = (calc_x(mposition.x as f32), calc_y(mposition.y as f32));
                    let offset = (last_point.x - pos.0, last_point.y - pos.1);
                    let handle_b = (last_point.x + offset.0, last_point.y + offset.1);

                    get_outline_mut!(v)[idx].last_mut().unwrap().a =
                        Handle::At(calc_x(mposition.x as f32), calc_y(mposition.y as f32));
                    get_outline_mut!(v)[idx].last_mut().unwrap().b =
                        Handle::At(handle_b.0, handle_b.1);
                } else {
                    /*mem::replace(
                        get_outline_mut!(v)[idx].last_mut().unwrap(),
                        (Point::from_x_y_type((calc_x(mposition.x as f32), calc_y(mposition.y as f32)), PointType::Curve))
                    );*/
                }
            }
            None => {}
        }
    });

    true
}

pub fn mouse_pressed_pen(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<Option<PointData>>>,
    button: MouseButton,
) -> bool {
    let mposition = v.borrow().mousepos;

    PEN_DATA.with(|vv| {
        let contour = vv.borrow().contour;
        match contour {
            Some(idx) => {
                get_outline_mut!(v)[idx].push(Point::from_x_y_type(
                    (calc_x(mposition.x as f32), calc_y(mposition.y as f32)),
                    PointType::Curve,
                ));
            }
            None => {
                let mut new_contour: Contour<Option<PointData>> = Vec::new();
                new_contour.push(Point::from_x_y_type(
                    (calc_x(mposition.x as f32), calc_y(mposition.y as f32)),
                    PointType::Curve,
                ));
                get_outline_mut!(v).push(new_contour);
                vv.borrow_mut().contour = Some(get_outline!(v).len() - 1);
            }
        }
    });
    true
}

pub fn mouse_released_pen(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<Option<PointData>>>,
    button: MouseButton,
) -> bool {
    let mposition = v.borrow().mousepos;

    PEN_DATA.with(|vv| {
        //vv.borrow_mut().contour = None;
        if let Some(idx) = vv.borrow().contour {
            get_outline_mut!(v)[idx].last_mut().map(|point| {
                if point.a != Handle::Colocated {
                    point.ptype = PointType::Curve;
                }
            });
        }
    });

    true
}

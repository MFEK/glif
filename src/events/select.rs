// Select
use super::prelude::*;
use crate::state::Follow;
use glifparser::{Handle, WhichHandle};

/// Get indexes stored by clicked_point_or_handle and move the points they refer to around.
pub fn mouse_moved<T>(position: (f64, f64), v: &RefCell<state::State<T>>) -> bool {
    let mposition = update_mousepos(position, &v, false);
    if !v.borrow().mousedown {
        return false;
    }

    let x = calc_x(mposition.0 as f32);
    let y = calc_y(mposition.1 as f32);
    let follow = TOOL_DATA.with(|p| p.borrow().follow);
    let contour = TOOL_DATA.with(|p| p.borrow().contour);
    let cur_point = TOOL_DATA.with(|p| p.borrow().cur_point);
    let which_handle = TOOL_DATA.with(|p| p.borrow().handle);

    let single_point = match (contour, cur_point, which_handle) {
        // Point itself is being moved.
        (Some(ci), Some(pi), WhichHandle::Neither) => {
            let (cx, cy) = (get_outline!(v)[ci][pi].x, get_outline!(v)[ci][pi].y);
            let (dx, dy) = (cx - x, cy - y);

            get_outline_mut!(v)[ci][pi].x = x;
            get_outline_mut!(v)[ci][pi].y = y;

            match follow {
                // ForceLine makes no sense in this context, but putting it here prevents us from
                // falling back to the No branch.
                Follow::Mirror | Follow::ForceLine => {
                    let a = get_outline!(v)[ci][pi].a;
                    let b = get_outline!(v)[ci][pi].b;
                    match a {
                        Handle::At(hx, hy) => {
                            get_outline_mut!(v)[ci][pi].a = Handle::At(hx - dx, hy - dy)
                        }
                        Handle::Colocated => (),
                    }
                    match b {
                        Handle::At(hx, hy) => {
                            get_outline_mut!(v)[ci][pi].b = Handle::At(hx - dx, hy - dy)
                        }
                        Handle::Colocated => (),
                    }
                }
                _ => (),
            }

            true
        }
        // A control point (A or B) is being moved.
        (Some(ci), Some(pi), wh) => {
            let handle = match wh {
                WhichHandle::A => get_outline!(v)[ci][pi].a,
                WhichHandle::B => get_outline!(v)[ci][pi].b,
                WhichHandle::Neither => unreachable!("Should've been matched by above?!"),
            };

            // Current x, current y
            let (cx, cy) = match handle {
                Handle::At(cx, cy) => (cx, cy),
                _ => panic!("Clicked non-existent handle A! Cidx {} pidx {}", ci, pi),
            };

            // Difference in x, difference in y
            let (dx, dy) = (cx - x, cy - y);

            // If Follow::Mirror (left mouse button), other control point (handle) will do mirror
            // image action of currently selected control point. Perhaps pivoting around central
            // point is better?
            macro_rules! move_mirror {
                ($cur:ident, $mirror:ident) => {
                    get_outline_mut!(v)[ci][pi].$cur = Handle::At(x, y);
                    let h = get_outline!(v)[ci][pi].$mirror;
                    match h {
                        Handle::At(hx, hy) => {
                            if follow == Follow::Mirror {
                                get_outline_mut!(v)[ci][pi].$mirror = Handle::At(hx + dx, hy + dy);
                            } else if follow == Follow::ForceLine {
                                let (px, py) =
                                    (get_outline!(v)[ci][pi].x, get_outline!(v)[ci][pi].y);
                                let (dx, dy) = (px - x, py - y);

                                get_outline_mut!(v)[ci][pi].$mirror = Handle::At(px + dx, py + dy);
                            }
                        }
                        Handle::Colocated => (),
                    }
                };
            }

            #[rustfmt::skip]
            match wh {
                WhichHandle::A => { move_mirror!(a, b); },
                WhichHandle::B => { move_mirror!(b, a); },
                WhichHandle::Neither => unreachable!("Should've been matched by above?!"),
            }

            true
        }
        _ => false,
    };

    if !single_point {
        v.borrow_mut().corner_two = Some(mposition);
    }
    true
}

// Placeholder
pub fn mouse_button<T>(
    _position: (f64, f64),
    _v: &RefCell<state::State<T>>,
    _meta: MouseMeta,
) -> bool {
    false
}

/// Transform mouse click position into indexes into STATE.glyph.glif.outline and the handle if
/// applicable, and store it in TOOL_DATA.
fn clicked_point_or_handle(
    position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
) -> Option<(usize, usize, WhichHandle)> {
    let factor = v.borrow().factor;
    let mposition = update_mousepos(position, &v, true);
    let _contour_idx = 0;
    let _point_idx = 0;

    // How we do this is quite naïve. For each click, we just iterate all points and check the
    // point and both handles. It's just a bunch of floating point comparisons in a compiled
    // language, so I'm not too concerned about it, and even in the TT2020 case doesn't seem to
    // slow anything down.
    for (contour_idx, contour) in get_outline!(v).iter().enumerate() {
        for (point_idx, point) in contour.iter().enumerate() {
            let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
            // Topleft corner of point
            let point_tl = SkPoint::new(
                calc_x(point.x as f32) - (size / 2.),
                calc_y(point.y as f32) - (size / 2.),
            );
            let point_rect = SkRect::from_point_and_size(point_tl, (size, size));
            // Topleft corner of handle a
            let a = point.handle_or_colocated(WhichHandle::A, |f| f, |f| f);
            let a_tl = SkPoint::new(calc_x(a.0) - (size / 2.), calc_y(a.1) - (size / 2.));
            let a_rect = SkRect::from_point_and_size(a_tl, (size, size));
            // Topleft corner of handle b
            let b = point.handle_or_colocated(WhichHandle::B, |f| f, |f| f);
            let b_tl = SkPoint::new(calc_x(b.0) - (size / 2.), calc_y(b.1) - (size / 2.));
            let b_rect = SkRect::from_point_and_size(b_tl, (size, size));

            // winit::PhysicalPosition as an SkPoint
            let sk_mpos = SkPoint::new(mposition.0 as f32, mposition.1 as f32);

            if point_rect.contains(sk_mpos) {
                return Some((contour_idx, point_idx, WhichHandle::Neither));
            } else if a_rect.contains(sk_mpos) {
                return Some((contour_idx, point_idx, WhichHandle::A));
            } else if b_rect.contains(sk_mpos) {
                return Some((contour_idx, point_idx, WhichHandle::B));
            }
        }
    }
    None
}

pub fn mouse_pressed(
    position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
    meta: MouseMeta,
) -> bool {
    let single_point = match clicked_point_or_handle(position, v) {
        Some((ci, pi, wh)) => TOOL_DATA.with(|p| {
            let follow: Follow = meta.into();
            debug!(
                "Clicked point: {:?} {:?}. Follow behavior: {}",
                get_outline!(v)[ci][pi],
                wh,
                follow
            );
            p.borrow_mut().contour = Some(ci);
            p.borrow_mut().cur_point = Some(pi);
            p.borrow_mut().follow = follow;
            p.borrow_mut().handle = wh;
            true
        }),
        None => TOOL_DATA.with(|p| {
            p.borrow_mut().contour = None;
            p.borrow_mut().cur_point = None;
            p.borrow_mut().handle = WhichHandle::Neither;
            false
        }),
    };

    if !single_point {
        v.borrow_mut().show_sel_box = true;
        let position = v.borrow().mousepos;
        v.borrow_mut().mousepos = position;
        if v.borrow().show_sel_box {
            v.borrow_mut().corner_one = Some(position);
            v.borrow_mut().corner_two = Some(position);
        }
    }
    false
}

pub fn mouse_released<T>(
    _position: (f64, f64),
    v: &RefCell<state::State<T>>,
    _meta: MouseMeta,
) -> bool {
    v.borrow_mut().show_sel_box = false;
    true
}

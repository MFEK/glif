// Pen
use super::prelude::*;

use glifparser::{self, Contour, Handle, Outline, Point, PointType};

pub fn mouse_moved(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<Option<PointData>>>,
) -> bool {
    let mposition = update_mousepos(position, &v, false);

    TOOL_DATA.with(|vv| {
        let contour = vv.borrow().contour;
        match contour {
            Some(idx) => {
                if v.borrow().mousedown {
                    let last_point = get_outline!(v)[idx].last().unwrap().clone();

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

pub fn mouse_pressed(
    _position: PhysicalPosition<f64>,
    v: &RefCell<state::State<Option<PointData>>>,
    _meta: MouseMeta,
) -> bool {
    let mposition = v.borrow().mousepos;

    TOOL_DATA.with(|vv| {
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

pub fn mouse_released(
    _position: PhysicalPosition<f64>,
    v: &RefCell<state::State<Option<PointData>>>,
    _meta: MouseMeta,
) -> bool {
    let _mposition = v.borrow().mousepos;

    TOOL_DATA.with(|vv| {
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

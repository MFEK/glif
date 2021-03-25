// Draw selection box
use super::constants::*;
use super::points::calc::*;
use skulpin::skia_safe::dash_path_effect;
use skulpin::skia_safe::{Canvas, Contains, Paint, PaintStyle, Path, Point, Rect};
use std::cell::RefCell;

use crate::state;
use crate::state::State;

use crate::glifparser;
use crate::util::math::FlipIfRequired as _;

pub fn draw_selbox<T>(canvas: &mut Canvas, v: &RefCell<State<T>>) -> Rect {
    let c1 = v
        .borrow()
        .corner_one
        .unwrap_or((0., 0.));
    let c2 = v
        .borrow()
        .corner_two
        .unwrap_or((0., 0.));

    let mut path = Path::new();
    let mut paint = Paint::default();
    let rect = Rect::from_point_and_size(
        (c1.0 as f32, c1.1 as f32),
        ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
    );
    path.add_rect(rect, None);
    path.close();
    paint.set_color(OUTLINE_STROKE);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / v.borrow().factor));
    let dash_offset = (1. / v.borrow().factor) * 2.;
    paint.set_path_effect(dash_path_effect::new(&[dash_offset, dash_offset], 0.0));
    canvas.draw_path(&path, &paint);

    rect
}

pub fn build_sel_vec_from_rect(
    mut rect: Rect,
    outline: Option<&Vec<glifparser::Contour<Option<state::PointData>>>>,
) -> Vec<glifparser::Point<Option<state::PointData>>> {
    rect.flip_if_required();

    let mut selected = Vec::new();
    for o in outline {
        for contour in o {
            for point in contour {
                if Rect::from(rect).contains(Point::from((calc_x(point.x), calc_y(point.y)))) {
                    selected.push(point.clone());
                }
            }
        }
    }
    selected
}

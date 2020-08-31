// Draw selection box
use super::constants::*;
use super::points::calc::*;
use glutin::dpi::PhysicalPosition;
use reclutch::skia::dash_path_effect;
use reclutch::skia::{Canvas, Contains, Paint, PaintStyle, Path, Point, Rect};
use std::cell::RefCell;

use crate::state;
use state::State;

use crate::glifparser;

pub fn draw_selbox(canvas: &mut Canvas, v: &RefCell<State>) -> Rect {
    let c1 = v
        .borrow()
        .corner_one
        .unwrap_or(PhysicalPosition { x: 0., y: 0. });
    let c2 = v
        .borrow()
        .corner_two
        .unwrap_or(PhysicalPosition { x: 0., y: 0. });

    let mut path = Path::new();
    let mut paint = Paint::default();
    let rect = Rect::from_point_and_size(
        (c1.x as f32, c1.y as f32),
        ((c2.x - c1.x) as f32, (c2.y - c1.y) as f32),
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
    rect: Rect,
    outline: Option<&Vec<glifparser::Contour>>,
) -> Vec<glifparser::Point> {
    let mut selected = Vec::new();
    for o in outline {
        for contour in o {
            for point in contour {
                if Rect::from(rect).contains(Point::from(point)) {
                    selected.push(point.clone());
                }
            }
        }
    }
    selected
}

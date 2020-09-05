use super::constants::*;
use super::points;
use super::points::calc::*;
use crate::glifparser;
use crate::state;
use glifparser::{Handle, OutlineType, WhichHandle};
use reclutch::skia::path::Iter;
use reclutch::skia::{Canvas, Paint, PaintStyle, Path, Point};
use state::State;
use std::cell::RefCell;

pub fn draw_glyph<T>(canvas: &mut Canvas, v: &RefCell<State<T>>) -> Path {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    //paint.set_stroke_width(PEN_SIZE.max(canvas.image_info().dimensions().width as f32 / 360.0));
    paint.set_style(PaintStyle::StrokeAndFill);
    paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / state.with(|v| v.borrow().factor)));

    let mut path = Path::new();

    paint.set_color(OUTLINE_FILL);

    let outline_type = v.borrow().glyph.as_ref().unwrap().glif.order;

    for outline in v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref() {
        for contour in outline {
            if contour.len() == 0 {
                continue;
            }
            path.move_to((calc_x(contour[0].x), calc_y(contour[0].y)));
            let firstpoint: &glifparser::Point<T> = contour.first().unwrap();
            let mut prevpoint: &glifparser::Point<T> = contour.first().unwrap();
            let mut pointiter = contour.iter().enumerate();
            for (i, point) in pointiter {
                match point.ptype {
                    glifparser::PointType::Line => {
                        path.line_to((calc_x(point.x), calc_y(point.y)));
                    }
                    glifparser::PointType::Curve => {
                        assert_eq!(outline_type, OutlineType::Cubic);
                        let h1 = prevpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                        let h2 = point.handle_or_colocated(WhichHandle::B, calc_x, calc_y);
                        path.cubic_to(h1, h2, (calc_x(point.x), calc_y(point.y)));
                    }
                    glifparser::PointType::QCurve => {
                        assert_eq!(outline_type, OutlineType::Quadratic);
                        let h1 = prevpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                        path.quad_to(h1, (calc_x(point.x), calc_y(point.y)));
                    }
                    _ => {}
                }
                prevpoint = &point;
            }
            match contour.last() {
                Some(lastpoint) => {
                    let h1 = lastpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                    let h2 = firstpoint.handle_or_colocated(WhichHandle::B, calc_x, calc_y);
                    match outline_type {
                        OutlineType::Cubic => {
                            path.cubic_to(h1, h2, (calc_x(firstpoint.x), calc_y(firstpoint.y)))
                        }
                        OutlineType::Quadratic => {
                            path.quad_to(h1, (calc_x(firstpoint.x), calc_y(firstpoint.y)))
                        }
                        OutlineType::Spiro => panic!("Spiro as yet unimplemented."),
                    };
                }
                None => {}
            }
            path.close();
        }

        //path.dump();
        canvas.draw_path(&path, &paint);
        paint.set_color(OUTLINE_STROKE);
        paint.set_style(PaintStyle::Stroke);
        canvas.draw_path(&path, &paint);
    }

    path
}

use super::constants::*;
use super::points;
use super::points::calc::*;
use crate::state::{PointData, PreviewMode, State};
use crate::STATE;
use glifparser::{self, Handle, OutlineType, PointType, WhichHandle};
use skulpin::skia_safe::path::Iter;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path, Point};
use std::cell::RefCell;

pub fn draw(canvas: &mut Canvas) -> Path {
    STATE.with(|v| {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        if v.borrow().preview_mode == PreviewMode::Paper {
            paint.set_style(PaintStyle::Fill);
            paint.set_color(PAPER_FILL);
        } else {
            paint.set_style(PaintStyle::StrokeAndFill);
            paint.set_stroke_width(
                OUTLINE_STROKE_THICKNESS * (1. / STATE.with(|v| v.borrow().factor)),
            );
            paint.set_color(OUTLINE_FILL);
        }

        let mut path = Path::new();

        let outline_type = v.borrow().glyph.as_ref().unwrap().glif.order;

        for outline in v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref() {
            for contour in outline {
                if contour.len() == 0 {
                    continue;
                }
                path.move_to((calc_x(contour[0].x), calc_y(contour[0].y)));
                let firstpoint: &glifparser::Point<Option<PointData>> = contour.first().unwrap();
                let mut prevpoint: &glifparser::Point<Option<PointData>> = contour.first().unwrap();
                let mut pointiter = contour.iter().enumerate();
                for (i, point) in pointiter {
                    match point.ptype {
                        PointType::Line => {
                            path.line_to((calc_x(point.x), calc_y(point.y)));
                        }
                        PointType::Curve => {
                            assert_eq!(outline_type, OutlineType::Cubic);
                            let h1 = prevpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                            let h2 = point.handle_or_colocated(WhichHandle::B, calc_x, calc_y);
                            path.cubic_to(h1, h2, (calc_x(point.x), calc_y(point.y)));
                        }
                        PointType::QCurve => {
                            assert_eq!(outline_type, OutlineType::Quadratic);
                            let h1 = prevpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                            path.quad_to(h1, (calc_x(point.x), calc_y(point.y)));
                        }
                        _ => {}
                    }
                    prevpoint = &point;
                }
                if firstpoint.ptype != PointType::Move {
                    match contour.last() {
                        Some(lastpoint) => {
                            let h1 = lastpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                            match outline_type {
                                OutlineType::Cubic => {
                                    let h2 = firstpoint.handle_or_colocated(
                                        WhichHandle::B,
                                        calc_x,
                                        calc_y,
                                    );
                                    path.cubic_to(
                                        h1,
                                        h2,
                                        (calc_x(firstpoint.x), calc_y(firstpoint.y)),
                                    )
                                }
                                OutlineType::Quadratic => {
                                    match lastpoint.ptype {
                                        PointType::QClose => {
                                            // This is safe as a lone QClose is illegal and should
                                            // cause a crash anyway if it's happening.
                                            let prevpoint = &contour[contour.len() - 2];
                                            let ph = prevpoint.handle_or_colocated(
                                                WhichHandle::A,
                                                calc_x,
                                                calc_y,
                                            );
                                            path.quad_to(ph, h1)
                                        }
                                        _ => path.quad_to(
                                            h1,
                                            (calc_x(firstpoint.x), calc_y(firstpoint.y)),
                                        ),
                                    }
                                }
                                OutlineType::Spiro => panic!("Spiro as yet unimplemented."),
                            };
                        }
                        None => {}
                    }
                    path.close();
                }
            }

            //Skia C++-compatible dump:
            //path.dump();
            canvas.draw_path(&path, &paint);
            if v.borrow().preview_mode != PreviewMode::Paper {
                paint.set_color(OUTLINE_STROKE);
                paint.set_style(PaintStyle::Stroke);
                canvas.draw_path(&path, &paint);
            }
        }
        path
    })
}

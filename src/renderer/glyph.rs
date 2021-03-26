use super::constants::*;

use super::points::calc::*;
use crate::state::{PointData, PreviewMode};
use crate::STATE;
use glifparser::{self, OutlineType, PointType, WhichHandle};

use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path};

pub use crate::state::Glyph; // types
pub use crate::state::{HandleStyle, PointLabels}; // enums
pub use crate::state::{Mode, CONSOLE, TOOL_DATA}; // globals

pub use crate::events::vws;

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
            for (idx, contour) in outline.iter().enumerate() {
                if contour.len() == 0 {
                    continue;
                }

                let should_draw = STATE.with(|v| vws::should_draw_contour(&v, idx));

                if !should_draw {
                    continue;
                }

                path.move_to((calc_x(contour[0].x), calc_y(contour[0].y)));
                let firstpoint: &glifparser::Point<Option<PointData>> = contour.first().unwrap();
                let mut prevpoint: &glifparser::Point<Option<PointData>> = contour.first().unwrap();
                let pointiter = contour.iter().enumerate();
                for (_i, point) in pointiter {
                    // the move_to handles the first point
                    if _i == 0 {
                        continue;
                    };
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

// TODO: Move the shared functionality between these two functions into it's own function.
pub fn draw_previews(canvas: &mut Canvas) -> Path {
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

        for outline in &v.borrow().vws_previews {
            for contour in outline {
                if contour.len() == 0 {
                    continue;
                }
                path.move_to((calc_x(contour[0].x), calc_y(contour[0].y)));
                let firstpoint: &glifparser::Point<Option<MFEKmath::piecewise::glif::PointData>> =
                    contour.first().unwrap();
                let mut prevpoint: &glifparser::Point<Option<MFEKmath::piecewise::glif::PointData>> = contour.first().unwrap();
                let pointiter = contour.iter().enumerate();
                for (_i, point) in pointiter {
                    if _i == 0 {
                        continue;
                    };
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

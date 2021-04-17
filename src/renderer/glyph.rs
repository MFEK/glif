use std::borrow::Borrow;

use super::constants::*;

use super::points::calc::*;
use crate::state::{PointData, PreviewMode, Editor};
use glifparser::{self, Contour, PointType, WhichHandle, OutlineType};

use skulpin::{skia_bindings::SkPath_AddPathMode, skia_safe::{Canvas, Paint, PaintStyle, Path}};

pub use crate::state::Glyph; // types
pub use crate::state::{HandleStyle, PointLabels}; // enums
pub use crate::state::{CONSOLE};
pub use crate::events::ToolEnum; // globals

//TODO: pub use crate::events::vws;

#[derive(Clone, Copy, PartialEq)]
pub enum SkPathBuildMode {
    Open,
    Closed,
    OpenAndClosed,
}

fn add_contour_to_skpath(
    contour: &Contour<PointData>,
    path: &mut Path,
    mode: SkPathBuildMode,
    outline_type: OutlineType,
    idx: usize,
) {
    if contour.len() == 0 {
        return;
    }
    let should_draw = true; //TODO: STATE.with(|v| vws::should_draw_contour(&v, idx));

    if !should_draw {
        return;
    }

    let firstpoint: &glifparser::Point<PointData> = contour.first().unwrap();
    let mut prevpoint: &glifparser::Point<PointData> = contour.first().unwrap();
    let pointiter = contour.iter().enumerate();

    match mode {
        SkPathBuildMode::Open => {
            if firstpoint.ptype != PointType::Move {
                return;
            }
        }
        SkPathBuildMode::Closed => {
            if firstpoint.ptype == PointType::Move {
                return;
            }
        }
        SkPathBuildMode::OpenAndClosed => {}
    }

    path.move_to((calc_x(contour[0].x), calc_y(contour[0].y)));

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
                        let h2 = firstpoint.handle_or_colocated(WhichHandle::B, calc_x, calc_y);
                        path.cubic_to(h1, h2, (calc_x(firstpoint.x), calc_y(firstpoint.y)))
                    }
                    OutlineType::Quadratic => {
                        match lastpoint.ptype {
                            PointType::QClose => {
                                // This is safe as a lone QClose is illegal and should
                                // cause a crash anyway if it's happening.
                                let prevpoint = &contour[contour.len() - 2];
                                let ph =
                                    prevpoint.handle_or_colocated(WhichHandle::A, calc_x, calc_y);
                                path.quad_to(ph, h1)
                            }
                            _ => path.quad_to(h1, (calc_x(firstpoint.x), calc_y(firstpoint.y))),
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

pub fn draw(v: &Editor, canvas: &mut Canvas) -> Path {

    let mut total_path = Path::new();
    v.with_glif(|glif| {
        let outline_type = glif.order;

        for (layer_idx, layer) in glif.layers.iter().enumerate() {
            let mut path = Path::new();
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
        
            if v.preview_mode == PreviewMode::Paper {
                paint.set_style(PaintStyle::Fill);
                paint.set_color(PAPER_FILL);
            } else {
                paint.set_style(PaintStyle::StrokeAndFill);
                paint.set_stroke_width(
                    OUTLINE_STROKE_THICKNESS * (1. / v.factor),
                );
                paint.set_color(OUTLINE_FILL);
            }

            for outline in &layer.outline {
                for (idx, contour) in outline.iter().enumerate() {
                    add_contour_to_skpath(
                        &contour,
                        &mut path,
                        SkPathBuildMode::Closed,
                        outline_type,
                        idx,
                    );
                }
    
                canvas.draw_path(&path, &paint);
                path = Path::new();
                paint.set_style(PaintStyle::Stroke);
    
                for (idx, contour) in outline.iter().enumerate() {
                    add_contour_to_skpath(
                        &contour,
                        &mut path,
                        SkPathBuildMode::Open,
                        outline_type,
                        idx,
                    );
                }
    
                canvas.draw_path(&path, &paint);
    
                path = Path::new();
    
                for (idx, contour) in outline.iter().enumerate() {
                    add_contour_to_skpath(
                        &contour,
                        &mut path,
                        SkPathBuildMode::OpenAndClosed,
                        outline_type,
                        idx,
                    );
                }
    
                if v.borrow().preview_mode != PreviewMode::Paper {
                    paint.set_color(OUTLINE_STROKE);
                    paint.set_style(PaintStyle::Stroke);
                    canvas.draw_path(&path, &paint);
                }
            }

            if Some(layer_idx) == v.layer_idx { total_path = path; }
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(OUTLINE_STROKE);
        paint.set_style(PaintStyle::Stroke);
    
        if v.preview_mode == PreviewMode::Paper { return };

        for outline in glif.layers[v.layer_idx.unwrap()].outline.as_ref() {
            let mut path = Path::new();            

            for (idx, contour) in outline.iter().enumerate() {
                add_contour_to_skpath(
                    &contour,
                    &mut path,
                    SkPathBuildMode::OpenAndClosed,
                    outline_type,
                    idx,
                );
            }

            canvas.draw_path(&path, &paint);


            if v.borrow().preview_mode != PreviewMode::Paper {
                paint.set_color(OUTLINE_STROKE);
                paint.set_style(PaintStyle::Stroke);
                canvas.draw_path(&path, &paint);
            }
        }
    });

    return total_path;
}

// TODO: Move the shared functionality between these two functions into it's own function.
/* 
pub fn draw_previews(v: &State, canvas: &mut Canvas) -> Path {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    if v.borrow().preview_mode == PreviewMode::Paper {
        paint.set_style(PaintStyle::Fill);
        paint.set_color(PAPER_FILL);
    } else {
        paint.set_style(PaintStyle::StrokeAndFill);
        paint.set_stroke_width(
            OUTLINE_STROKE_THICKNESS * (1. / v.factor),
        );
        paint.set_color(OUTLINE_FILL);
    }

    let mut path = Path::new();

    let outline_type = v.with_glif(&|glif| {glif.order});

    for outline in &v.borrow().vws_previews {
        for contour in outline {
            if contour.len() == 0 {
                continue;
            }
            path.move_to((calc_x(contour[0].x), calc_y(contour[0].y)));
            let firstpoint: &glifparser::Point<Option<()>> =
                contour.first().unwrap();
            let mut prevpoint: &glifparser::Point<
                Option<Option<()>>,
            > = contour.first().unwrap();
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
}
*/
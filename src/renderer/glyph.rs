use super::constants::*;
use super::points::calc::*;

use glifparser::{MFEKGlif, glif::{LayerOperation, MFEKPointData}, outline::skia::{ToSkiaPaths, SkiaPointTransforms}};
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path, PathOp};

use crate::editor::{Editor, PreviewMode, Viewport};
pub use crate::editor::{HandleStyle, PointLabels, Glyph, CONSOLE}; // enums
pub use crate::tools::ToolEnum; // globals

//TODO: pub use crate::events::vws;

#[derive(Clone, Copy, PartialEq)]
pub enum SkPathBuildMode {
    Open,
    Closed,
    OpenAndClosed,
}

pub fn draw(v: &Editor, canvas: &mut Canvas) -> Path {
    let mut total_path = Path::new();
    v.with_glif(|glif| {
        for (layer_idx, layer) in glif.layers.iter().enumerate() {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
        
            if v.viewport.preview_mode == PreviewMode::Paper {
                paint.set_style(PaintStyle::Fill);
                paint.set_color(PAPER_FILL);
            } else {
                paint.set_style(PaintStyle::StrokeAndFill);
                paint.set_stroke_width(
                    OUTLINE_STROKE_THICKNESS * (1. / v.viewport.factor),
                );
                paint.set_color(OUTLINE_FILL);
            }

            for outline in &layer.outline {
                let skpaths = outline.to_skia_paths(Some(SkiaPointTransforms{calc_x: calc_x, calc_y: calc_y}));

                skpaths.closed.as_ref().map(|p| canvas.draw_path(&p, &paint));

                paint.set_style(PaintStyle::Stroke);
                skpaths.open.as_ref().map(|p| canvas.draw_path(&p, &paint));
    
                if v.viewport.preview_mode != PreviewMode::Paper {
                    paint.set_color(OUTLINE_STROKE);
                    paint.set_style(PaintStyle::Stroke);
                    skpaths.closed.as_ref().map(|p| canvas.draw_path(&p, &paint));
                }

                if layer_idx == v.get_active_layer() {
                    total_path = skpaths.into();
                }
            }
        }

        if v.viewport.preview_mode == PreviewMode::Paper { return };
    });

    return total_path;
}

// Before we draw we've got to build a flattened path out of the glyph by resolving
// each layer operation in turn.
pub fn draw_mfek( canvas: &mut Canvas, glif: &MFEKGlif<MFEKPointData>, viewport: &Viewport, active_layer: usize)  -> Path {
    let mut active_path = Path::new();
    let mut total_open_path = Path::new();
    let mut total_closed_path = Path::new();
    let mut total_outline_path = Path::new();

    for (layer_idx, layer) in glif.layers.iter().enumerate() {
        if let Some(outline) = layer.outline.as_ref() {
            let skpaths = outline.to_skia_paths(Some(SkiaPointTransforms{calc_x: calc_x, calc_y: calc_y}));

            if layer_idx == active_layer {
                active_path = skpaths.clone().into();
            }

            if let Some(op) = &layer.operation {
                let pathop = match op {
                    LayerOperation::Difference  => PathOp::Difference,
                    LayerOperation::Union  => PathOp::Union,
                    LayerOperation::Intersect => PathOp::Intersect,
                    LayerOperation::XOR => PathOp::XOR
                };

                if let Some(open) = skpaths.open {
                    total_open_path.add_path(&open, (0., 0.), skulpin::skia_safe::path::AddPathMode::Append);
                }

                if let Some(closed) = skpaths.closed {
                    total_outline_path.add_path(&closed, (0., 0.), skulpin::skia_safe::path::AddPathMode::Append);
                    if let Some(result) = total_closed_path.op(&closed, pathop) {
                        total_closed_path = Path::new();
                        total_closed_path.reverse_add_path(&result.as_winding().unwrap());
                    }
                    else 
                    {
                        total_closed_path.add_path(&closed, (0., 0.), skulpin::skia_safe::path::AddPathMode::Append);
                    }
                }
            }
            else
            {
                if let Some(open) = skpaths.open {
                    total_open_path.add_path(&open, (0., 0.), skulpin::skia_safe::path::AddPathMode::Append);
                }

                if let Some(closed) = skpaths.closed {
                    total_outline_path.add_path(&closed, (0., 0.), skulpin::skia_safe::path::AddPathMode::Append);
                    total_closed_path.add_path(&closed, (0., 0.), skulpin::skia_safe::path::AddPathMode::Append);
                }
            }

        }
    }

    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    if viewport.preview_mode == PreviewMode::Paper {
        paint.set_style(PaintStyle::Fill);
        paint.set_color(PAPER_FILL);
    } else {
        paint.set_style(PaintStyle::StrokeAndFill);
        paint.set_stroke_width(
            OUTLINE_STROKE_THICKNESS * (1. / viewport.factor),
        );
        paint.set_color(OUTLINE_FILL);
    }

    canvas.draw_path(&total_closed_path, &paint);

    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&total_open_path, &paint);

    if viewport.preview_mode != PreviewMode::Paper {
        paint.set_color(OUTLINE_STROKE);
        paint.set_style(PaintStyle::Stroke);
        canvas.draw_path(&total_closed_path, &paint);
        canvas.draw_path(&total_outline_path, &paint);
    }

    return active_path;    
}
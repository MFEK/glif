use super::constants::*;
use super::points::calc::*;

use glifparser::{glif::{LayerOperation}, outline::skia::{ToSkiaPaths, SkiaPointTransforms}};
use glifparser::FlattenedGlif;
use skulpin::skia_safe::{Canvas, Color4f, Paint, PaintStyle, Path, PathOp, Rect};

use crate::editor::{Editor, PreviewMode};
pub use crate::editor::{HandleStyle, PointLabels, CONSOLE}; // enums
pub use crate::tools::ToolEnum; // globals
use crate::renderer::string::UiString;

pub fn draw_components(v: &Editor, canvas: &mut Canvas) {
    let glif = v.preview.as_ref().unwrap();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(OUTLINE_STROKE);
    paint.set_style(PaintStyle::Stroke);
    let mut path = Path::new();
    for rect in glif.component_rects.as_ref().unwrap() {
        let skrect = Rect::new(calc_x(rect.minx), calc_y(rect.miny), calc_x(rect.maxx), calc_y(rect.maxy));
        let uis = UiString::with_colors(&rect.name, COMPONENT_NAME_COLOR, None);
        uis.draw(v, (calc_x(rect.minx), calc_y(rect.maxy)), canvas);
        path.add_rect(skrect, None);
    }
    let skpaths = glif.flattened.as_ref().map(|f|f.to_skia_paths(Some(SkiaPointTransforms{calc_x: calc_x, calc_y: calc_y})));
    skpaths.map(|skp|skp.closed.map(|skpc|canvas.draw_path(&skpc, &paint)));
    canvas.draw_path(&path, &paint);
}

//TODO: pub use crate::events::vws;
// Before we draw we've got to build a flattened path out of the glyph by resolving
// each layer operation in turn.
pub fn draw(canvas: &mut Canvas, v: &mut Editor, active_layer: usize)  -> Path {
    let glif = v.preview.as_mut().unwrap();
    let mut active_path = Path::new();
    let mut total_open_path = Path::new();
    let mut total_closed_path = Path::new();
    let mut total_outline_path = Path::new();
    let mut root_color = if let Some(color) = glif.layers[0].color {
        Some(Color4f::new(color[0],color[1],color[2],color[3]))
    } else {
        None
    };


    for (layer_idx, layer) in glif.layers.iter().enumerate() {
        if !layer.visible { continue; }

        if layer.operation.is_none() && layer_idx != 0 {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            
            if v.viewport.preview_mode == PreviewMode::Paper {
                paint.set_style(PaintStyle::Fill);
            } else {
                paint.set_style(PaintStyle::StrokeAndFill);
                paint.set_color(OUTLINE_FILL);
                paint.set_stroke_width(
                    OUTLINE_STROKE_THICKNESS * (1. / v.viewport.factor),
                );
            }

            if let Some(color) = root_color {
                paint.set_color4f(color, None);
            }

            canvas.draw_path(&total_closed_path, &paint);

            paint.set_style(PaintStyle::Stroke);
            canvas.draw_path(&total_open_path, &paint);

            if v.viewport.preview_mode != PreviewMode::Paper {
                paint.set_color(OUTLINE_STROKE);
                if let Some(color) = root_color {
                    paint.set_color4f(color, None);
                }
                paint.set_style(PaintStyle::Stroke);
                canvas.draw_path(&total_closed_path, &paint);
                canvas.draw_path(&total_outline_path, &paint);
            }
            
            total_open_path = Path::new();
            total_closed_path = Path::new();
            total_outline_path = Path::new();

            root_color = if let Some(color) = layer.color {
                Some(Color4f::new(color[0],color[1],color[2],color[3]))
            } else {
                None
            };
        }

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
    
    if v.viewport.preview_mode == PreviewMode::Paper {
        paint.set_style(PaintStyle::Fill);
    } else {
        paint.set_style(PaintStyle::StrokeAndFill);
        paint.set_color(OUTLINE_FILL);
        paint.set_stroke_width(
            OUTLINE_STROKE_THICKNESS * (1. / v.viewport.factor),
        );
    }

    if let Some(color) = root_color {
        paint.set_color4f(color, None);
    }

    canvas.draw_path(&total_closed_path, &paint);

    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&total_open_path, &paint);

    if v.viewport.preview_mode != PreviewMode::Paper {
        paint.set_style(PaintStyle::Stroke);
        canvas.draw_path(&total_closed_path, &paint);
        canvas.draw_path(&total_outline_path, &paint);
    }

    // Cache component rects and flattened outline on MFEKGlif
    match &glif.component_rects {
        Some(_) => {draw_components(v, canvas);},
        None => {
            let mut rects = Some(vec![]);
            let flattened = glif.flattened(&mut rects);
            flattened.map(|f| {
                glif.flattened = f.flattened;
                glif.component_rects = rects;
            });
        },
    }

    return active_path;    
}

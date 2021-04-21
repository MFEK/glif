use super::constants::*;
use super::points::calc::*;

use glifparser::outline::skia::{ToSkiaPaths, SkiaPointTransforms};
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path};

use crate::editor::{PreviewMode, Editor};
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
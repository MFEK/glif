use super::constants::*;
use super::points::calc::{calc_x, calc_y};
use super::string::UiString;

use crate::editor::Editor;
use crate::user_interface::viewport::Viewport;

use glifparser::Anchor;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path as SkPath};

pub fn draw_anchors(v: &mut Editor, viewport: &Viewport, canvas: &mut Canvas) {
    v.with_glyph(|glif| {
        for anchor in &glif.anchors {
            draw_anchor(&anchor, viewport, canvas);
        }
    });
}

fn draw_anchor(anchor: &Anchor, viewport: &Viewport, canvas: &mut Canvas) {
    let mut path = SkPath::new();
    let (x, y) = (calc_x(anchor.x), calc_y(anchor.y));
    let radius = ANCHOR_RADIUS * (1. / viewport.factor);
    path.move_to((x - radius, y));
    path.quad_to((x, y), (x, y + radius));
    path.quad_to((x, y), (x + radius, y));
    path.quad_to((x, y), (x, y - radius));
    path.quad_to((x, y), (x - radius, y));
    path.close();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    paint.set_color(ANCHOR_FILL);
    canvas.draw_path(&path, &paint);
    paint.set_style(PaintStyle::Stroke);
    paint.set_color(ANCHOR_STROKE);
    paint.set_stroke_width(ANCHOR_STROKE_THICKNESS * (1. / viewport.factor));
    canvas.draw_path(&path, &paint);
    let uis =
        UiString::centered_with_colors(&anchor.class, ANCHOR_NAME_COLOR, Some(ANCHOR_NAME_BGCOLOR));
    uis.draw(viewport, (x, y - (radius * 1.3)), canvas);
}

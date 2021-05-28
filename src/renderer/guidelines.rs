use crate::editor::Editor;
use crate::user_interface::viewport::Viewport;

use super::constants::*;
use super::points::calc::*;
use skulpin::skia_safe::{Canvas, Color, Paint, PaintStyle, Path};

use glifparser::{Guideline, GuidelinePoint};
use glifparser::IntegerOrFloat;

pub fn draw_guideline(v: &Editor, viewport: &Viewport, canvas: &mut Canvas, guideline: &Guideline, color: Option<u32>) {
    let angle = guideline.angle * DEGREES_IN_RADIANS;
    let _extra = (viewport.offset.0 * (1. / viewport.factor), viewport.offset.1 * (1. / viewport.factor));
    let at2 = GuidelinePoint { x: guideline.at.x+((1000.*viewport.winsize.0 as f32)*f32::from(angle).cos()), y: guideline.at.y+((1000.*viewport.winsize.1 as f32)*f32::from(angle).sin()) };
    let at3 = GuidelinePoint { x: guideline.at.x+((-(1000.*viewport.winsize.0 as f32))*f32::from(angle).cos()), y: guideline.at.y+((-(1000.*viewport.winsize.1 as f32))*f32::from(angle).sin()) };
    let factor = viewport.factor;
    let mut path = Path::new();
    path.move_to((calc_x(at2.x), calc_y(at2.y)));
    path.line_to((calc_x(at3.x), calc_y(at3.y)));
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let color = color.map(|c|Color::from(c)).unwrap_or(Color::from(LBEARING_STROKE));
    paint.set_color(color);
    paint.set_stroke_width(GUIDELINE_THICKNESS * (1. / factor));
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&path, &paint);
}

pub fn draw_lbearing(v: &Editor, viewport: &Viewport, canvas: &mut Canvas) {
    draw_guideline(
        v,
        viewport,
        canvas,
        &Guideline::from_x_y_angle(0., 0., IntegerOrFloat::Float(90.)),
        Some(LBEARING_STROKE),
    );
}

pub fn draw_rbearing(v: &Editor, viewport: &Viewport, width: u64, canvas: &mut Canvas) {
    draw_guideline(
        v,
        viewport,
        canvas,
        &Guideline::from_x_y_angle(width as f32, 0., IntegerOrFloat::Float(90.)),
        Some(RBEARING_STROKE),
    );
}

pub fn draw_baseline(v: &Editor, viewport: &Viewport, canvas: &mut Canvas) {
    draw_guideline(
        v,
        viewport,
        canvas,
        &Guideline::from_x_y_angle(0., 0., IntegerOrFloat::Float(0.)),
        None,
    );
}

pub fn draw_all(v: &Editor, viewport: &Viewport, canvas: &mut Canvas) {
    draw_lbearing(v, viewport, canvas);
    match v.with_glyph(|glif| glif.width) {
        Some(w) => draw_rbearing(v, viewport, w, canvas),
        None => {}
    }
    draw_baseline(v, viewport, canvas);

    v.with_glyph(|glyph| {
        for guideline in &glyph.guidelines {
            draw_guideline(
                v, 
                viewport,
                canvas,
                guideline,
                None,
            );
        }
    })
}

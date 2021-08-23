use crate::editor::Editor;
use crate::user_interface::viewport::Viewport;

use super::constants::*;
use super::points::calc::*;
use skulpin::skia_safe::{Canvas, Color, Paint, PaintStyle, Path};

use glifparser::IntegerOrFloat;
use glifparser::{Guideline, GuidelinePoint};

pub fn draw_guideline(
    viewport: &Viewport,
    canvas: &mut Canvas,
    guideline: &Guideline,
    color: Option<u32>,
) {
    let angle = guideline.angle * DEGREES_IN_RADIANS;
    let _extra = (
        viewport.offset.0 * (1. / viewport.factor),
        viewport.offset.1 * (1. / viewport.factor),
    );
    let at2 = GuidelinePoint {
        x: guideline.at.x + ((1000. * viewport.winsize.0 as f32) * f32::from(angle).cos()),
        y: guideline.at.y + ((1000. * viewport.winsize.1 as f32) * f32::from(angle).sin()),
    };
    let at3 = GuidelinePoint {
        x: guideline.at.x + ((-(1000. * viewport.winsize.0 as f32)) * f32::from(angle).cos()),
        y: guideline.at.y + ((-(1000. * viewport.winsize.1 as f32)) * f32::from(angle).sin()),
    };
    let factor = viewport.factor;
    let mut path = Path::new();
    path.move_to((calc_x(at2.x), calc_y(at2.y)));
    path.line_to((calc_x(at3.x), calc_y(at3.y)));
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let color = color
        .map(|c| Color::from(c))
        .unwrap_or(Color::from(LBEARING_STROKE));
    paint.set_color(color);
    paint.set_stroke_width(GUIDELINE_THICKNESS * (1. / factor));
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&path, &paint);
}

pub fn draw_lbearing(viewport: &Viewport, canvas: &mut Canvas) {
    draw_guideline(
        viewport,
        canvas,
        &Guideline::from_x_y_angle(0., 0., IntegerOrFloat::Float(90.)),
        Some(LBEARING_STROKE),
    );
}

pub fn draw_rbearing(viewport: &Viewport, width: u64, canvas: &mut Canvas) {
    draw_guideline(
        viewport,
        canvas,
        &Guideline::from_x_y_angle(width as f32, 0., IntegerOrFloat::Float(90.)),
        Some(RBEARING_STROKE),
    );
}

pub fn draw_baseline(viewport: &Viewport, canvas: &mut Canvas) {
    draw_guideline(
        viewport,
        canvas,
        &Guideline::from_x_y_angle(0., 0., IntegerOrFloat::Float(0.)),
        None,
    );
}

pub fn draw_all(v: &Editor, viewport: &Viewport, canvas: &mut Canvas) {
    draw_lbearing(viewport, canvas);
    match v.with_glyph(|glif| glif.width) {
        Some(w) => draw_rbearing(viewport, w, canvas),
        None => {}
    }
    draw_baseline(viewport, canvas);

    v.with_glyph(|glyph| {
        // These draw guidelines defined in the specific glyph, if any.
        // e.g., in a multi-script font, the Hebrew glyphs may define a "Hebrew x-height",
        // which would be different than the Latin x-height.
        for guideline in &glyph.guidelines {
            draw_guideline(viewport, canvas, guideline, None);
        }
    });

    // These draw the UFO-global guidelines. This includes always (in a valid UFO) ascender and
    // descender, but in a single-script font, like for example a Latin-only font, may include
    // user-defined guidelines like x-height, cap-height, "digit height", etc., which are not
    // meaningful to the output format.
    for guideline in &v.guidelines {
        draw_guideline(viewport, canvas, guideline, Some(UFO_GUIDELINE_STROKE));
    }
}

use crate::state::Editor;

use super::constants::*;
use super::points::calc::*;
use skulpin::skia_safe::{Canvas, Color, Paint, PaintStyle, Path};
#[derive(Clone)]
pub enum GuidelineType {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct Guideline {
    pub gtype: GuidelineType,
    pub where_: f32,
    pub selected: bool,
    pub name: Option<String>,
}

pub fn draw_guideline(v: &Editor, color: Color, where_: f32, gtype: GuidelineType, canvas: &mut Canvas) {
    let mut paint = Paint::default();
    let mut path = Path::new();
    let factor = v.factor;
    let offset = v.offset;
    match gtype {
        GuidelineType::Vertical => {
            path.move_to((where_, -(offset.1 * (1. / factor))));
            path.line_to((
                where_,
                v.winsize.1 as f32 * (1. / factor) + -(offset.1 * (1. / factor)),
            ));
        }
        GuidelineType::Horizontal => {
            path.move_to((-(offset.0 * (1. / factor)), where_));
            path.line_to((
                (v.winsize.0 as f32 * (1. / factor)) + (-(offset.0 * (1. / factor))),
                where_,
            ));
        }
    }
    path.close();
    paint.set_anti_alias(true);
    paint.set_color(color);
    paint.set_stroke_width(GUIDELINE_THICKNESS * (1. / factor));
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&path, &paint);
}

pub fn draw_lbearing(v: &Editor, canvas: &mut Canvas) {
    draw_guideline(
        v,
        Color::from(LBEARING_STROKE),
        0.,
        GuidelineType::Vertical,
        canvas,
    );
}

pub fn draw_rbearing(v: &Editor, width: u64, canvas: &mut Canvas) {
    draw_guideline(
        v,
        Color::from(RBEARING_STROKE),
        width as f32,
        GuidelineType::Vertical,
        canvas,
    );
}

pub fn draw_baseline(v: &Editor, canvas: &mut Canvas) {
    draw_guideline(
        v,
        Color::from(LBEARING_STROKE),
        calc_y(0.),
        GuidelineType::Horizontal,
        canvas,
    );
}

pub fn draw_all(v: &Editor, canvas: &mut Canvas) {
    draw_lbearing(v, canvas);
    match v.with_glif(|glif| glif.width) {
        Some(w) => draw_rbearing(v, w, canvas),
        None => {}
    }
    draw_baseline(v, canvas);

    v.with_glyph(|glyph| {
        for guideline in &glyph.guidelines {
            draw_guideline(
                v, 
                Color::from(LBEARING_STROKE),
                calc_y(guideline.where_),
                GuidelineType::Horizontal,
                canvas,
            );
        }
    })

}

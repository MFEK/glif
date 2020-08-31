use super::constants::*;
use super::points::calc::*;
use crate::state;
use reclutch::skia::{Canvas, Color, Paint, PaintStyle, Path};
use state::State;

enum GuidelineType {
    Horizontal,
    Vertical,
}

fn draw_guideline(color: Color, where_: f32, gtype: GuidelineType, canvas: &mut Canvas) {
    let mut paint = Paint::default();
    let mut path = Path::new();
    let factor = state.with(|v| v.borrow().factor);
    let offset = state.with(|v| v.borrow().offset);
    match gtype {
        GuidelineType::Vertical => {
            state.with(|v| {
                path.move_to((where_, -v.borrow().offset.1 * (1. / factor)));
                path.line_to((where_, v.borrow().winsize.height as f32 * (1. / factor)));
            });
        }
        GuidelineType::Horizontal => {
            path.move_to((0., where_));
            path.line_to((
                state.with(|v| {
                    (v.borrow().winsize.width as f32 * (1. / factor))
                        + (-(v.borrow().offset.0 * (1. / factor)))
                }),
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

pub fn draw_lbearing(canvas: &mut Canvas) {
    draw_guideline(
        Color::from(LBEARING_STROKE),
        0.,
        GuidelineType::Vertical,
        canvas,
    );
}

pub fn draw_rbearing(width: u64, canvas: &mut Canvas) {
    draw_guideline(
        Color::from(RBEARING_STROKE),
        width as f32,
        GuidelineType::Vertical,
        canvas,
    );
}

pub fn draw_baseline(canvas: &mut Canvas) {
    draw_guideline(
        Color::from(LBEARING_STROKE),
        calc_y(0.),
        GuidelineType::Horizontal,
        canvas,
    );
}

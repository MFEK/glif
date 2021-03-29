use skulpin::skia_safe::{Canvas, Font, FontStyle, Paint, Path, Rect, TextBlob, Typeface};

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{STATE, TOOL_DATA}; // for State.factor

pub fn draw_point_number(at: (f32, f32), number: isize, canvas: &mut Canvas) {
    let converted = number.to_string();
    draw_string_at_point(at, &converted, canvas);
}

pub fn draw_point_location(at: (f32, f32), original: (f32, f32), canvas: &mut Canvas) {
    let converted = format!("{}, {}", original.0, original.1);
    draw_string_at_point(at, &converted, canvas);
}

fn pointfont_from_size_and_factor(size: f32, factor: f32) -> Font {
    Font::from_typeface_with_params(
        Typeface::from_name("", FontStyle::bold()).expect("Failed to load bold font"),
        14.0 * 1. / factor,
        1.0,
        0.0,
    )
}

// Creating the font is an expensive operation to do every frame. So, we keep a cache of fonts
// based on the current zoom.
thread_local! {
    static POINTFONTS: RefCell<HashMap<usize, Font>> = {
        let factor = STATE.with(|v| v.borrow().factor);
        let mut h = HashMap::new();
        let font = pointfont_from_size_and_factor(14.0, factor);
        h.insert((14.0 * 1. / factor).round() as usize, font);
        RefCell::new(h)
    };
}

fn draw_string_at_point(mut at: (f32, f32), s: &str, canvas: &mut Canvas) {
    let factor = STATE.with(|v| v.borrow().factor);
    let mut paint = Paint::default();
    paint.set_color(0xff_ff0000);
    paint.set_anti_alias(true);

    let (blob, rect) = {
        POINTFONTS.with(|f| {
            let mut hm = f.borrow_mut();
            let f = hm.get(&((14.0 * 1. / factor).round() as usize));
            let font = match f {
                Some(fon) => fon,
                None => {
                    hm.insert(
                        (14.0 * 1. / factor).round() as usize,
                        pointfont_from_size_and_factor(14.0, factor),
                    );
                    hm.get(&((14.0 * 1. / factor).round() as usize)).unwrap()
                }
            };

            let blob = TextBlob::from_str(s, font).expect(&format!("Failed to shape {}", s));
            let (_, rect) = font.measure_str(s, Some(&paint));
            (blob, rect)
        })
    };

    let mut paint2 = Paint::default();
    paint2.set_color(0xaa_ffffff);
    paint2.set_anti_alias(true);
    let mut path = Path::new();
    let padding = 5.;
    at = (at.0, at.1 - (padding + 20. * (1. / factor)));
    let at_rect = Rect::from_point_and_size(at, (rect.width() + 5., rect.height() + 5.));
    path.add_rect(at_rect, None);
    path.close();
    canvas.draw_path(&path, &paint2);

    at = (
        at.0 + (padding / 2.),
        at.1 + ((padding / 2.) + 10. * (1. / factor)),
    );
    canvas.draw_text_blob(&blob, at, &paint);
}

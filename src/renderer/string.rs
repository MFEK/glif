use skulpin::skia_safe::{Canvas, Color, Font, FontStyle, Paint, Path, Rect, TextBlob, Typeface};

use crate::editor::Editor;

use std::cell::RefCell;
use std::collections::HashMap;

use super::constants::*;

pub static POINTFONTSIZE: f32 = 14.0;

pub struct UiString<'a> {
    pub string: &'a str,
    pub color: u32,
    pub bgcolor: Option<u32>,
    pub centered: bool,
    pub size: f32,
    pub padding: Option<f32>,
}

impl<'a> UiString<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            string: s,
            size: POINTFONTSIZE,
            bgcolor: Some(DEFAULT_STRING_BGCOLOR),
            color: DEFAULT_STRING_COLOR,
            centered: false,
            padding: None,
        }
    }

    pub fn centered(s: &'a str) -> Self {
        let mut ret = Self::new(s);
        ret.centered = true;
        ret
    }

    pub fn with_colors(s: &'a str, color: u32, bgcolor: Option<u32>) -> Self {
        let mut ret = Self::new(s);
        ret.color = color;
        ret.bgcolor = bgcolor;
        ret
    }

    pub fn centered_with_colors(s: &'a str, color: u32, bgcolor: Option<u32>) -> Self {
        let mut ret = Self::with_colors(s, color, bgcolor);
        ret.centered = true;
        ret
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }
}

// Creating the font is an expensive operation to do every frame. So, we keep a cache of fonts
// based on the current zoom.
thread_local! {
    pub static POINTFONTS: RefCell<HashMap<usize, Font>> = {
        let factor = 1.;
        let mut h = HashMap::new();
        let font = pointfont_from_size_and_factor(14.0, factor);
        h.insert((14.0 * 1. / factor).round() as usize, font);
        RefCell::new(h)
    };
}

pub fn pointfont_from_size_and_factor(size: f32, factor: f32) -> Font {
    Font::from_typeface_with_params(
        Typeface::from_name("", FontStyle::bold()).expect("Failed to load bold font"),
        size * 1. / factor,
        1.0,
        0.0,
    )
}

impl UiString<'_> {
    pub fn draw(&self, v: &Editor, mut at: (f32, f32), canvas: &mut Canvas) {
        let factor = v.viewport.factor;
        let mut paint = Paint::default();
        paint.set_color(self.color);
        paint.set_anti_alias(true);

        let (blob, rect) = {
            POINTFONTS.with(|f| {
                let mut hm = f.borrow_mut();
                let f = hm.get(&((self.size * 1. / factor).round() as usize));
                let font = match f {
                    Some(fon) => fon,
                    None => {
                        hm.insert(
                            (self.size * 1. / factor).round() as usize,
                            pointfont_from_size_and_factor(self.size, factor),
                        );
                        hm.get(&((self.size * 1. / factor).round() as usize))
                            .unwrap()
                    }
                };

                let blob = TextBlob::from_str(self.string, font)
                    .expect(&format!("Failed to shape {}", self.string));
                let (_, rect) = font.measure_str(self.string, Some(&paint));
                (blob, rect)
            })
        };

        let center = if self.centered {
            -(rect.width() / 2.)
        } else {
            0.
        };

        let mut padding = 0.;
        if self.padding.is_some() {
            padding = self.padding.unwrap() * (1. / factor);
            at = (at.0 - padding, at.1 - padding);
        }

        if let Some(bgcolor) = self.bgcolor {
            let mut paint2 = Paint::default();
            paint2.set_color(bgcolor);
            paint2.set_anti_alias(true);
            let mut path = Path::new();
            let at_rect = Rect::from_point_and_size(
                (at.0 - (padding / 2.), at.1 - self.size),
                (rect.width() + 5. + (padding / 2.), rect.height() + 5.),
            );
            path.add_rect(at_rect, None);
            path.close();
            canvas.draw_path(&path, &paint2);
        }

        at = (at.0 + center, at.1);
        canvas.draw_text_blob(&blob, at, &paint);
    }
}

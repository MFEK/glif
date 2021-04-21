//! Our Console is not a normal Unix console. stdin is always one line long, and stdout can be any
//! number of lines but disappears often and is not possible to be scrolled back. (It is always the
//! output of a single Command.) That's because this is a *Renderer Console*, not supposed to
//! represent the Console itself, but rather just what we show the user on the screen. We output to
//! the normal stdout as well, that's the persistent stdout.

use lazy_static::lazy_static;

pub struct Console {
    pub stdin: String,
    pub stdout: String,
    // Pressing `:` activates the console, like in Vim.
    pub active: bool,
}

impl Default for Console {
    fn default() -> Self {
        Console {
            stdin: String::new(),
            stdout: String::new(),
            active: false,
        }
    }
}

impl Console {
    pub fn active(&mut self, b: bool) {
        self.active = b;
    }
}

use skulpin::skia_safe::{Data, Font, FontStyle, Matrix, Typeface};

use crate::{editor::Editor, system_fonts};
lazy_static! {
    static ref MONO_FONT_BYTES: Option<Vec<u8>> = {
        match system_fonts::SYSTEMMONO.path {
            Some(_) => Some(system_fonts::SYSTEMMONO.data.clone()),
            _ => None,
        }
    };
}

lazy_static! {
    static ref CONSOLE_TYPEFACE: Typeface = {
        match &*MONO_FONT_BYTES {
            Some(ref bytes) => {
                Typeface::from_data(unsafe { Data::new_bytes(bytes.as_slice()) }, None)
                    .expect("Failed to load mono font from memory")
            }
            None => Typeface::from_name("monospace", FontStyle::bold())
                .expect("Failed to load mono font"),
        }
    };
}

use super::constants::*;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path, Rect, TextBlob};
impl Console {
    pub fn draw(&mut self, v: &Editor, canvas: &mut Canvas) {
        if !self.active {
            return;
        }

        canvas.save();
        let mut matrix = Matrix::new_identity();
        matrix.set_scale((1., 1.), None);

        let font = Font::from_typeface_with_params(&*CONSOLE_TYPEFACE, 14., 1., 0.0);
        let winsize = v.viewport.winsize;
        let mut topleft = (0., winsize.1 as f32);
        let mut size = (winsize.0 as f32, 0.);

        let (_, trect) = font.measure_str("Q", None);
        topleft.1 -= CONSOLE_PADDING_Y_TOP + CONSOLE_PADDING_Y_BOTTOM;
        topleft.1 -= trect.height(); // premultiplied by font
        size.1 += CONSOLE_PADDING_Y_TOP + CONSOLE_PADDING_Y_BOTTOM;
        size.1 += trect.height(); // premultiplied by font

        // Draw background
        let console_rect = Rect::from_point_and_size(topleft, size);
        let mut paint = Paint::default();
        let mut path = Path::new();
        paint.set_style(PaintStyle::Fill);
        paint.set_color(CONSOLE_FILL);
        path.add_rect(console_rect, None);
        path.close();

        canvas.draw_path(&path, &paint);

        // Draw text
        let blob = TextBlob::from_str(&self.stdin, &font)
            .expect(&format!("Failed to shape {}", &self.stdin));

        paint.set_color(CONSOLE_TEXT_FILL);
        topleft.0 += CONSOLE_PADDING_X;
        topleft.1 += CONSOLE_PADDING_Y_BOTTOM;
        topleft.1 += trect.height(); // premultiplied by font

        canvas.draw_text_blob(&blob, topleft, &paint);

        canvas.restore();
    }
}

#[allow(unused)]
enum Return {
    OK,
    Failure(String),
}

#[allow(unused)]
struct Command {
    name: String,
    args: Vec<String>,
    run: dyn Fn() -> Return,
}

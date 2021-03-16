//! Skia renderer.

use crate::state::PreviewMode;
use crate::{CONSOLE, STATE};
use glifparser::Handle;

pub mod constants;
use self::constants::*;

pub mod console;
mod guidelines;
pub use self::guidelines::{Guideline, GuidelineType};
pub mod points; // point drawing functions
                // This imports calc_x, etc. which transforms coordinates between .glif and Skia

mod glyph;
mod selbox;
mod viewport;

// Provides thread-local global variables.
pub use crate::events::vws;
pub use crate::state::Glyph; // types
pub use crate::state::TOOL_DATA; // globals
pub use crate::state::{HandleStyle, Mode, PointLabels}; // enums

use skulpin::skia_safe::{
    gradient_shader, Canvas, Color, IRect, Matrix, Paint, PaintJoin, PaintStyle, Path, Point, Rect,
    TileMode,
};

#[derive(Clone, Copy, PartialEq)]
pub enum UIPointType {
    Point((Handle, Handle)),
    Handle,
    Anchor,
    Direction,
}

pub fn render_frame(canvas: &mut Canvas) {
    canvas.save();
    let pm = STATE.with(|v| v.borrow().preview_mode);
    canvas.clear(if pm == PreviewMode::Paper {
        PAPER_BGCOLOR
    } else {
        BACKGROUND_COLOR
    });
    // This will change the SkCanvas transformation matrix, and everything from here to
    // canvas.restore() will need to take that matrix into consideration.
    viewport::redraw_viewport(canvas);

    if pm != PreviewMode::Paper || PAPER_DRAW_GUIDELINES {
        guidelines::draw_all(canvas);
    }
    let path = glyph::draw(canvas);
    let _path = glyph::draw_previews(canvas);

    match pm {
        PreviewMode::None => {
            let mode = STATE.with(|v| v.borrow().mode);
            match mode {
                Mode::VWS => vws::draw_handles(canvas),
                _ => {}
            };
            points::draw_all(canvas);
            points::draw_selected(canvas);
        }
        PreviewMode::NoUnselectedPoints => {
            points::draw_selected(canvas);
        }
        PreviewMode::Paper => (),
    }
    match pm {
        PreviewMode::Paper => (),
        _ => {
            points::draw_directions(path, canvas);
        }
    }
    // Reset transformation matrix
    canvas.restore();

    // Draw console
    CONSOLE.with(|c| c.borrow_mut().draw(canvas));
}

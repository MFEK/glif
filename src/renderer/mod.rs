//! Skia renderer.

use crate::{events::{EditorEvent}, state::{PreviewMode, Editor}};
use crate::{CONSOLE};
use glifparser::Handle;

pub mod constants;
use self::constants::*;

pub mod console;
pub mod guidelines;
pub mod points; // point drawing functions
                // This imports calc_x, etc. which transforms coordinates between .glif and Skia
pub mod string; // drawing strings on Skia

mod glyph;
pub mod viewport;

// Provides thread-local global variables.
// TODO: pub use crate::events::vws;
pub use crate::state::{HandleStyle, PointLabels}; // enums

use skulpin::skia_safe::Canvas;

#[derive(Clone, Copy, PartialEq)]
pub enum UIPointType {
    Point((Handle, Handle)),
    Handle,
    #[allow(unused)]
    Anchor,
    Direction,
}

pub fn render_frame(v: &mut Editor, canvas: &mut Canvas) {
    canvas.save();
    let pm = v.preview_mode;
    canvas.clear(if pm == PreviewMode::Paper {
        PAPER_BGCOLOR
    } else {
        BACKGROUND_COLOR
    });
    // This will change the SkCanvas transformation matrix, and everything from here to
    // canvas.restore() will need to take that matrix into consideration.
    viewport::redraw_viewport(v, canvas);

    if pm != PreviewMode::Paper || PAPER_DRAW_GUIDELINES {
        guidelines::draw_all(v, canvas);
    }

    let path = v.with_glif(|glyph| glyph::draw(glyph, v, canvas));
    v.set_flattened(false);
    v.with_glif(|glyph| glyph::draw_components(glyph, v, canvas));
    // TODO: let _path = glyph::draw_previews(v, canvas);

    match pm {
        PreviewMode::None => {
            points::draw_all(v, canvas);
            //points::draw_selected(v, canvas);
            v.dispatch_editor_event(EditorEvent::Draw {
                skia_canvas: canvas,
            });
        }
        PreviewMode::NoUnselectedPoints => {
            //points::draw_selected(v, canvas);
        }
        PreviewMode::Paper => (),
    }
    match pm {
        PreviewMode::Paper => (),
        _ => {
            points::draw_directions(v, path, canvas);
        }
    }
    // Reset transformation matrix
    canvas.restore();

    // Draw console
    CONSOLE.with(|c| c.borrow_mut().draw(v, canvas));
}

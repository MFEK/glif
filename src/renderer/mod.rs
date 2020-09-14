//! Skia renderer.

use super::glifparser;
use crate::state::PreviewMode;
use crate::{CONSOLE, STATE};
use glifparser::{Handle, PointType, WhichHandle};

pub mod constants;
use self::constants::*;

pub mod console;
mod guidelines;
pub use self::guidelines::{Guideline, GuidelineType};
pub mod points; // point drawing functions
                // This imports calc_x, etc. which transforms coordinates between .glif and Skia
use self::points::calc::*;
mod glyph;
mod selbox;
mod viewport;

use skulpin::skia_safe::{
    gradient_shader, Canvas, Color, IRect, Matrix, Paint, PaintJoin, PaintStyle, Path, Point, Rect,
    TileMode,
};

use skulpin::winit::dpi::PhysicalPosition;
use std::cell::RefCell;
use std::cmp::min;

#[derive(Clone, Copy, PartialEq)]
pub enum HandleStyle {
    None,
    Handlebars((Handle, Handle)),
    Floating((Handle, Handle)),
}
#[derive(Clone, Copy, PartialEq)]
pub enum UIPointType {
    Point(HandleStyle),
    Handle,
    Anchor,
    Direction,
}
enum RendererPointType {
    Plain(UIPointType),
    WithPointNumber(UIPointType),
    WithPointPosition(UIPointType),
    WithPointNumberAndPosition(UIPointType),
}

pub fn render_frame(canvas: &mut Canvas) {
    let pm = STATE.with(|v|v.borrow().preview_mode);
    canvas.clear(if pm == PreviewMode::Paper { PAPER_BGCOLOR } else { BACKGROUND_COLOR });
    viewport::redraw_viewport(canvas);

    if pm != PreviewMode::Paper || PAPER_DRAW_GUIDELINES {
        guidelines::draw_all(canvas);
    }
    let path = glyph::draw(canvas);

    match pm {
        PreviewMode::None => {
            points::draw_all(canvas);
            points::draw_selected(canvas);
        }
        PreviewMode::NoUnselectedPoints => {
            points::draw_selected(canvas);
        }
        PreviewMode::Paper => ()
    }
    match pm {
        PreviewMode::Paper => (),
        _ => {
            points::draw_directions(path, canvas);
        }
    }
    CONSOLE.with(|c| c.borrow_mut().draw(canvas));
}

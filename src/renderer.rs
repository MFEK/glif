//! Skia renderer.

use super::glifparser;
use crate::state::State;
use crate::STATE;
use glifparser::{Handle, WhichHandle};

pub mod constants;
use self::constants::*;

mod guidelines;
pub mod points; // point drawing functions
                // This imports calc_x, etc. which transforms coordinates between .glif and Skia
use self::points::calc::*;
mod glyph;
mod selbox;

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

use std::thread::LocalKey;
pub fn render_frame(canvas: &mut Canvas) {
    canvas.clear(CLEAR_COLOR);
    STATE.with(|v| {
        let size = {
            let dim = canvas.image_info().dimensions();
            min(dim.width, dim.height) as i32
        };

        let center = (size / 2, size / 2);

        guidelines::draw_lbearing(canvas);
        guidelines::draw_rbearing(v.borrow().glyph.as_ref().unwrap().glif.width, canvas);
        guidelines::draw_baseline(canvas);

        let path = glyph::draw_glyph(canvas, &v);

        let mut i: isize = -1;
        for outline in v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref() {
            for contour in outline {
                for point in contour {
                    if point.b != Handle::Colocated {
                        i += 1;
                    }
                    points::draw_complete_point(point, Some(i), false, canvas);
                    if point.a != Handle::Colocated {
                        i += 1;
                    }
                    i += 1;
                }
            }
        }

        if v.borrow().show_sel_box {
            let rect = selbox::draw_selbox(canvas, &v);
            let selected = selbox::build_sel_vec_from_rect(
                rect,
                v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref(),
            );
            v.borrow_mut().selected = selected;
        }

        for point in &v.borrow().selected {
            points::draw_complete_point(point, None, true, canvas);
        }

        points::draw_directions(path, canvas);
    });
}

pub fn update_viewport(canvas: &mut Canvas) {
    let mut scale = STATE.with(|v| v.borrow().factor);
    let mut offset = STATE.with(|v| v.borrow().offset);
    STATE.with(|v| ::events::update_viewport(Some(offset), Some(scale), &v, canvas));
}

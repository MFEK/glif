//! Skia renderer.

use super::glifparser;
use super::state::state;
use super::state::State;
use glifparser::{Handle, WhichHandle};

pub mod constants;
use self::constants::*;

mod guidelines;
mod points; // point drawing functions
            // This imports calc_x, etc. which transforms coordinates between .glif and Skia
use self::points::calc::*;
mod glyph;
mod selbox;

use reclutch::skia::{
    gradient_shader, Canvas, Color, IRect, Matrix, Paint, PaintJoin, PaintStyle, Path, Point, Rect,
    TileMode,
};

use glutin::dpi::PhysicalPosition;
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
pub fn render_frame(frame: usize, fps: usize, bpm: usize, canvas: &mut Canvas) {
    canvas.clear(CLEAR_COLOR);
    state.with(|v| {
        let step = 12.0 * bpm as f32 / 60.0 / fps as f32;
        let frame_count = (360.0 / step) as usize;

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

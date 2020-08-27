use super::glifparser;
use glifparser::{Handle, WhichHandle};
use super::state::State;
use super::state::state;

mod constants;
use self::constants::*;
mod points; // point drawing functions
mod guidelines;
// This imports calc_x, etc. which transforms coordinates between .glif and Skia
use self::points::calc::*;
mod glyph;
mod selbox;

use skia_safe::{
    gradient_shader, Color, Matrix, Paint, PaintJoin, PaintStyle, Path, Point, TileMode, Rect, IRect, Canvas
};

use std::cmp::min;
use std::cell::RefCell;
use glutin::dpi::PhysicalPosition;

#[derive(PartialEq)]
enum HandleStyle { Handlebars, Floating }
#[derive(PartialEq)]
pub enum UIPointType { Point, Handle(HandleStyle), Anchor }
enum RendererPointType { Plain(UIPointType), WithPointNumber(UIPointType), WithPointPosition(UIPointType), WithPointNumberAndPosition(UIPointType) }

use std::thread::LocalKey;
pub fn render_frame(frame: usize, fps: usize, bpm: usize, canvas: &mut Canvas) {
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

        for outline in v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref() {
            for contour in outline {
                for point in contour {
                    points::draw_complete_point(point, UIPointType::Point, false, canvas);
                }
            }
        }

        if v.borrow().show_sel_box {
            let rect = selbox::draw_selbox(canvas, &v);
            let selected = selbox::build_sel_vec_from_rect(rect, v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref());
            v.borrow_mut().selected = selected;
        }

        for point in &v.borrow().selected {
            points::draw_complete_point(point, UIPointType::Point, true, canvas);
        }

        points::draw_directions(path, canvas);
    });
}

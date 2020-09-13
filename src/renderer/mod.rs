//! Skia renderer.

use super::glifparser;
use crate::state::State;
use crate::STATE;
use glifparser::{Handle, PointType, WhichHandle};

pub mod constants;
use self::constants::*;

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

use std::thread::LocalKey;
pub fn render_frame(canvas: &mut Canvas) {
    canvas.clear(CLEAR_COLOR);
    viewport::redraw_viewport(canvas);
    STATE.with(|v| {
        let size = {
            let dim = canvas.image_info().dimensions();
            min(dim.width, dim.height) as i32
        };

        let center = (size / 2, size / 2);

        guidelines::draw_lbearing(canvas);
        guidelines::draw_rbearing(v.borrow().glyph.as_ref().unwrap().glif.width, canvas);
        guidelines::draw_baseline(canvas);

        for guideline in &v.borrow().glyph.as_ref().unwrap().guidelines {
            guidelines::draw_guideline(
                Color::from(LBEARING_STROKE),
                calc_y(guideline.where_),
                GuidelineType::Horizontal,
                canvas,
            );
        }

        let path = glyph::draw_glyph(canvas, &v);

        let mut i: isize = -1;
        for outline in v.borrow().glyph.as_ref().unwrap().glif.outline.as_ref() {
            for contour in outline {
                let mut prevpoint = contour.first().unwrap();
                for point in contour {
                    points::draw_handlebars(Some(prevpoint), point, false, canvas);
                    prevpoint = &point;
                }
            }

            for contour in outline {
                let mut prevpoint = contour.first().unwrap();
                for point in contour {
                    if point.b != Handle::Colocated {
                        i += 1;
                    }
                    points::draw_complete_point(point, Some(i), false, canvas);
                    if point.a != Handle::Colocated {
                        i += 1;
                    }
                    i += 1;
                    prevpoint = &point;
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
            if point.ptype != PointType::QCurve {
                points::draw_handlebars(None, point, true, canvas);
            }
        }

        for point in &v.borrow().selected {
            points::draw_complete_point(point, None, true, canvas);
        }

        points::draw_directions(path, canvas);
    });
}

// Draw selection box
use super::constants::*;
use super::points::calc::*;
use glifparser::PointData;
use skulpin::skia_safe::dash_path_effect;
use skulpin::skia_safe::{Canvas, Contains, Paint, PaintStyle, Path, Point, Rect};
use std::cell::RefCell;

use crate::state;
use crate::state::State;

use crate::glifparser;
use crate::util::math::FlipIfRequired as _;

pub fn draw_selbox<P: PointData>(canvas: &mut Canvas, v: &RefCell<State>) -> Rect {

}

pub fn build_sel_vec_from_rect(
    mut rect: Rect,
    outline: Option<&Vec<glifparser::Contour<()>>>,
) -> Vec<glifparser::Point<Option<state::PointData>>> {
    rect.flip_if_required();

    let mut selected = Vec::new();
    for o in outline {
        for contour in o {
            for point in contour {
                if Rect::from(rect).contains(Point::from((calc_x(point.x), calc_y(point.y)))) {
                    selected.push(point.clone());
                }
            }
        }
    }
    selected
}

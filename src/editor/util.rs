use MFEKmath::{Bezier, Evaluate, Piecewise, Vector, evaluate::Primitive};
use flo_curves::bezier::solve_curve_for_t;
use glifparser::WhichHandle;
use crate::get_outline;
use crate::get_contour_len;
use crate::renderer::constants::*;
use crate::renderer::points::calc::*;
use skulpin::skia_safe::Point as SkPoint;
use skulpin::skia_safe::Rect as SkRect;
use skulpin::skia_safe::Contains;

use super::Editor;

//TODO: Move to tool utility file
#[derive(PartialEq, Clone, Copy)]
pub enum SelectPointInfo {
    Start,
    End
}
// This file is mainly utilities that are common use cases for the editor, but don't necessarily need to be
// in Editor.

/// Utility function to quickly check which point or mouse is hovering. Optional mask parameter specifies a point to ignore.
pub fn clicked_point_or_handle(v: &Editor, position: (f32, f32), mask: Option<(usize, usize)>) -> Option<(usize, usize, WhichHandle)> {
    let factor = v.viewport.factor;
    let _contour_idx = 0;
    let _point_idx = 0;

    // How we do this is quite naïve. For each click, we just iterate all points and check the
    // point and both handles. It's just a bunch of floating point comparisons in a compiled
    // language, so I'm not too concerned about it, and even in the TT2020 case doesn't seem to
    // slow anything down.
    v.with_active_layer(|layer| {
        for (contour_idx, contour) in get_outline!(layer).iter().enumerate() {
            for (point_idx, point) in contour.iter().enumerate() {
                if let Some(mask) = mask { if contour_idx == mask.0 && point_idx == mask.1 { continue }};

                let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
                // Topleft corner of point
                let point_tl = SkPoint::new(
                    calc_x(point.x as f32) - (size / 2.),
                    calc_y(point.y as f32) - (size / 2.),
                );
                let point_rect = SkRect::from_point_and_size(point_tl, (size, size));
                // Topleft corner of handle a
                let a = point.handle_or_colocated(WhichHandle::A, |f| f, |f| f);
                let a_tl = SkPoint::new(calc_x(a.0) - (size / 2.), calc_y(a.1) - (size / 2.));
                let a_rect = SkRect::from_point_and_size(a_tl, (size, size));
                // Topleft corner of handle b
                let b = point.handle_or_colocated(WhichHandle::B, |f| f, |f| f);
                let b_tl = SkPoint::new(calc_x(b.0) - (size / 2.), calc_y(b.1) - (size / 2.));
                let b_rect = SkRect::from_point_and_size(b_tl, (size, size));
    
                // winit::PhysicalPosition as an SkPoint
                let sk_mpos = SkPoint::new(position.0 as f32, position.1 as f32);
    
                if point_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::Neither));
                } else if a_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::A));
                } else if b_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::B));
                }
            }
        }
        None
    })
}

/// Checks if the active point is the active contour's start or end. Does not modify.
pub fn get_contour_start_or_end(v: &Editor, contour_idx: usize, point_idx: usize) -> Option<SelectPointInfo>
{
    let contour_len = v.with_active_layer(|layer| {get_contour_len!(layer, contour_idx)} ) - 1;
    match point_idx {
        0 => Some(SelectPointInfo::Start),
        contour_len => Some(SelectPointInfo::End),
        _ => None
    }
}

pub fn is_point_selected(v: &Editor, contour_idx: usize, point_idx: usize) -> bool
{
    if let Some(editor_pidx) = v.point_idx {
        let editor_cidx = v.contour_idx.unwrap();

        if contour_idx == editor_cidx && point_idx == editor_pidx { return true };
    }

    if v.selected.contains(&(contour_idx, point_idx)) { return true };

    return false;
}


pub struct PenPointInfo {
    pub t: f64,
    pub contour_idx: usize,
    pub seg_idx: usize,
    pub point: (f32, f32),
    pub a: (f32, f32),
    pub b: (f32, f32),
}

pub fn nearest_point_on_curve(v: &Editor, position: (f32, f32)) -> Option<PenPointInfo>
{
    v.with_active_layer(|layer| {
        let pw: Piecewise<Piecewise<Bezier>> = layer.outline.as_ref().unwrap().into();
        
        let mut distance = f64::INFINITY;
        let mut current = None;
        let mut h1 = None;
        let mut h2 = None;

        let mut t = None;
        let mut contour_idx = None;
        let mut seg_idx = None;

        for (cx, contour) in pw.segs.iter().enumerate() {
            for (bx, bezier) in contour.segs.iter().enumerate() {
                let mouse_vec = Vector::from_components(calc_x(position.0) as f64, calc_y(position.1 as f32) as f64);
                let ct = solve_curve_for_t(bezier, &mouse_vec, 3.5 / v.viewport.factor as f64);
                
                if let Some(ct) = ct {
                    let new_distance = bezier.at(ct).distance(mouse_vec);
                    if new_distance < distance {
                        distance = new_distance;
                        current = Some(bezier.at(ct));
                        t = Some(ct);
                        contour_idx = Some(cx);
                        seg_idx = Some(bx);

                        let subdivisions = bezier.subdivide(ct);
                        if let Some(subdivisions) = subdivisions {
                            h1 = Some(subdivisions.0.to_control_points()[2]);
                            h2 = Some(subdivisions.1.to_control_points()[1]);
                        }
                        else
                        {
                            return None
                        }
                    }
                }
            }
        }

        if let Some(current) = current { 
            let (h1, h2) = (h1.unwrap(), h2.unwrap());
            Some(PenPointInfo {
                t: t.unwrap(),
                contour_idx: contour_idx.unwrap(),
                seg_idx: seg_idx.unwrap(),
                point: (current.x as f32, current.y as f32),
                a: (h1.x as f32, h1.y as f32),
                b: (h2.x as f32, h2.y as f32),
            })
        } else { None }
    })
}
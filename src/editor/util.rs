use std::collections::HashSet;

use crate::get_contour_len;
use crate::{tools::prelude::math::FlipIfRequired, user_interface::Interface};
use flo_curves::{bezier::{Curve as FloCurve, solve_curve_for_t_within}, geo::Coord2};
use glifparser::{
    glif::{MFEKOutline, MFEKPointData},
    Handle, WhichHandle,
};
use glifrenderer::constants::{POINT_RADIUS, POINT_STROKE_THICKNESS};
use glifrenderer::{calc_x, calc_y};
use skulpin::skia_safe::Contains;
use skulpin::skia_safe::Point as SkPoint;
use skulpin::skia_safe::Rect as SkRect;
use MFEKmath::{Bezier, Piecewise, Primitive};

use super::Editor;

//TODO: Move to tool utility file
#[derive(PartialEq, Clone, Copy)]
pub enum SelectPointInfo {
    Start,
    End,
}
// This file is mainly utilities that are common use cases for the editor, but don't necessarily need to be
// in Editor.

/// Utility function to quickly check which point or mouse is hovering. Optional mask parameter specifies a point to ignore.
pub fn clicked_point_or_handle(
    v: &Editor,
    i: &Interface,
    position: (f32, f32),
    mask: Option<(usize, usize)>,
) -> Option<(usize, usize, WhichHandle)> {
    let factor = i.viewport.factor;
    let _contour_idx = 0;
    let _point_idx = 0;

    // How we do this is quite naïve. For each click, we just iterate all points and check the
    // point and both handles. It's just a bunch of floating point comparisons in a compiled
    // language, so I'm not too concerned about it, and even in the TT2020 case doesn't seem to
    // slow anything down.
    v.with_active_layer(|layer| {
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            for (point_idx, point) in contour.inner.iter().enumerate() {
                if let Some(mask) = mask {
                    if contour_idx == mask.0 && point_idx == mask.1 {
                        continue;
                    }
                };

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
pub fn get_contour_start_or_end(
    v: &Editor,
    contour_idx: usize,
    point_idx: usize,
) -> Option<SelectPointInfo> {
    let contour_len = v.with_active_layer(|layer| get_contour_len!(layer, contour_idx)) - 1;
    match point_idx {
        0 => Some(SelectPointInfo::Start),
        idx => {
            if idx == contour_len {
                Some(SelectPointInfo::End)
            } else {
                None
            }
        }
    }
}

pub struct HoveredPointInfo {
    pub t: f64,
    pub contour_idx: usize,
    pub seg_idx: usize,
    pub point: (f32, f32),
    pub a: (f32, f32),
    pub b: (f32, f32),
}

pub fn nearest_point_on_curve(
    v: &Editor,
    i: &Interface,
    position: (f32, f32),
) -> Option<HoveredPointInfo> {
    v.with_active_layer(|layer| {
        let pw: Piecewise<Piecewise<Bezier>> = (&layer.outline).into();

        let mut distance = f64::INFINITY;
        let mut current = None;
        let mut h1 = None;
        let mut h2 = None;

        let mut t = None;
        let mut contour_idx = None;
        let mut seg_idx = None;

        for (cx, contour) in pw.segs.iter().enumerate() {
            for (bx, mbezier) in contour.segs.iter().enumerate() {
                use flo_curves::BezierCurveFactory as _;
                let bezier = FloCurve::from_points(Coord2(mbezier.w1.x, mbezier.w1.y), (Coord2(mbezier.w2.x, mbezier.w2.y), Coord2(mbezier.w3.x, mbezier.w3.y)), Coord2(mbezier.w4.x, mbezier.w4.y));
                let mouse_vec = Coord2(
                    calc_x(position.0) as f64,
                    calc_y(position.1 as f32) as f64,
                );
                let ct = solve_curve_for_t_within(&bezier, &mouse_vec, Some(3.5 / i.viewport.factor as f64));

                if let Some(ct) = ct {
                    use flo_curves::BezierCurve as _;
                    use flo_curves::Coordinate as _;
                    let new_distance = bezier.point_at_pos(ct).distance_to(&mouse_vec);
                    if new_distance < distance {
                        distance = new_distance;
                        current = Some(bezier.point_at_pos(ct));
                        t = Some(ct);
                        contour_idx = Some(cx);
                        seg_idx = Some(bx);

                        let subdivisions = mbezier.subdivide(ct);
                        if let Some(subdivisions) = subdivisions {
                            h1 = Some(subdivisions.0.to_control_points()[2]);
                            h2 = Some(subdivisions.1.to_control_points()[1]);
                        } else {
                            return None;
                        }
                    }
                }
            }
        }

        if let Some(current) = current {
            let (h1, h2) = (h1.unwrap(), h2.unwrap());
            Some(HoveredPointInfo {
                t: t.unwrap(),
                contour_idx: contour_idx.unwrap(),
                seg_idx: seg_idx.unwrap(),
                point: (current.0 as f32, current.1 as f32),
                a: (h1.x as f32, h1.y as f32),
                b: (h2.x as f32, h2.y as f32),
            })
        } else {
            None
        }
    })
}

pub fn build_box_selection(
    mut rect: SkRect,
    outline: &MFEKOutline<MFEKPointData>,
) -> HashSet<(usize, usize)> {
    rect.flip_if_required();

    let mut selected = HashSet::new();
    for (cidx, contour) in outline.iter().enumerate() {
        for (pidx, point) in contour.inner.iter().enumerate() {
            if rect.contains(SkPoint::from((calc_x(point.x), calc_y(point.y)))) {
                selected.insert((cidx, pidx));
            }
        }
    }

    selected
}

pub fn move_point(outline: &mut MFEKOutline<MFEKPointData>, ci: usize, pi: usize, x: f32, y: f32) {
    let (cx, cy) = (outline[ci].inner[pi].x, outline[ci].inner[pi].y);
    let (dx, dy) = (cx - x, cy - y);

    outline[ci].inner[pi].x = x;
    outline[ci].inner[pi].y = y;

    let a = outline[ci].inner[pi].a;
    let b = outline[ci].inner[pi].b;
    match a {
        Handle::At(hx, hy) => outline[ci].inner[pi].a = Handle::At(hx - dx, hy - dy),
        Handle::Colocated => (),
    }
    match b {
        Handle::At(hx, hy) => outline[ci].inner[pi].b = Handle::At(hx - dx, hy - dy),
        Handle::Colocated => (),
    }
}

pub fn move_point_without_handles(
    outline: &mut MFEKOutline<MFEKPointData>,
    ci: usize,
    pi: usize,
    x: f32,
    y: f32,
) {
    outline[ci].inner[pi].x = x;
    outline[ci].inner[pi].y = y;
}

pub fn get_handle_pos(
    outline: &mut MFEKOutline<MFEKPointData>,
    ci: usize,
    pi: usize,
    handle: WhichHandle,
) -> Option<(f32, f32)> {
    let ret_handle = match handle {
        WhichHandle::Neither => panic!("This function should be called with a proper handle!"),
        WhichHandle::A => outline[ci].inner[pi].a,
        WhichHandle::B => outline[ci].inner[pi].b,
    };

    match ret_handle {
        Handle::At(hx, hy) => {
            return Some((hx, hy));
        }
        Handle::Colocated => None,
    }
}

pub fn move_handle(
    outline: &mut MFEKOutline<MFEKPointData>,
    ci: usize,
    pi: usize,
    handle: WhichHandle,
    x: f32,
    y: f32,
) {
    match handle {
        WhichHandle::Neither => panic!("This function should be called with a proper handle!"),
        WhichHandle::A => match outline[ci].inner[pi].a {
            Handle::At(_hx, _hy) => outline[ci].inner[pi].a = Handle::At(x, y),
            Handle::Colocated => (),
        },
        WhichHandle::B => match outline[ci].inner[pi].b {
            Handle::At(_hx, _hy) => outline[ci].inner[pi].b = Handle::At(x, y),
            Handle::Colocated => (),
        },
    };
}

// This file is mainly utilities that are common use cases for the editor, but don't necessarily need to be
// in Editor.

use crate::{get_contour_len, get_point_mut};
use crate::user_interface::Interface;
use flo_curves::{
    bezier::{solve_curve_for_t_along_axis, Curve as FloCurve},
    geo::Coord2,
};
use glifparser::{WhichHandle};
use glifrenderer::constants::{POINT_RADIUS, POINT_STROKE_THICKNESS};
use skulpin::skia_safe::Contains;
use skulpin::skia_safe::Point as SkPoint;
use skulpin::skia_safe::Rect as SkRect;
use MFEKmath::{Bezier, Piecewise, Primitive as MathPrimitive};

use super::Editor;
use glifparser::glif::mfek::contour::MFEKContourCommon;

//TODO: Move to tool utility file
#[derive(PartialEq, Clone, Copy)]
pub enum SelectPointInfo {
    Start,
    End,
}

/*
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct PointKey {
    /// contour index
    pub ci: u32,
    /// point index
    pub pi: u32,
    /// which handle? if unset you get closest by distance
    pub wh: Option<glifparser::WhichHandle>,
}

#[derive(Clone, Copy)]
pub struct ClickedPoint<'hitomi, 'a, 'cute> {
    pub key: PointKey,
    v:       &'hitomi Editor,
    i:       &'a      Interface,
    ignored: &'cute   HashSet<PointKey>,
}

impl ClickedPoint<'_, '_, '_> {
    pub fn solve<T: glifparser::PointData>(
        position: (f32, f32),
        mask: Option<(usize, usize)>,
    // TODO: Error type for the different reasons points may not be selected.
    ) -> Result<glifparser::Point<T>, ()> {
        // contour idx, point idx, which handle enum w/Neither variant
        //let (ci, pi, wh) = clicked_point_or_handle(self.v, self.i, position, mask);
        Err(())
    }
}*/

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
    for (contour_idx, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
        for (point_idx, point) in contour.inner.iter().enumerate() {
            if let Some(mask) = mask {
                if contour_idx == mask.0 && point_idx == mask.1 {
                    continue;
                }
            };

            let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
            // Topleft corner of point
            let point_tl = SkPoint::new(point.x() as f32 - (size / 2.), point.y() as f32 - (size / 2.));
            let point_rect = SkRect::from_point_and_size(point_tl, (size, size));

            // winit::PhysicalPosition as an SkPoint
            let sk_mpos = SkPoint::new(position.0 as f32, position.1 as f32);

            if point_rect.contains(sk_mpos) {
                return Some((contour_idx, point_idx, WhichHandle::Neither));
            }

            if let Some(handle_pos) = point.get_handle_position(WhichHandle::A) {
                let a_tl = SkPoint::new(handle_pos.0 - (size / 2.), handle_pos.1 - (size / 2.));
                let a_rect = SkRect::from_point_and_size(a_tl, (size, size));
                
                if a_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::A))
                }
            }

            if let Some(handle_pos) = point.get_handle_position(WhichHandle::B) {
                let b_tl = SkPoint::new(handle_pos.0 - (size / 2.), handle_pos.1 - (size / 2.));
                let b_rect = SkRect::from_point_and_size(b_tl, (size, size));

                if b_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::B))
                }
            }
        }
    }
    None
}

/// Checks if the active point is the active contour's start or end. Does not modify.
pub fn get_contour_start_or_end(
    v: &Editor,
    contour_idx: usize,
    point_idx: usize,
) -> Option<SelectPointInfo> {
    let layer = v.get_active_layer_ref();
    let contour_len = get_contour_len!(layer, contour_idx) - 1;

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
    {
        let mut distance = f64::INFINITY;
        let mut current = None;
        let mut h1 = None;
        let mut h2 = None;

        let mut t = None;
        let mut contour_idx = None;
        let mut seg_idx = None;

        for (cx, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
            if let Some(cubic_contour) = contour.cubic() {
                let pw: Piecewise<Bezier> = Piecewise::from(cubic_contour);
                for (bx, mbezier) in pw.segs.iter().enumerate() {
                    use flo_curves::BezierCurveFactory as _;
                    let bezier = FloCurve::from_points(
                        Coord2(mbezier.w1.x, mbezier.w1.y),
                        (
                            Coord2(mbezier.w2.x, mbezier.w2.y),
                            Coord2(mbezier.w3.x, mbezier.w3.y),
                        ),
                        Coord2(mbezier.w4.x, mbezier.w4.y),
                    );
                    let mouse_vec = Coord2(position.0 as f64, position.1 as f32 as f64);
                    let ct = solve_curve_for_t_along_axis(
                        &bezier,
                        &mouse_vec,
                        3.5 / i.viewport.factor as f64,
                    );
    
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
    
                            let subdivisions = MathPrimitive::subdivide(mbezier, ct);
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
    }
}

pub fn move_all_layers(v: &mut Editor, mut x: f32, mut y: f32) {
    v.with_glyph_mut(|glyph| {
        for li in 0..glyph.layers.len() {
            for ci in 0..glyph.layers[li].outline.len() {
                for pi in 0..glyph.layers[li].outline[ci].inner.len() {
                    let point = get_point_mut!(glyph.layers[li], ci, pi).unwrap();
                    let (cx, cy) = point.get_position();

                    if x.is_nan() {
                        x = 0.;
                    }
                    if y.is_nan() {
                        y = 0.;
                    }

                    let (x, y) = (cx - x, cy - y);
                    point.set_position(x, y)
                }
            }
        }
    });
}

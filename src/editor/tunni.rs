use flo_curves::Line;
use glifparser::{glif::contour::MFEKContourCommon, MFEKPointData, Point};
use MFEKmath::{vec2, Vector};

use crate::{get_point, user_interface::Interface};

use super::Editor;

// This struct holds the indices of the tunni line in the form of (ci, pi)
#[derive(Clone, Debug)]
pub struct TunniLineInfo {
    pub a: (usize, usize),
    pub b: (usize, usize),
}

pub fn get_tunni_point(
    first: &Point<MFEKPointData>,
    second: &Point<MFEKPointData>,
) -> Option<Vector> {
    let a = first.a;
    let b = second.b;

    // both of our handles need to be located
    match (a, b) {
        (glifparser::Handle::At(ax, ay), glifparser::Handle::At(bx, by)) => {
            // Construct a line between the handles
            let av = vec2![ax, ay];
            let bv = vec2![bx, by];

            let fv = vec2![first.x, first.y];
            let sv = vec2![second.x, second.y];

            let pint = flo_curves::line::ray_intersects_ray(&(fv, av), &(sv, bv));
            if let Some(intersection) = pint {
                return Some(((av * 2. - fv) + (bv * 2. - sv)) - intersection);
            }
        }
        _ => {}
    }

    None
}

pub fn construct_tunni_line(
    first: &Point<MFEKPointData>,
    second: &Point<MFEKPointData>,
) -> Option<(Vector, Vector)> {
    let a = first.a;
    let b = second.b;

    if get_tunni_point(first, second).is_none() {
        return None;
    }

    // both of our handles need to be located
    match (a, b) {
        (glifparser::Handle::At(ax, ay), glifparser::Handle::At(bx, by)) => {
            // Construct a line between the handles
            let av = vec2![ax, ay];
            let bv = vec2![bx, by];

            let fv = vec2![first.x, first.y];
            let sv = vec2![second.x, second.y];

            // Calculate vectors from points to handles
            let va = av - fv;
            let vb = bv - sv;

            // Calculate the line vector and the normal of the line
            let lv = sv - fv;

            // Calculate cross products
            let cross_product_a = va.cross(lv);
            let cross_product_b = vb.cross(lv);

            // If the signs of the cross products are the same, handles lie on the same side of the line
            if (cross_product_a >= 0.0 && cross_product_b >= 0.0)
                || (cross_product_a < 0.0 && cross_product_b < 0.0)
            {
                return Some((av, bv));
            }
        }
        _ => {}
    }

    None
}

pub fn get_tunni_line_from_info(v: &Editor, info: &TunniLineInfo) -> Option<(Vector, Vector)> {
    // we need to get both of the ci, pi pairs in tunnilineinfo
    let a = get_point!(v.get_active_layer_ref(), info.a.0, info.a.1);
    let b = get_point!(v.get_active_layer_ref(), info.b.0, info.b.1);

    return construct_tunni_line(a.unwrap().cubic().unwrap(), b.unwrap().cubic().unwrap());
}

pub fn get_tunni_point_from_info(v: &Editor, info: &TunniLineInfo) -> Option<Vector> {
    // we need to get both of the ci, pi pairs in tunnilineinfo
    let a = get_point!(v.get_active_layer_ref(), info.a.0, info.a.1);
    let b = get_point!(v.get_active_layer_ref(), info.b.0, info.b.1);

    return get_tunni_point(a.unwrap().cubic().unwrap(), b.unwrap().cubic().unwrap());
}

fn get_distance_from_tunni_line(pos: Vector, line: (Vector, Vector)) -> f64 {
    let flo_line = <(Vector, Vector) as Line>::from_points(line.0, line.1);

    let pos_to_line = flo_line.pos_for_point(&pos);
    let clamped_t = pos_to_line.clamp(0., 1.);
    let clamped_position = flo_line.point_at_pos(clamped_t);

    return clamped_position.distance(pos);
}

pub fn get_closest_tunni_line(v: &Editor, i: &Interface) -> Option<TunniLineInfo> {
    let mut closest_distance = f64::INFINITY;
    let mut closest_tunni: Option<TunniLineInfo> = None;

    let mp = vec2![i.mouse_info.position.0, i.mouse_info.position.1];

    for (contour_idx, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
        if let Some(cubic) = contour.cubic() {
            for (point_idx, pair) in cubic.windows(2).enumerate() {
                if let Some(line) = construct_tunni_line(&pair[0], &pair[1]) {
                    let tunni_point = get_tunni_point(&pair[0], &pair[1]).unwrap();
                    let point_distance = tunni_point.distance(mp);
                    let distance = get_distance_from_tunni_line(mp, line);

                    if distance < closest_distance || point_distance < closest_distance {
                        closest_distance = f64::min(closest_distance, distance);
                        closest_tunni = Some(TunniLineInfo {
                            a: (contour_idx, point_idx),
                            b: (contour_idx, point_idx + 1),
                        });
                    }
                }
            }

            if cubic.is_closed() && cubic.len() > 1 {
                if let Some(line) =
                    construct_tunni_line(&cubic.last().unwrap(), &cubic.first().unwrap())
                {
                    let distance = get_distance_from_tunni_line(mp, line);

                    let point_idx = cubic.len() - 1;
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_tunni = Some(TunniLineInfo {
                            a: (contour_idx, point_idx),
                            b: (contour_idx, 0),
                        });
                    }
                }
            }
        }
    }

    return closest_tunni;
}

pub enum Tunni {
    Point,
    Line,
}

const MIN_DISTANCE_FOR_CLICK: f64 = 5.;

pub fn clicked_tunni_point_or_line(v: &Editor, i: &Interface) -> Option<(TunniLineInfo, Tunni)> {
    let closest_point_or_line = get_closest_tunni_line(v, i);

    if let Some(tunni_info) = closest_point_or_line {
        let tunni_point = get_tunni_point_from_info(v, &tunni_info)
            .expect("get_closest_tunni should always return a valid tunni line.");
        let tunni_line = get_tunni_line_from_info(v, &tunni_info)
            .expect("get_closest_tunni should always return a valid tunni line.");

        let point_distance = Vector::distance(tunni_point, i.mouse_info.position.into());
        let line_distance = get_distance_from_tunni_line(i.mouse_info.position.into(), tunni_line);

        let closest = f64::min(point_distance, line_distance);

        println!("FOUND CLOSEST {0}", closest);
        if closest < MIN_DISTANCE_FOR_CLICK * (1. / i.viewport.factor) as f64 {
            if closest == point_distance {
                return Some((tunni_info, Tunni::Point));
            } else {
                return Some((tunni_info, Tunni::Line));
            }
        }
    }

    None
}

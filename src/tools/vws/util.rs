use super::super::prelude::*;
use glifparser::{
    glif::{ContourOperations, InterpolationType, VWSHandle},
    CapType, JoinType, VWSContour, WhichHandle,
};
use MFEKmath::{Evaluate, Piecewise, Vector};

// This file holds utility functions for working with vws. Things like if a handle is clicked.

pub fn get_vws_contour(v: &Editor, contour_idx: usize) -> Option<VWSContour> {
    if let Some(contour_op) = v.get_active_layer_ref().outline[contour_idx]
        .operation
        .clone()
    {
        return if let ContourOperations::VariableWidthStroke { data } = contour_op {
            Some(data)
        } else {
            None
        };
    }

    None
}

pub fn set_vws_contour(v: &mut Editor, contour_idx: usize, contour: VWSContour) {
    v.get_active_layer_mut().outline[contour_idx].operation =
        Some(ContourOperations::VariableWidthStroke {
            data: contour.clone(),
        });
}

pub fn mouse_coords_to_handle_space(
    v: &Editor,
    meta: MouseInfo,
    wh: WhichHandle,
) -> Result<(f64, f64), ()> {
    let (start_pos, tangent, _handle_pos) =
        get_vws_handle_pos(v, v.contour_idx.unwrap(), v.point_idx.unwrap(), wh)?;

    let side_multiplier = match wh {
        WhichHandle::A => -1.,
        WhichHandle::B => 1.,
        _ => unreachable!(),
    };

    let tangent = tangent.normalize();
    let normal = Vector::from_components(tangent.y, -tangent.x).normalize();

    let mouse_vec =
        start_pos - Vector::from_components(meta.position.0 as f64, meta.position.1 as f64);
    let mouse_vec_normal = mouse_vec.normalize();

    //return mouse_vec_normal.dot(handle_vec) * mouse_vec.magnitude();
    let normal_offset = f64::max(
        mouse_vec_normal.dot(normal) * mouse_vec.magnitude() * side_multiplier,
        0.,
    );
    let tangent_offset = mouse_vec_normal.dot(tangent) * mouse_vec.magnitude();

    Ok((normal_offset, tangent_offset))
}

pub fn set_vws_handle(
    v: &mut Editor,
    side: WhichHandle,
    mirror: bool,
    constrain: bool,
    normal_offset: f64,
    tangent_offset: f64,
) {
    let contour_idx = v.contour_idx.unwrap();
    let point_idx = v.point_idx.unwrap();

    let contour_op = get_vws_contour(v, contour_idx);
    let mut vws_contour = if let Some(op) = contour_op {
        op
    } else {
        generate_vws_contour(v, contour_idx)
    };

    let contour_pw = Piecewise::from(&get_contour!(v.get_active_layer_ref(), contour_idx));

    let side_multiplier = match side {
        WhichHandle::A => 1.,
        WhichHandle::B => -1.,
        _ => unreachable!(),
    };

    let tangent_offset = if constrain {
        0.
    } else {
        side_multiplier * tangent_offset
    };

    // if we're editing the first point we need to mirror it in the 'imaginary' last point
    if point_idx == 0 && contour_pw.is_closed() {
        let last_handle = vws_contour.handles.len() - 1;

        match side {
            WhichHandle::A => vws_contour.handles[last_handle].left_offset = normal_offset,
            WhichHandle::B => vws_contour.handles[last_handle].right_offset = normal_offset,
            _ => {}
        }

        vws_contour.handles[last_handle].tangent_offset = tangent_offset;
        if mirror {
            vws_contour.handles[last_handle].left_offset = normal_offset;
            vws_contour.handles[last_handle].right_offset = normal_offset;
        }
    }

    match side {
        WhichHandle::A => {
            vws_contour.handles[point_idx].left_offset = normal_offset;
        }
        WhichHandle::B => {
            vws_contour.handles[point_idx].right_offset = normal_offset;
        }
        _ => {}
    }

    if mirror {
        vws_contour.handles[point_idx].left_offset = normal_offset;
        vws_contour.handles[point_idx].right_offset = normal_offset;
        vws_contour.handles[point_idx].tangent_offset = tangent_offset;
    } else {
        vws_contour.handles[point_idx].tangent_offset = tangent_offset;
    }

    set_vws_contour(v, contour_idx, vws_contour);
}

pub fn set_all_vws_handles(v: &mut Editor, handle: WhichHandle, mirror: bool, normal_offset: f64) {
    let contour_idx = v.contour_idx.unwrap();
    let mut vws_contour =
        get_vws_contour(v, contour_idx).unwrap_or_else(|| generate_vws_contour(v, contour_idx));

    for handle_idx in 0..vws_contour.handles.len() {
        if mirror {
            vws_contour.handles[handle_idx].left_offset = normal_offset;
            vws_contour.handles[handle_idx].right_offset = normal_offset;
        } else {
            match handle {
                WhichHandle::A => vws_contour.handles[handle_idx].left_offset = normal_offset,
                WhichHandle::B => vws_contour.handles[handle_idx].right_offset = normal_offset,
                _ => {} // should be unreachable
            }
        }
    }

    set_vws_contour(v, contour_idx, vws_contour);
}

pub fn get_vws_handle_pos(
    v: &Editor,
    contour_idx: usize,
    handle_idx: usize,
    side: WhichHandle,
) -> Result<(Vector, Vector, Vector), ()> {
    let vws_contour =
        get_vws_contour(v, contour_idx).unwrap_or_else(|| generate_vws_contour(v, contour_idx));
    let contour_pw = Piecewise::from(&get_contour!(v.get_active_layer_ref(), contour_idx));

    if handle_idx > vws_contour.handles.len() - 1 {
        log::warn!(
            "Failed to get requested VWS len position ({}/{}).",
            handle_idx,
            vws_contour.handles.len() - 1
        );
        return Err(());
    }
    let vws_handle = vws_contour.handles[handle_idx];

    // if we've got an open contour and are dealing with the last handle we need special logic
    let (_bezier, start_point, tangent, normal) = if handle_idx == contour_pw.segs.len() {
        let bezier = &contour_pw.segs[handle_idx - 1];
        let start_point = bezier.end_point();
        let tangent = bezier.tangent_at(1.).normalize();
        let normal = Vector {
            x: tangent.y,
            y: -tangent.x,
        }
        .normalize();

        (bezier, start_point, tangent, normal)
    } else {
        let bezier = &contour_pw.segs[handle_idx];
        let start_point = bezier.start_point();
        let tangent = bezier.tangent_at(0.).normalize();
        let normal = Vector {
            x: tangent.y,
            y: -tangent.x,
        }
        .normalize();

        (bezier, start_point, tangent, normal)
    };

    let max_tangent = f64::max(vws_handle.right_offset, vws_handle.left_offset);

    let scaled_tangent_offset = match side {
        WhichHandle::A => vws_handle.left_offset / max_tangent,
        WhichHandle::B => vws_handle.right_offset / max_tangent,
        WhichHandle::Neither => panic!("Should be unreachable!"),
    };

    Ok(match side {
        WhichHandle::A => (
            start_point,
            tangent,
            start_point
                + normal * vws_handle.left_offset
                + tangent * -vws_handle.tangent_offset * scaled_tangent_offset,
        ),
        WhichHandle::B => (
            start_point,
            tangent,
            start_point
                + normal * -vws_handle.right_offset
                + tangent * vws_handle.tangent_offset * scaled_tangent_offset,
        ),
        _ => unreachable!(),
    })
}

fn generate_vws_contour(v: &Editor, contour_idx: usize) -> VWSContour {
    let mut new_vws_contour = VWSContour {
        handles: Vec::new(),
        cap_start_type: CapType::Round,
        cap_end_type: CapType::Round,
        join_type: JoinType::Round,
        remove_internal: false,
        remove_external: false,
    };

    for _i in 0..get_contour_len!(v.get_active_layer_ref(), contour_idx) + 1 {
        new_vws_contour.handles.push(VWSHandle {
            left_offset: 10.,
            right_offset: 10.,
            interpolation: InterpolationType::Linear,
            tangent_offset: 0.,
        })
    }

    new_vws_contour
}

pub fn clicked_handle(
    v: &Editor,
    i: &Interface,
    meta: MouseInfo,
) -> Option<(usize, usize, WhichHandle)> {
    let factor = i.viewport.factor;
    let mouse_pos = meta.position;

    for (contour_idx, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
        let contour_pw = Piecewise::from(contour);

        let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
        for vws_handle_idx in 0..contour_pw.segs.len() {
            let (handle_pos_left, handle_pos_right) = match (
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A),
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B),
            ) {
                (Ok(l), Ok(r)) => (l.2, r.2),
                (_e_l, _e_r) => return None,
            };

            let handle_left_point = SkPoint::new(
                handle_pos_left.x as f32 - (size / 2.),
                handle_pos_left.y as f32 - (size / 2.),
            );
            let handle_left_rect = SkRect::from_point_and_size(handle_left_point, (size, size));

            let handle_right_point = SkPoint::new(
                handle_pos_right.x as f32 - (size / 2.),
                handle_pos_right.y as f32 - (size / 2.),
            );
            let handle_right_rect = SkRect::from_point_and_size(handle_right_point, (size, size));

            let sk_mpos = SkPoint::new(mouse_pos.0 as f32, mouse_pos.1 as f32);

            if handle_left_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::A));
            } else if handle_right_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::B));
            }
        }

        if contour.inner.first().unwrap().ptype == glifparser::PointType::Move {
            let vws_handle_idx = contour_pw.segs.len();

            let (handle_pos_left, handle_pos_right) = match (
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A),
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B),
            ) {
                (Ok(l), Ok(r)) => (l.2, r.2),
                (_e_l, _e_r) => return None,
            };

            let handle_left_point = SkPoint::new(
                handle_pos_left.x as f32 - (size / 2.),
                handle_pos_left.y as f32 - (size / 2.),
            );
            let handle_left_rect = SkRect::from_point_and_size(handle_left_point, (size, size));

            let handle_right_point = SkPoint::new(
                handle_pos_right.x as f32 - (size / 2.),
                handle_pos_right.y as f32 - (size / 2.),
            );
            let handle_right_rect = SkRect::from_point_and_size(handle_right_point, (size, size));

            let sk_mpos = SkPoint::new(meta.position.0 as f32, meta.position.1 as f32);

            if handle_left_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::A));
            } else if handle_right_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::B));
            }
        }
    }

    None
}

use MFEKmath::{Bezier, Primitive};
use glifparser::{glif::{contour::MFEKContourCommon, contour_operations::ContourOperation}, Point, PointType, WhichHandle, contour::State, Contour, MFEKPointData};
use glifrenderer::points::draw_point;

use crate::{editor::{Editor, util::{HoveredPointInfo}}, user_interface::MouseInfo, get_contour_len, tool_behaviors::move_handle::MoveHandle};
use super::PenMode;

/// This is the cubic specific pen implementation, and should serve as an example of the new pen API.
/// One of the biggest things of note is that these mode structs should -not- contain modification specific state.
/// State like that should be handled purely inside ToolBehaviors implemented for the mode. You can have state in these
/// top level modes like UI state where a user might select a point type e.g. Spiro, but these modes are instantiated with
/// Pen so the state will carry across modifications, and mode changes.
#[derive(Clone, Debug)]
pub struct CubicMode {
    // cubic requires no state unlike a mode like Spiro
}

impl PenMode for CubicMode {
    fn new_contour(&self, v: &mut Editor, mouse_info: MouseInfo) {
        let mouse_pos = mouse_info.position;
        v.contour_idx = {
            let layer = v.get_active_layer_mut();
            let mut new_contour: Contour<MFEKPointData> = Vec::new();
            new_contour.push(Point::from_x_y_type(
                (mouse_pos.0 as f32, mouse_pos.1 as f32),
                if mouse_info.modifiers.shift {
                    PointType::Curve
                } else {
                    PointType::Move
                },
            ));

            layer.outline.push(new_contour.into());
            Some(layer.outline.len() - 1)
        };
        v.point_idx = Some(0);
    }
    
    fn add_point(&self, v: &mut Editor, mouse_info: MouseInfo) {
        let mouse_pos = mouse_info.position;
        let contour_idx = v.contour_idx.unwrap();
        let contour_len = get_contour_len!(v.get_active_layer_ref(), contour_idx);

        if v.point_idx.unwrap() == contour_len - 1 {
            v.point_idx = {
                let layer = v.get_active_layer_mut();
                layer.outline[contour_idx].operation.insert_op(contour_len);
                let contour = layer.outline[contour_idx].cubic_mut().unwrap();
                contour.push(Point::from_x_y_type(
                    (mouse_pos.0 as f32, mouse_pos.1 as f32),
                    PointType::Curve,
                ));

                Some(get_contour_len!(layer, contour_idx) - 1)
            };
        } else if v.point_idx.unwrap() == 0 {
            {
                let layer = v.get_active_layer_mut();
                let contour = layer.outline[contour_idx].cubic_mut().unwrap();
                let point_type = contour[0].ptype;

                if contour.is_open() {
                    contour[0].ptype = PointType::Curve;
                }

                contour.insert(
                    0,
                    Point::from_x_y_type((mouse_pos.0 as f32, mouse_pos.1 as f32), point_type),
                );

                layer.outline[contour_idx].operation.insert_op( 0);
            };
            v.end_modification();
        }
        v.push_behavior(Box::new(MoveHandle::new(WhichHandle::A, mouse_info, true)));
    }

    fn draw_nearest_point(&self, i: &crate::user_interface::Interface, canvas: &mut MFEKmath::skia_safe::Canvas, info: HoveredPointInfo) {
        draw_point::<()>(
            &i.viewport,
            &Point::from_x_y_type(info.point, PointType::Curve),
            None,
            true,
            canvas
        )
    }
    
    fn subdivide_curve(&self, v: &mut Editor, info: HoveredPointInfo) {
        let mut second_idx_zero = false;
        let layer = v.get_active_layer_mut();
        layer.outline[info.contour_idx].operation.insert_op( info.seg_idx);
        let contour = layer.outline[info.contour_idx].cubic_mut().unwrap();
        let mut point = contour.remove(info.seg_idx);
        let mut next_point = if info.seg_idx == contour.len() {
            second_idx_zero = true;
            contour.remove(0)
        } else {
            contour.remove(info.seg_idx)
        };

        let bez = Bezier::from(&point, &next_point);
        let subdivisions = bez.subdivide(info.t);

        if let Some(subdivisions) = subdivisions {
            let (sub_a, sub_b) = (
                subdivisions.0.to_control_points(),
                subdivisions.1.to_control_points(),
            );
            point.a = sub_a[1].to_handle();
            next_point.b = sub_b[2].to_handle();

            if second_idx_zero {
                contour.insert(0, next_point);
            } else {
                contour.insert(info.seg_idx, next_point);
            }

            let (x, y) = (sub_a[3].x, sub_a[3].y);
            contour.insert(
                info.seg_idx,
                Point::from_x_y_a_b_type(
                    (x as f32, y as f32),
                    (sub_b[1].to_handle(), sub_a[2].to_handle()),
                    PointType::Curve,
                ),
            );

            contour.insert(info.seg_idx, point);
        }
    }
}
use glifparser::{glif::{point::hyper::{HyperPoint, HyperPointType}, inner::{hyper::MFEKHyperInner, MFEKContourInner}, MFEKContour, contour_operations::ContourOperation, contour::MFEKContourCommon}, MFEKPointData, WhichHandle};

use crate::{editor::{Editor, util::HoveredPointInfo}, user_interface::MouseInfo, get_contour_len, tool_behaviors::move_handle::MoveHandle};

use super::PenMode;

#[derive(Clone, Debug)]
pub struct HyperMode {
    // cubic requires no state unlike a mode like Spiro
}

impl PenMode for HyperMode {
    fn new_contour(&self, v: &mut Editor, mouse_info: MouseInfo) {
        let mouse_pos = mouse_info.position;
        v.contour_idx = {
            let layer = v.get_active_layer_mut();
            let mut new_points: Vec<HyperPoint<MFEKPointData>> = Vec::new();
            new_points.push(HyperPoint::new(mouse_pos.0, mouse_pos.1, HyperPointType::Curve, true));

            let new_contour = MFEKHyperInner::new(new_points, true);
            let new_outer:MFEKContour<MFEKPointData> = MFEKContour::new(
                MFEKContourInner::Hyper(new_contour),
                None,
            );

            layer.outline.push(new_outer);
            Some(layer.outline.len() - 1)
        };
        v.point_idx = Some(0);
    }
    
    fn add_point(&self, v: &mut Editor, mouse_info: MouseInfo) {
        let mouse_pos = mouse_info.position;
        let contour_idx = v.contour_idx.unwrap();
        let contour_len = get_contour_len!(v.get_active_layer_ref(), contour_idx);
        
        let smooth = !mouse_info.modifiers.alt;

        if v.point_idx.unwrap() == contour_len - 1 {
            v.point_idx = {
                let layer = v.get_active_layer_mut();
                layer.outline[contour_idx].operation_mut().insert_op(contour_len);
                let contour = layer.outline[contour_idx].hyper_mut().unwrap();
                contour.get_points_mut().push(HyperPoint::new(
                    mouse_pos.0 as f32, 
                    mouse_pos.1 as f32,
                    HyperPointType::Curve,
                    smooth
                ));

                Some(get_contour_len!(layer, contour_idx) - 1)
            };
        } else if v.point_idx.unwrap() == 0 {
            {
                let layer = v.get_active_layer_mut();
                let contour = layer.outline[contour_idx].hyper_mut().unwrap();

                contour.get_points_mut().insert(
                    0,
                    HyperPoint::new(mouse_pos.0 as f32, mouse_pos.1 as f32, HyperPointType::Curve, true),
                );

                layer.outline[contour_idx].operation_mut().insert_op( 0);
            };
            v.end_modification();
        }
        v.push_behavior(Box::new(MoveHandle::new(WhichHandle::A, mouse_info, true)));
    }

    fn draw_nearest_point(&self, _i: &crate::user_interface::Interface, _canvas: &mut MFEKmath::skia_safe::Canvas, _info: HoveredPointInfo) {
    }
    
    fn subdivide_curve(&self, _v: &mut Editor, _info: HoveredPointInfo) {
    }
}
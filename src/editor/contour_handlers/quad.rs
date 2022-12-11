use glifparser::glif::{contour_operations::ContourOperation, contour::MFEKContourCommon};

use crate::editor::Editor;

use super::ContourHandler;

#[derive(Debug)]
pub struct CubicHandler;

impl ContourHandler for QuadHandler {
    fn merge_contours(&self, v: &mut Editor, start_contour: usize, end_contour: usize) {
        let (cidx, pidx) = {
            let layer = v.get_active_layer_mut();


            let start_clone = layer.outline[start_contour].clone();
            layer.outline[end_contour].operation.append(&start_clone);
            // start is the contour who's start is being merged and end is the contour who's end is being merged
            let mut start = layer.outline[start_contour].cubic_mut().unwrap().clone();
            let end = layer.outline[end_contour].cubic_mut().unwrap();

            let end_len = end.len();
            end[end_len-1].b = start[0].a;

            let p_idx = end.len() - 1;
            start.remove(0);
            for point in start.iter() {
                end.push(point.clone());
            }

            layer.outline.remove(start_contour);

            let mut selected = end_contour;
            if end_contour > start_contour {
                selected = end_contour - 1
            }

            (selected, p_idx)
        };

        v.contour_idx = Some(cidx);
        v.point_idx = Some(pidx);
    }
}
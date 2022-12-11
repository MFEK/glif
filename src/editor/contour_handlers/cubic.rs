use glifparser::{glif::{contour_operations::ContourOperation, contour::{MFEKContourCommon, MFEKCommonOuter}}, WhichHandle};

use crate::editor::Editor;

use super::ContourHandler;

#[derive(Debug)]
pub struct CubicHandler;

impl ContourHandler for CubicHandler {
    fn merge_contours(&self, v: &mut Editor, start_contour: usize, end_contour: usize) {
        let (cidx, pidx) = {
            let layer = v.get_active_layer_mut();

            // start is the contour who's start is being merged and end is the contour who's end is being merged
            let mut start = layer.outline[start_contour].clone();
            let end = &mut layer.outline[end_contour];

            let p_idx = end.len() - 1;
            end.delete(0);
            end.append(&mut start);

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
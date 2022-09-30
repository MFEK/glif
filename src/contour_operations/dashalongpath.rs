use glifparser::glif::{DashContour, Glif, MFEKContour, MFEKOutline};

use super::ContourOperationData;
use crate::util::MFEKGlifPointData;

impl ContourOperationData for DashContour {
    fn build(&self, contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData> {
        let mut glif = Glif::default();
        glif.outline = Some(vec![contour.inner.clone()]);
        let dash_output = MFEKmath::dash_along_glif(&glif, self);

        let mut output: MFEKOutline<MFEKGlifPointData> = Vec::new();
        if let Some(outline) = dash_output.outline {
            for contour in outline {
                output.push(contour.into());
            }
        }

        output
    }

    fn sub(&mut self, _contour: &MFEKContour<MFEKGlifPointData>, _begin: usize, _end: usize) {}

    fn append(
        &mut self,
        _contour: &MFEKContour<MFEKGlifPointData>,
        _append: &MFEKContour<MFEKGlifPointData>,
    ) {
    }

    fn insert(&mut self, _contour: &MFEKContour<MFEKGlifPointData>, _point_idx: usize) {}
    fn remove(&mut self, _contour: &MFEKContour<MFEKGlifPointData>, _point_idx: usize) {}
}

use glifparser::glif::{DashContour, Glif, MFEKContour, MFEKOutline};

use super::ContourOperation;
use crate::util::MFEKGlifPointData;

impl ContourOperation for DashContour {
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

    fn sub(&self, _contour: &MFEKContour<MFEKGlifPointData>, _begin: usize, _end: usize) -> Self {
        self.clone()
    }

    fn append(
        &self,
        _contour: &MFEKContour<MFEKGlifPointData>,
        _append: &MFEKContour<MFEKGlifPointData>,
    ) -> Self {
        self.clone()
    }

    fn insert(&self, _contour: &MFEKContour<MFEKGlifPointData>, _point_idx: usize) -> Self {
        self.clone()
    }
}

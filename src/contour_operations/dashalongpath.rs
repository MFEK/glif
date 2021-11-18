use glifparser::glif::{DashContour, Glif, MFEKContour, MFEKOutline, MFEKPointData};

use super::ContourOperation;

impl ContourOperation for DashContour {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
        let mut glif = Glif::default();
        glif.outline = Some(vec![contour.inner.clone()]);
        let dash_output = MFEKmath::dash_along_glif(&glif, self);

        let mut output: MFEKOutline<MFEKPointData> = Vec::new();
        if let Some(outline) = dash_output.outline {
            for contour in outline {
                output.push(contour.into());
            }
        }

        output
    }

    fn sub(&self, _contour: &MFEKContour<MFEKPointData>, _begin: usize, _end: usize) -> Self {
        self.clone()
    }

    fn append(
        &self,
        _contour: &MFEKContour<MFEKPointData>,
        _append: &MFEKContour<MFEKPointData>,
    ) -> Self {
        self.clone()
    }

    fn insert(&self, _contour: &MFEKContour<MFEKPointData>, _point_idx: usize) -> Self {
        self.clone()
    }
}

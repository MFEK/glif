use MFEKmath::{Piecewise, pattern_along_path_mfek};
use glifparser::glif::{MFEKContour, MFEKOutline, MFEKPointData, PAPContour};

use super::ContourOperation;

impl ContourOperation for PAPContour {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData>
    {
        let contour_pw = Piecewise::from(&contour.inner);

        let pap_output = pattern_along_path_mfek(&contour_pw, self);

        let mut output: MFEKOutline<MFEKPointData> = Vec::new();
        for contour in pap_output.segs {
            output.push(contour.to_contour().into());
        }

        output
    }

    fn sub(&self, _contour: &MFEKContour<MFEKPointData>, _begin: usize, _end: usize) -> Self {
        self.clone()
    }

    fn append(&self, _contour: &MFEKContour<MFEKPointData>, _append: &MFEKContour<MFEKPointData>) -> Self {
        self.clone()
    }

    fn insert(&self, _contour: &MFEKContour<MFEKPointData>, _point_idx: usize) -> Self {
        self.clone()
    }
}

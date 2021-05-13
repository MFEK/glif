use MFEKmath::{Piecewise, VWSSettings, pattern_along_path_mfek};
use glifparser::{VWSContour, glif::{self, MFEKContour, MFEKOutline, MFEKPointData, PAPContour, VWSHandle}};

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

        return output;
    }

    fn sub(&self, contour: &MFEKContour<MFEKPointData>, begin: usize, end: usize) -> Self {
        return self.clone();
    }

    fn append(&self, contour: &MFEKContour<MFEKPointData>, append: &MFEKContour<MFEKPointData>) -> Self {
        return self.clone();
    }

    fn insert(&self, contour: &MFEKContour<MFEKPointData>, point_idx: usize) -> Self {
        return self.clone();
    }
}
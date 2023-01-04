use glifparser::{glif::{MFEKContour, MFEKOutline, contour::MFEKContourCommon}, MFEKPointData};
use glifparser::glif::contour_operations::pap::PAPContour;
use MFEKmath::{pattern_along_path_mfek, Piecewise};

use super::{ContourOperationBuild};

impl ContourOperationBuild for PAPContour<MFEKPointData> {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
        let contour_pw = Piecewise::from(contour.cubic().unwrap());

        let pap_output = pattern_along_path_mfek(&contour_pw, self);

        let mut output: MFEKOutline<MFEKPointData> = Vec::new();
        for contour in pap_output.segs {
            output.push(contour.to_contour().into());
        }

        output
    }
}
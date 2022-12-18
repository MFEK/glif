use MFEKmath::mfek::ResolveCubic;
use glifparser::MFEKPointData;
use glifparser::glif::contour::MFEKContourCommon;
use glifparser::glif::inner::MFEKContourInner;
use glifparser::glif::{MFEKContour, MFEKOutline};
use glifparser::glif::contour_operations::vws::{VWSContour};
use MFEKmath::{variable_width_stroke, Piecewise, VWSSettings};

use super::ContourOperationBuild;

impl ContourOperationBuild for VWSContour {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
        let contour_pw = Piecewise::from(contour.to_cubic());

        let settings = VWSSettings::<MFEKPointData> {
            cap_custom_start: None,
            cap_custom_end: None,
        };

        let vws_output = variable_width_stroke(&contour_pw, self, &settings);

        let mut output: MFEKOutline<MFEKPointData> = Vec::new();
        for contour in vws_output.segs {
            output.push(contour.to_contour().into());
        }

        output
    }
}

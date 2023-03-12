use glifparser::glif::contour::MFEKContourCommon;
use glifparser::glif::{contour_operations::dash::DashContour, MFEKContour, MFEKOutline};
use glifparser::{Glif, MFEKPointData};
use MFEKmath::mfek::ResolveCubic;

use super::ContourOperationBuild;

impl ContourOperationBuild for DashContour {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
        let mut glif = Glif::default();

        // TODO: Get rid of this call to resolve to cubic and use some internal cache.
        glif.outline = Some(vec![contour.to_cubic().cubic().unwrap().clone()]);
        let dash_output = MFEKmath::dash_along_glif(&glif, self);

        let mut output: MFEKOutline<MFEKPointData> = Vec::new();
        if let Some(outline) = dash_output.outline {
            for contour in outline {
                output.push(contour.into());
            }
        }

        output
    }
}

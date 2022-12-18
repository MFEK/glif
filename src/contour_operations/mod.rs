pub mod dashalongpath;
pub mod patternalongpath;
pub mod variablewidthstroke;

use glifparser::glif::inner::MFEKContourInner;
use glifparser::glif::{MFEKContour, MFEKOutline};
use glifparser::glif::contour_operations::{ContourOperations, unknown_op_outline};
use glifparser::MFEKPointData;

pub trait ContourOperationBuild {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData>;
}

impl ContourOperationBuild for Option<ContourOperations<MFEKPointData>> {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
        if contour.operation().is_none() {
            let mut ret: MFEKOutline<MFEKPointData> = MFEKOutline::new();
            ret.push(MFEKContour::new(contour.inner().clone(), None));
            return ret;
        }

        match self.as_ref() {
            Some(ContourOperations::VariableWidthStroke { data }) => data.build(contour),
            Some(ContourOperations::PatternAlongPath { data }) => data.build(contour),
            Some(ContourOperations::DashAlongPath { data }) => data.build(contour),
            _ => unknown_op_outline(),
        }
    }
}

pub mod dashalongpath;
pub mod patternalongpath;
pub mod variablewidthstroke;

use glifparser::glif::{MFEKContour, MFEKOutline};
use glifparser::glif::contour_operations::{ContourOperations, unknown_op_outline};
use glifparser::MFEKPointData;

pub trait ContourOperationBuild {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData>;
}

impl ContourOperationBuild for Option<ContourOperations<MFEKPointData>> {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
        let op = contour.operation.clone();
        if op.is_none() {
            return vec![contour.clone()];
        }

        match op.unwrap() {
            ContourOperations::VariableWidthStroke { data } => data.build(contour),
            ContourOperations::PatternAlongPath { data } => data.build(contour),
            ContourOperations::DashAlongPath { data } => data.build(contour),
            _ => unknown_op_outline(),
        }
    }
}

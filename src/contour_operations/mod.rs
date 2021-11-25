pub mod dashalongpath;
pub mod patternalongpath;
pub mod variablewidthstroke;
use glifparser::glif::{ContourOperations, MFEKContour, MFEKOutline, MFEKPointData};

use log;

fn unknown_op() -> Option<ContourOperations<MFEKPointData>> {
    log::warn!("Found unknown contour operation attached to contour. File was generated with newer MFEKglif, please upgrade to edit properly.");
    None
}

fn unknown_op_outline() -> MFEKOutline<MFEKPointData> {
    unknown_op();
    MFEKOutline::new()
}

pub trait ContourOperation {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData>;
    fn sub(&self, contour: &MFEKContour<MFEKPointData>, begin: usize, end: usize) -> Self;
    fn append(
        &self,
        contour: &MFEKContour<MFEKPointData>,
        append: &MFEKContour<MFEKPointData>,
    ) -> Self;
    fn insert(&self, contour: &MFEKContour<MFEKPointData>, idx: usize) -> Self;
}

pub fn sub(
    contour: &MFEKContour<MFEKPointData>,
    begin: usize,
    end: usize,
) -> Option<ContourOperations<MFEKPointData>> {
    let op = contour.operation.clone();
    op.as_ref()?;

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data } => {
            Some(ContourOperations::VariableWidthStroke {
                data: data.sub(contour, begin, end),
            })
        }
        ContourOperations::PatternAlongPath { data } => Some(ContourOperations::PatternAlongPath {
            data: data.sub(contour, begin, end),
        }),
        ContourOperations::DashAlongPath { data } => Some(ContourOperations::DashAlongPath {
            data: data.sub(contour, begin, end),
        }),
        _ => unknown_op(),
    }
}

pub fn append(
    contour: &MFEKContour<MFEKPointData>,
    append: &MFEKContour<MFEKPointData>,
) -> Option<ContourOperations<MFEKPointData>> {
    let op = contour.operation.clone();
    op.as_ref()?;

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data } => {
            Some(ContourOperations::VariableWidthStroke {
                data: data.append(contour, append),
            })
        }
        ContourOperations::PatternAlongPath { data } => Some(ContourOperations::PatternAlongPath {
            data: data.append(contour, append),
        }),
        ContourOperations::DashAlongPath { data } => Some(ContourOperations::DashAlongPath {
            data: data.append(contour, append),
        }),
        _ => unknown_op(),
    }
}

pub fn build(contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
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

pub fn insert(contour: &MFEKContour<MFEKPointData>, idx: usize) -> Option<ContourOperations<MFEKPointData>> {
    let op = contour.operation.clone();
    op.as_ref()?;

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data } => {
            Some(ContourOperations::VariableWidthStroke {
                data: data.insert(contour, idx),
            })
        }
        ContourOperations::PatternAlongPath { data } => Some(ContourOperations::PatternAlongPath {
            data: data.insert(contour, idx),
        }),
        ContourOperations::DashAlongPath { data } => Some(ContourOperations::DashAlongPath {
            data: data.insert(contour, idx),
        }),
        _ => unknown_op(),
    }
}

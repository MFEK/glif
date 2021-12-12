pub mod dashalongpath;
pub mod patternalongpath;
pub mod variablewidthstroke;

use crate::util::MFEKGlifPointData;

use glifparser::glif::{ContourOperations, MFEKContour, MFEKOutline};

use log;

fn unknown_op() -> Option<ContourOperations<MFEKGlifPointData>> {
    log::warn!("Found unknown contour operation attached to contour. File was generated with newer MFEKglif, please upgrade to edit properly.");
    None
}

fn unknown_op_outline() -> MFEKOutline<MFEKGlifPointData> {
    unknown_op();
    MFEKOutline::new()
}

pub trait ContourOperation {
    fn build(&self, contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData>;
    fn sub(&self, contour: &MFEKContour<MFEKGlifPointData>, begin: usize, end: usize) -> Self;
    fn append(
        &self,
        contour: &MFEKContour<MFEKGlifPointData>,
        append: &MFEKContour<MFEKGlifPointData>,
    ) -> Self;
    fn insert(&self, contour: &MFEKContour<MFEKGlifPointData>, idx: usize) -> Self;
}

pub fn sub(
    contour: &MFEKContour<MFEKGlifPointData>,
    begin: usize,
    end: usize,
) -> Option<ContourOperations<MFEKGlifPointData>> {
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
    contour: &MFEKContour<MFEKGlifPointData>,
    append: &MFEKContour<MFEKGlifPointData>,
) -> Option<ContourOperations<MFEKGlifPointData>> {
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

pub fn build(contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData> {
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

pub fn insert(
    contour: &MFEKContour<MFEKGlifPointData>,
    idx: usize,
) -> Option<ContourOperations<MFEKGlifPointData>> {
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

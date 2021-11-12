pub mod variablewidthstroke;
pub mod patternalongpath;
use glifparser::glif::{ContourOperations, MFEKContour, MFEKOutline, MFEKPointData};

pub trait ContourOperation {
    fn build(&self, contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData>;
    fn sub(&self, contour: &MFEKContour<MFEKPointData>, begin: usize, end: usize) -> Self;
    fn append(&self, contour: &MFEKContour<MFEKPointData>, append: &MFEKContour<MFEKPointData>) -> Self;
    fn insert(&self, contour: &MFEKContour<MFEKPointData>, idx: usize) -> Self;
}

pub fn sub(contour: &MFEKContour<MFEKPointData>, begin: usize, end: usize) -> Option<ContourOperations>
{
    let op = contour.operation.clone();
    op.as_ref()?;

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data} => {
            Some(ContourOperations::VariableWidthStroke { data: data.sub(contour, begin, end) })
        }
        ContourOperations::PatternAlongPath { data} => {
            Some(ContourOperations::PatternAlongPath { data: data.sub(contour, begin, end) })
        }
    }
}


pub fn append(contour: &MFEKContour<MFEKPointData>, append: &MFEKContour<MFEKPointData>) -> Option<ContourOperations>
{
    let op = contour.operation.clone();
    op.as_ref()?;

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data} => {
            Some(ContourOperations::VariableWidthStroke { data: data.append(contour, append) })
        }
        ContourOperations::PatternAlongPath { data} => {
            Some(ContourOperations::PatternAlongPath { data: data.append(contour, append) })
        }
    }
}

pub fn build(contour: &MFEKContour<MFEKPointData>) -> MFEKOutline<MFEKPointData> {
    let op = contour.operation.clone();
    if op.is_none() { return vec!(contour.clone()) }

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data} => {
            data.build(contour)
        }
        ContourOperations::PatternAlongPath { data} => {
            data.build(contour)
        }
    }
}

pub fn insert(contour: &MFEKContour<MFEKPointData>, idx: usize) -> Option<ContourOperations> {
    let op = contour.operation.clone();
    op.as_ref()?;

    match op.unwrap() {
        ContourOperations::VariableWidthStroke { data} => {
            Some(ContourOperations::VariableWidthStroke { data: data.insert(contour, idx) })
        }
        ContourOperations::PatternAlongPath { data} => {
            Some(ContourOperations::PatternAlongPath { data: data.insert(contour, idx) })
        }
    }
}

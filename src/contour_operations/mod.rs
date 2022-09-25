pub mod dashalongpath;
pub mod patternalongpath;
pub mod variablewidthstroke;

use crate::util::MFEKGlifPointData;

use glifparser::{glif::{ContourOperations, MFEKContour, MFEKOutline}};

use log;

fn unknown_op() {
    log::warn!("Found unknown contour operation attached to contour. File was generated with newer MFEKglif, please upgrade to edit properly.");
}

fn unknown_op_outline() -> MFEKOutline<MFEKGlifPointData> {
    unknown_op();
    MFEKOutline::new()
}

pub trait ContourOperationData {
    fn build(&self, contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData>;
    fn sub(&mut self, contour: &MFEKContour<MFEKGlifPointData>, begin: usize, end: usize);
    fn append(
        &mut self,
        contour: &MFEKContour<MFEKGlifPointData>,
        append: &MFEKContour<MFEKGlifPointData>,
    );
    fn insert(&mut self, contour: &MFEKContour<MFEKGlifPointData>, idx: usize);
}

pub trait ContourOperation {
    fn build(&self, contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData>;
    fn sub(&mut self, contour: &MFEKContour<MFEKGlifPointData>, begin: usize, end: usize);
    fn append(
        &mut self,
        contour: &MFEKContour<MFEKGlifPointData>,
        append: &MFEKContour<MFEKGlifPointData>,
    );
    fn insert_op(&mut self, contour: &MFEKContour<MFEKGlifPointData>, idx: usize);
}

impl ContourOperation for Option<ContourOperations<MFEKGlifPointData>> {
    fn sub(
        &mut self,
        contour: &MFEKContour<MFEKGlifPointData>,
        begin: usize,
        end: usize,
    ) {
        if let Some(op) = self.as_mut() {
            match op {
                ContourOperations::VariableWidthStroke { ref mut data } => {data.sub(contour, begin, end)}
                ContourOperations::PatternAlongPath { ref mut data } => {data.sub(contour, begin, end)}
                ContourOperations::DashAlongPath { ref mut data } => {data.sub(contour, begin, end)}
                _ => unknown_op()
            }
        }
    }
    
    fn append(
        &mut self,
        contour: &MFEKContour<MFEKGlifPointData>,
        append: &MFEKContour<MFEKGlifPointData>,
    ) {    
        if let Some(op) = self.as_mut() {
            match op {
                ContourOperations::VariableWidthStroke { ref mut data } => {data.append(contour, append)}
                ContourOperations::PatternAlongPath { ref mut data } => {data.append(contour, append)}
                ContourOperations::DashAlongPath { ref mut data } => {data.append(contour, append)}
                _ => unknown_op()
            }
        }
    }
    
    fn build(&self, contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData> {
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
    
    fn insert_op(
        &mut self,
        contour: &MFEKContour<MFEKGlifPointData>,
        idx: usize,
    ) {
        if let Some(op) = self.as_mut() {
            match op {
                ContourOperations::VariableWidthStroke { ref mut data } => {data.insert(contour, idx)}
                ContourOperations::PatternAlongPath { ref mut data } => {data.insert(contour, idx)}
                ContourOperations::DashAlongPath { ref mut data } => {data.insert(contour, idx)}
                _ => unknown_op()
            }
        }
    }
    
}


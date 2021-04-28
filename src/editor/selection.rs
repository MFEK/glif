use std::collections::HashMap;
use MFEKmath::{Bezier, Evaluate, Piecewise};
use glifparser::{Handle, Outline, Point, PointType, glif::{Layer, MFEKPointData}};

use crate::renderer::points::calc::*;
use super::Editor;
use super::util::is_point_selected;

impl Editor {
    /// Copy the current selection and put it in our clipboard. 
    pub fn copy_selection(&mut self) {        
        let layer = &self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<Vec<Point<MFEKPointData>>> = Vec::new();
        for (contour_idx, contour) in layer.outline.as_ref().unwrap().iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();
            let mut deleted = false;
            for (point_idx, point) in contour.iter().enumerate() {
                let to_delete = !is_point_selected(self, contour_idx, point_idx);

                if to_delete {
                    results.push(cur_contour);
                    cur_contour = Vec::new();
                    deleted = true;
                } else  {
                    cur_contour.push(point.clone());
                }
            }
            results.push(cur_contour);

            if results.len() > 1 && contour.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.append(&mut results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.len() != 0 {
                    if deleted {
                        result.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result); 
                }
            }
        }

        self.clipboard = Some(Layer{
            outline: Some(new_outline),
            contour_ops: HashMap::new(),
            operation: None,
        })
    }

    pub fn paste_selection(&mut self, position: (f32, f32)) {  
        self.begin_layer_modification("Paste clipboard.");
        if let Some(clipboard) = &self.clipboard {
            self.contour_idx = None;
            self.point_idx = None;
            self.selected.clear();

            let pw: Piecewise<Piecewise<Bezier>> = clipboard.outline.as_ref().unwrap().into();
            let size = pw.bounds();

            let outline = clipboard.outline.as_ref().unwrap();
            let translated_outline: Outline<MFEKPointData> = outline.iter().map(|contour| {
                contour.iter().map(|point| {
                    let mut translated_point = point.clone();
                    let offset_x = calc_x(position.0 as f32) - size.width() as f32 / 2.;
                    let offset_y = calc_y(position.1 as f32) - size.height() as f32 / 2.;


                    translated_point.x = translated_point.x + offset_x;
                    translated_point.y = translated_point.y + offset_y;

                    translated_point.a = match translated_point.a {
                        Handle::At(x, y) => { Handle::At(
                            x + offset_x,
                            y + offset_y

                    )}
                        _ => {Handle::Colocated}
                    };
                    
                    translated_point.b = match translated_point.b {
                        Handle::At(x, y) => { Handle::At(
                            x + offset_x,
                            y + offset_y

                    )}
                        _ => {Handle::Colocated}
                    };
                    translated_point
                }).collect()
            }).collect();

            let layer = &mut self.glyph.as_mut().unwrap().glif.layers[self.layer_idx.unwrap()];
            for contour in translated_outline {
                let cur_idx = layer.outline.as_ref().unwrap().len();
                for (new_selection, _) in contour.iter().enumerate() {
                    self.selected.insert((cur_idx, new_selection));
                }
                layer.outline.as_mut().unwrap().push(contour);
            }
        }
        self.end_layer_modification();
    }

    pub fn delete_selection(&mut self) {
        self.begin_layer_modification("Delete selection.");
        
        let layer = &self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<Vec<Point<MFEKPointData>>> = Vec::new();
        for (contour_idx, contour) in layer.outline.as_ref().unwrap().iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();
            let mut deleted = false;
            for (point_idx, point) in contour.iter().enumerate() {
                let to_delete = is_point_selected(self, contour_idx, point_idx);

                if to_delete {
                    results.push(cur_contour);
                    cur_contour = Vec::new();
                    deleted = true;
                } else  {
                    cur_contour.push(point.clone());
                }
            }
            results.push(cur_contour);

            if results.len() > 1 && contour.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.append(&mut results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.len() != 0 {
                    if deleted {
                        result.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result); 
                }
            }
        }

        self.glyph.as_mut().unwrap().glif.layers[self.layer_idx.unwrap()].outline = Some(new_outline);

        self.end_layer_modification();

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }
}
use std::collections::HashMap;
use MFEKmath::{Bezier, Evaluate, Piecewise};
use glifparser::{Handle, Outline, Point, PointType, glif::{Layer, MFEKContour, MFEKPointData}};

use crate::renderer::points::calc::*;
use super::Editor;
use super::util::is_point_selected;

impl Editor {
    /// Copy the current selection and put it in our clipboard. 
    pub fn copy_selection(&mut self) {        
        let layer = &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<MFEKContour<MFEKPointData>> = Vec::new();
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();
            let mut deleted = false;
            for (point_idx, point) in contour.inner.iter().enumerate() {
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

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.append(&mut results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.len() != 0 {
                    if deleted {
                        result.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result.into()); 
                }
            }
        }

        self.clipboard = Some(Layer{
            name: "".to_string(),
            visible: true,
            color: None,
            outline: new_outline,
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

            let pw: Piecewise<Piecewise<Bezier>> = (&clipboard.outline).into();
            let size = pw.bounds();

            let translated_outline: Outline<MFEKPointData> = clipboard.outline.iter().map(|contour| {
                contour.inner.iter().map(|point| {
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

            let layer = &mut self.glyph.as_mut().unwrap().layers[self.layer_idx.unwrap()];
            for contour in translated_outline {
                let cur_idx = layer.outline.len();
                for (new_selection, _) in contour.iter().enumerate() {
                    self.selected.insert((cur_idx, new_selection));
                }
                layer.outline.push(contour.into());
            }
        }
        self.end_layer_modification();
    }

    pub fn delete_selection(&mut self) {
        self.begin_layer_modification("Delete selection.");
        
        let layer = &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<MFEKContour<MFEKPointData>> = Vec::new();
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();
            let mut deleted = false;
            for (point_idx, point) in contour.inner.iter().enumerate() {
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

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.append(&mut results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.len() != 0 {
                    if deleted {
                        result.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result.into()); 
                }
            }
        }

        self.glyph.as_mut().unwrap().layers[self.layer_idx.unwrap()].outline = new_outline;

        self.end_layer_modification();

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }
}
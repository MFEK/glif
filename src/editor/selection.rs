use glifparser::{
    glif::{Layer, MFEKContour, MFEKPointData},
    Handle, PointType,
};
use MFEKmath::{Rect, Vector};

use super::Editor;
use crate::contour_operations;

impl Editor {
    /// Copy the current selection and put it in our clipboard.
    pub fn copy_selection(&mut self) {
        let layer = &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<MFEKContour<MFEKPointData>> = Vec::new();
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();

            let mut begin = 0;

            let mut deleted = false;
            for (point_idx, point) in contour.inner.iter().enumerate() {
                let to_delete = !self.point_selected(contour_idx, point_idx);

                if to_delete {
                    let mut mfekcur: MFEKContour<MFEKPointData> = cur_contour.into();
                    mfekcur.operation = contour_operations::sub(&contour, begin, point_idx);
                    results.push(mfekcur);

                    cur_contour = Vec::new();
                    deleted = true;
                    begin = point_idx + 1;
                } else {
                    cur_contour.push(point.clone());
                }
            }
            let mut mfekcur: MFEKContour<MFEKPointData> = cur_contour.into();
            mfekcur.operation = contour_operations::sub(&contour, begin, contour.inner.len() - 1);
            results.push(mfekcur);

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.inner.append(&mut results[0].inner);
                move_to_front.operation = contour_operations::append(&move_to_front, &results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.inner.len() != 0 {
                    if deleted {
                        result.inner.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result);
                }
            }
        }

        self.clipboard = Some(Layer {
            name: "".to_string(),
            visible: true,
            color: None,
            outline: new_outline,
            operation: None,
            images: layer.images.clone(),
        })
    }

    pub fn paste_selection(&mut self, _position: (f32, f32)) {
        self.begin_modification("Paste clipboard.");
        if let Some(clipboard) = &self.clipboard {
            self.contour_idx = None;
            self.point_idx = None;
            self.selected.clear();

            let layer = &mut self.glyph.as_mut().unwrap().layers[self.layer_idx.unwrap()];
            for contour in clipboard.outline.iter() {
                let cur_idx = layer.outline.len();
                for (point_selection, _) in contour.inner.iter().enumerate() {
                    self.selected.insert((cur_idx, point_selection));
                }
                layer.outline.push(contour.clone());
            }
        }
        self.end_modification();
    }

    pub fn delete_selection(&mut self) {
        self.begin_modification("Delete selection.");

        let layer = &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<MFEKContour<MFEKPointData>> = Vec::new();
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();

            let mut begin = 0;

            let mut deleted = false;
            for (point_idx, point) in contour.inner.iter().enumerate() {
                let to_delete = self.point_selected(contour_idx, point_idx);

                if to_delete {
                    let mut mfekcur: MFEKContour<MFEKPointData> = cur_contour.into();
                    mfekcur.operation = contour_operations::sub(&contour, begin, point_idx);
                    results.push(mfekcur);

                    cur_contour = Vec::new();
                    deleted = true;
                    begin = point_idx + 1;
                } else {
                    cur_contour.push(point.clone());
                }
            }
            let mut mfekcur: MFEKContour<MFEKPointData> = cur_contour.into();
            mfekcur.operation = contour_operations::sub(&contour, begin, contour.inner.len() - 1);
            results.push(mfekcur);

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.inner.append(&mut results[0].inner);
                move_to_front.operation = contour_operations::append(&move_to_front, &results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.inner.len() != 0 {
                    if deleted {
                        result.inner.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result);
                }
            }
        }

        self.glyph.as_mut().unwrap().layers[self.layer_idx.unwrap()].outline = new_outline;

        self.end_modification();

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }

    pub fn build_selection_bounding_box(&self) -> Rect {
        let mut points = vec![];
        for (ci, pi) in &self.selected {
            let point = self.with_active_layer(|layer| layer.outline[*ci].inner[*pi].clone());
            points.push(Vector {
                x: point.x as f64,
                y: point.y as f64,
            });

            match point.a {
                Handle::At(x, y) => {
                    points.push(Vector {
                        x: x as f64,
                        y: y as f64,
                    });
                }
                _ => {}
            }

            match point.b {
                Handle::At(x, y) => {
                    points.push(Vector {
                        x: x as f64,
                        y: y as f64,
                    });
                }
                _ => {}
            }
        }

        return Rect::AABB_from_points(points);
    }

    pub fn get_selection_bounding_box_center(&self) -> (f32, f32) {
        let bounding_box = self.build_selection_bounding_box();

        let half_width = ((bounding_box.left - bounding_box.right) / 2.) as f32;
        let half_height = ((bounding_box.top - bounding_box.bottom) / 2.) as f32;
        return (
            bounding_box.left as f32 - half_width,
            bounding_box.top as f32 - half_height,
        );
    }

    pub fn selected(&self) -> Option<(usize, usize)> {
        if let (Some(ci), Some(pi)) = (self.contour_idx, self.point_idx) {
            // single click
            Some((ci, pi))
        } else if let Some((ci, pi)) = self.selected.iter().next() {
            // selbox
            Some((*ci, *pi))
        } else {
            None
        }
    }

    pub fn point_selected(&self, contour_idx: usize, point_idx: usize) -> bool {
        if let Some(editor_pidx) = self.point_idx {
            let editor_cidx = self.contour_idx.unwrap();

            if contour_idx == editor_cidx && point_idx == editor_pidx {
                return true;
            };
        }

        if self.selected.contains(&(contour_idx, point_idx)) {
            return true;
        };

        return false;
    }
}

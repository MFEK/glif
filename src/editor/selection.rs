use glifparser::{
    glif::{Layer, MFEKContour},
    outline::skia::ToSkiaPaths as _,
    Handle, PointType,
};
use MFEKmath::{Rect, Vector};

use arboard::{self, Clipboard};
use serde_json;
use shrinkwraprs;
use skulpin::skia_safe::Point as SkPoint;

use super::Editor;
use crate::contour_operations;
use crate::util::MFEKGlifPointData;

use std::collections::HashSet;
use std::fmt;

#[derive(shrinkwraprs::Shrinkwrap)]
#[shrinkwrap(mutable)]
pub(crate) struct EditorClipboard(pub(crate) Clipboard);

impl fmt::Debug for EditorClipboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Clipboard").finish()
    }
}

impl Default for EditorClipboard {
    fn default() -> Self {
        EditorClipboard(Clipboard::new().expect("Failed to initialize clipboard"))
    }
}

impl Editor {
    /// Copy the current selection and put it in our clipboard.
    pub fn copy_selection(&mut self) {
        let layer = &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<MFEKContour<MFEKGlifPointData>> = Vec::new();
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();

            let mut begin = 0;

            let mut deleted = false;
            for (point_idx, point) in contour.inner.iter().enumerate() {
                let to_delete = !self.point_selected(contour_idx, point_idx);

                if to_delete {
                    let mut mfekcur: MFEKContour<MFEKGlifPointData> = cur_contour.into();
                    mfekcur.operation = contour_operations::sub(contour, begin, point_idx);
                    results.push(mfekcur);

                    cur_contour = Vec::new();
                    deleted = true;
                    begin = point_idx + 1;
                } else {
                    cur_contour.push(point.clone());
                }
            }
            let mut mfekcur: MFEKContour<MFEKGlifPointData> = cur_contour.into();
            mfekcur.operation = contour_operations::sub(contour, begin, contour.inner.len());
            results.push(mfekcur);

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.inner.append(&mut results[0].inner);
                move_to_front.operation = contour_operations::append(&move_to_front, &results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if !result.inner.is_empty() {
                    if deleted {
                        result.inner.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result);
                }
            }
        }

        let mut cliptext = String::from("text/vnd.mfek.glifjson\t");

        cliptext.push_str(
            std::str::from_utf8(
                &serde_json::to_vec_pretty(&Layer {
                    name: "".to_string(),
                    visible: true,
                    color: None,
                    outline: new_outline,
                    operation: None,
                    images: layer.images.clone(),
                })
                .unwrap(),
            )
            .unwrap(),
        );

        self.clipboard.as_mut().set_text(cliptext).unwrap();
    }

    /// If `position` is provided, it means that the client is requesting that the layer outline be
    /// moved
    pub fn paste_selection(&mut self, position: Option<(f32, f32)>) {
        if let Ok(clipboard) = self.clipboard.as_mut().get_text() {
            let (mimetype, data) = if let Some((mimetype, data)) = clipboard.split_once('\t') {
                (mimetype, data)
            } else {
                log::debug!("Tried to paste in a clipboard w/o tab (\\t) character");
                return;
            };

            if mimetype != "text/vnd.mfek.glifjson" {
                log::warn!("We must've misrecognized data w/tab (\\t) character as ours, aborting");
                return;
            }

            let mut clipboard: Layer<_> = if let Ok(cdata) = serde_json::from_str(data) {
                cdata
            } else {
                log::error!("Could not understand text/vnd.mfek.glifjson we think we produced. Mismatched MFEKglif versions running on same machine?");
                return;
            };
            log::debug!("Got layer {} from clipboard", &clipboard.name);

            self.begin_modification("Paste clipboard.");
            self.contour_idx = None;
            self.point_idx = None;
            self.selected.clear();

            let new_selected = self.with_active_layer_mut(|layer| {
                if let Some(mpos) = position {
                    let comb = clipboard.outline.to_skia_paths(None).combined();
                    let b = comb.bounds();
                    let center = b.center();
                    let dist = SkPoint::new(mpos.0 - center.x, mpos.1 - center.y);
                    for contour in clipboard.outline.iter_mut() {
                        for point in contour.inner.iter_mut() {
                            point.x += dist.x;
                            point.y += dist.y;
                            if let Handle::At(mut ax, mut ay) = point.a {
                                ax += dist.x;
                                ay += dist.y;
                                point.a = Handle::At(ax, ay);
                            }
                            if let Handle::At(mut bx, mut by) = point.b {
                                bx += dist.x;
                                by += dist.y;
                                point.b = Handle::At(bx, by);
                            }
                        }
                    }
                }

                let mut new_selected = HashSet::new();

                for contour in clipboard.outline.iter_mut() {
                    let cur_idx = layer.outline.len();
                    for (point_selection, _) in contour.inner.iter().enumerate() {
                        new_selected.insert((cur_idx, point_selection));
                    }
                    layer.outline.push(contour.clone());
                }

                new_selected
            });

            self.selected.extend(new_selected);

            self.end_modification();
        }
    }

    pub fn delete_selection(&mut self) {
        self.begin_modification("Delete selection.");

        let layer = &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<MFEKContour<MFEKGlifPointData>> = Vec::new();
        for (contour_idx, contour) in layer.outline.iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();

            let mut begin = 0;

            let mut deleted = false;
            for (point_idx, point) in contour.inner.iter().enumerate() {
                let to_delete = self.point_selected(contour_idx, point_idx);

                if to_delete {
                    let mut mfekcur: MFEKContour<MFEKGlifPointData> = cur_contour.into();
                    mfekcur.operation = contour_operations::sub(contour, begin, point_idx);
                    results.push(mfekcur);

                    cur_contour = Vec::new();
                    deleted = true;
                    begin = point_idx + 1;
                } else {
                    cur_contour.push(point.clone());
                }
            }
            let mut mfekcur: MFEKContour<MFEKGlifPointData> = cur_contour.into();
            mfekcur.operation = contour_operations::sub(contour, begin, contour.inner.len());
            results.push(mfekcur);

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.inner.append(&mut results[0].inner);
                move_to_front.operation = contour_operations::append(&move_to_front, &results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if !result.inner.is_empty() {
                    if deleted {
                        result.inner.first_mut().unwrap().ptype = PointType::Move;
                        //result.inner.first_mut().unwrap().b = Handle::Colocated;
                        //result.inner.last_mut().unwrap().a = Handle::Colocated;
                    }
                    new_outline.push(result);
                }
            }
        }

        self.with_active_layer_mut(|layer| layer.outline = new_outline.clone());

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();

        self.end_modification();
    }

    pub fn build_selection_bounding_box(&self) -> Rect {
        let mut points = vec![];
        for (ci, pi) in &self.selected {
            let point = self.with_active_layer(|layer| layer.outline[*ci].inner[*pi].clone());
            points.push(Vector {
                x: point.x as f64,
                y: point.y as f64,
            });

            if let Handle::At(x, y) = point.a {
                points.push(Vector {
                    x: x as f64,
                    y: y as f64,
                });
            }

            if let Handle::At(x, y) = point.b {
                points.push(Vector {
                    x: x as f64,
                    y: y as f64,
                });
            }
        }

        Rect::AABB_from_points(points)
    }

    pub fn get_selection_bounding_box_center(&self) -> (f32, f32) {
        let bounding_box = self.build_selection_bounding_box();

        let half_width = ((bounding_box.left - bounding_box.right) / 2.) as f32;
        let half_height = ((bounding_box.top - bounding_box.bottom) / 2.) as f32;
        (
            bounding_box.left as f32 - half_width,
            bounding_box.top as f32 - half_height,
        )
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
            }
        }

        self.selected.contains(&(contour_idx, point_idx))
    }
}

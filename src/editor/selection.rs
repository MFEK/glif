use flo_curves::{BezierCurve, bezier::fit_curve_cubic};
use glifparser::{
    glif::{Layer, MFEKContour},
    outline::skia::ToSkiaPaths as _,
    Handle, PointType,
};
use MFEKmath::{Rect, Vector, Evaluate, Bezier};

use arboard::{self, Clipboard};
use serde_json;
use shrinkwraprs;
use skulpin::skia_safe::Point as SkPoint;

use super::Editor;
use crate::contour_operations::{ContourOperation};
use crate::user_interface::gui;
use crate::util::MFEKGlifPointData;

use std::collections::HashSet;
use std::fmt;

#[derive(shrinkwraprs::Shrinkwrap)]
#[shrinkwrap(mutable)]
pub(crate) struct EditorClipboard(pub(crate) Result<Clipboard, String>);

impl fmt::Debug for EditorClipboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Clipboard").finish()
    }
}

impl Default for EditorClipboard {
    fn default() -> Self {
        let cb = Clipboard::new();
        Self(match cb {
            Ok(cb) => Ok(cb),
            Err(e) => {
                gui::error!(
                    "Failed to start OS clipboard! Wayland? (Restart compositor??) {}",
                    &e
                );
                Err(e.to_string())
            }
        })
    }
}

impl EditorClipboard {
    /// Do something to OS clipboard if we can access it
    pub fn with<F, T>(&mut self, f: F) -> Option<T>
    where
        F: for<'a> Fn(&'a mut Clipboard) -> T,
    {
        match &mut self.0 {
            Ok(ref mut cb) => Some(f(cb)),
            Err(e) => {
                gui::error!("Cannot access clipboard! {:?}", &e);
                None
            }
        }
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
                    mfekcur.operation.sub(contour, begin, point_idx);
                    results.push(mfekcur);

                    cur_contour = Vec::new();
                    deleted = true;
                    begin = point_idx + 1;
                } else {
                    cur_contour.push(point.clone());
                }
            }
            let mut mfekcur: MFEKContour<MFEKGlifPointData> = cur_contour.into();
            mfekcur.operation.sub(contour, begin, contour.inner.len());
            results.push(mfekcur);

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.inner.append(&mut results[0].inner);

                let start = move_to_front.clone();
                let end = results[0].clone();
                move_to_front.operation.append(&start, &end);
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

        self.clipboard
            .with(|c| {
                c.set_text(cliptext.clone()).unwrap_or_else(|e| {
                    let e = e.to_string();
                    gui::error!("Clipboard issue—couldn't copy! {}", e);
                })
            })
            .unwrap_or(());
    }

    /// If `position` is provided, it means that the client is requesting that the layer outline be
    /// moved
    pub fn paste_selection(&mut self, position: Option<(f32, f32)>) {
        let mut clipboard: Layer<_> = if let Some(data) = self.clipboard.with(|clipboard: &mut Clipboard| {
            let cbtext; // [For borrow checker!]
            let (mimetype, data) = match clipboard.get_text() {
                Ok(t) => {
                    // [For borrow checker!] Hold a handle to clipboard text so not dropped at end of match {} block.
                    cbtext = t;
                    match cbtext.split_once('\t') {
                        Some((mt, d)) => {
                            (mt, d)
                        },
                        None => {
                            log::debug!("Tried to paste in a clipboard w/o tab (\\t) character");
                            return Err(());
                        },
                    }
                }
                Err(e) => {
                    gui::error!("Failed to paste! {:?}", &e);
                    return Err(());
                }
            };

            if mimetype != "text/vnd.mfek.glifjson" {
                log::warn!("We must've misrecognized data w/tab (\\t) character as ours, aborting");
                return Err(());
            }

            match serde_json::from_str(data) {
                Ok(d) => Ok(d),
                Err(e) => {
                    gui::error!("Could not understand text/vnd.mfek.glifjson we think we produced. Mismatched MFEKglif versions running on same machine? {:?}", &e);
                    Err(())
                }
            }
        }) {
            match data {
                Ok(d) => d,
                Err(()) => {
                    return;
                }
            }
        } else {
            return;
        };

        log::debug!("Got layer {} from clipboard", &clipboard.name);

        self.begin_modification("Paste clipboard.");
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();

        let new_selected = {
            let layer = self.get_active_layer_mut();
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
        };

        self.selected.extend(new_selected);

        self.end_modification();
    }

    pub fn delete_single_point(&mut self) {
        self.begin_modification("Delete selection.");

        let contour_idx = self.contour_idx.unwrap();
        let point_idx = self.point_idx.unwrap();

        let layer = self.get_active_layer_mut();
        let contour = &mut layer.outline[contour_idx];

        contour.inner.remove(point_idx);
        contour.operation.remove_op(&contour.clone(), point_idx);

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
        self.end_modification();
    }

    pub fn simplify_selection(&mut self) {      
        let contour_idx = self.contour_idx.unwrap();
        let point_idx = self.point_idx.unwrap();

        let layer = self.get_active_layer_ref();
        let contour = &layer.outline[contour_idx];

        // if we have less than three points in our contour there's nothing to work with so we just delete
        // the point
        if contour.inner.len() < 3 {
            self.delete_single_point();
            return
        }

        // if the contour is open and previous or next are out of bounds we're working with the start or end of the contour
        // so we abort and just delete the selection
        if (point_idx == 0 || point_idx == contour.inner.len() - 1) && contour.inner.first().unwrap().ptype == PointType::Move {
            self.delete_selection();
            return
        }

        let prev_idx = if point_idx == 0 {contour.inner.len() - 1} else {point_idx - 1};
        let next_idx = if point_idx == contour.inner.len() - 1 { 0 } else { point_idx + 1};

        let previous_point = contour.inner.get(prev_idx);
        let next_point = contour.inner.get(next_idx);

        // now that we know that the contour is closed we're going to wrap our prev and next points
        let previous_point = previous_point.unwrap();
        let next_point = next_point.unwrap_or(&contour.inner[0]);
        let point = &contour.inner[point_idx];

        let left_bezier = MFEKmath::Bezier::from(previous_point, point);
        let right_bezier = MFEKmath::Bezier::from(point, next_point);

        let left_bezier_characteristics = flo_curves::bezier::features_for_curve(&left_bezier, 0.1);
        let right_bezier_characteristics = flo_curves::bezier::features_for_curve(&right_bezier, 0.1);

        // if both of the beziers are not simple arches that do not change direction we abort and delete the 
        // point instead
        match left_bezier_characteristics {
            flo_curves::bezier::CurveFeatures::Arch => {},
            flo_curves::bezier::CurveFeatures::Parabolic => {},
            flo_curves::bezier::CurveFeatures::SingleInflectionPoint(_) => {},
            _ => {
                self.delete_single_point();
                return;
            }
        }

        match right_bezier_characteristics {
            flo_curves::bezier::CurveFeatures::Arch => {},
            flo_curves::bezier::CurveFeatures::Parabolic => {},
            flo_curves::bezier::CurveFeatures::SingleInflectionPoint(_) => {},
            _ => {
                self.delete_single_point();
                return;
            }
        }

        // we know that both are simple arches/linear beziers with no inflections, cusps, etc so next we're gonna see what
        // the total change in tangent over the course of the curves is and if that exceeds 90 degrees
        let start_tangent = left_bezier.tangent_at(0.0);
        let mid_left_tangent = left_bezier.tangent_at(1.0);
        let mid_right_tangent = right_bezier.tangent_at(0.0);
        let end_tangent = right_bezier.tangent_at(1.0);

        // if the difference between these tangents is greater than some small epsilon we abort and delete the point instead
        // because there's a sudden change in direction
        if mid_left_tangent.normalize().distance(mid_right_tangent.normalize()) > 0.01 {
            self.delete_single_point();
            return;
        }

        let mid_tangent = mid_left_tangent;
        let mut total_angle_change = 0.0;

        total_angle_change += start_tangent.angle(mid_tangent);
        total_angle_change += mid_tangent.angle(end_tangent);

        // Abort if the angle exceeds 90 + some small epsilon
        if total_angle_change > f64::to_radians(180.) {
            self.delete_single_point();
            return;
        }

        // we've finally handled all the cases in which we won't simplify we now need to build an array of points that
        // lie on the two beziers seperated by some small chord length
        let mut sample_points = Vec::new();

        for point in flo_curves::bezier::walk_curve_evenly(&left_bezier, 0.01, 0.001) {
            sample_points.push(point.start_point());
        }

        for point in flo_curves::bezier::walk_curve_evenly(&right_bezier, 0.01, 0.001) {
            sample_points.push(point.start_point());
        }

        let mut max_error = 10.;
        let mut fitted_curve: Vec<Bezier> = fit_curve_cubic(&sample_points, &start_tangent, &-end_tangent, max_error);

        while fitted_curve.len() > 1 {
            max_error = max_error + 1.;
            fitted_curve = fit_curve_cubic(&sample_points, &start_tangent, &-end_tangent, max_error);
        }

        let fitted_curve = fitted_curve[0].clone();

        self.begin_modification("Simplify selection.");
        let layer = self.get_active_layer_mut();
        let contour = &mut layer.outline[contour_idx];


        contour.inner[prev_idx].a = Handle::At(fitted_curve.w2.x as f32, fitted_curve.w2.y as f32);
        contour.inner[next_idx].b = Handle::At(fitted_curve.w3.x as f32, fitted_curve.w3.y as f32);

        contour.inner.remove(point_idx);
        contour.operation.remove_op(&contour.clone(), point_idx);

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
        self.end_modification();
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
                    mfekcur.operation.sub(contour, begin, point_idx);
                    results.push(mfekcur);

                    cur_contour = Vec::new();
                    deleted = true;
                    begin = point_idx + 1;
                } else {
                    cur_contour.push(point.clone());
                }
            }
            let mut mfekcur: MFEKContour<MFEKGlifPointData> = cur_contour.into();
            mfekcur.operation.sub(contour, begin, contour.inner.len());
            results.push(mfekcur);

            if results.len() > 1 && contour.inner.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.inner.append(&mut results[0].inner);

                let start = move_to_front.clone();
                let end = results[0].clone();
                move_to_front.operation.append(&start, &end);

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
        self.get_active_layer_mut().outline = new_outline;

        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();

        self.end_modification();
    }

    pub fn build_selection_bounding_box(&self) -> Rect {
        let mut points = vec![];
        for (ci, pi) in &self.selected {
            let point = self.get_active_layer_ref().outline[*ci].inner[*pi].clone();
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

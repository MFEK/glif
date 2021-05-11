use std::collections::HashSet;

// Select
use crate::{get_point};
use super::{EditorEvent, Tool, prelude::*};
use crate::renderer::{UIPointType, points::draw_point};
use crate::editor::{Editor, util::clicked_point_or_handle};

use skulpin::skia_safe::dash_path_effect;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path, Rect};
use derive_more::Display;

mod dialog;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
/// Point following behavior when using the select tool
pub enum Follow {
    // Other point will take mirror action of current point.
    Mirror,
    // Other point will be forced into a line with the current point as midpoint.
    ForceLine,
    // For a quadratic curve, the "other side" of the curve (in reality, the control point on an
    // adjacent curve), should follow too.
    QuadOpposite,
    // Other point will remain in fixed position.
    No,
}

use crate::tools::MouseInfo;
use crate::sdl2::mouse::MouseButton;
impl From<MouseInfo> for Follow {
    fn from(m: MouseInfo) -> Follow {
        match m {
            MouseInfo {
                button: MouseButton::Left,
                modifiers,
                ..
            } => {
                if modifiers.ctrl {
                    Follow::ForceLine
                } else {
                    Follow::No
                }
            }
            MouseInfo {
                button: MouseButton::Right,
                ..
            } => Follow::Mirror,
            _ => Follow::QuadOpposite,
        }
    }
}


// Select is a good example of a more complicated tool that keeps lots of state.
// It has state for which handle it's selected, follow rules, selection box, and to track if it's currently
// moving a point.
#[derive(Clone)]
pub struct Select {
    follow: Follow,
    handle: WhichHandle,
    corner_one: Option<(f32, f32)>,
    corner_two: Option<(f32, f32)>,
    show_sel_box: bool,
}

impl Tool for Select {
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                match event_type {
                    super::MouseEventType::Pressed => { self.mouse_pressed(v, meta) }
                    super::MouseEventType::Released => {self.mouse_released(v, meta)}
                    super::MouseEventType::Moved => { self.mouse_moved(v, meta) }
                    super::MouseEventType::DoubleClick => { self.mouse_double_pressed(v, meta) }
                }
            }
            EditorEvent::Draw { skia_canvas } => {
                self.draw_selbox(v, skia_canvas);
                self.draw_merge_preview(v, skia_canvas);
            }
            EditorEvent::Ui { ui } => {
                self.select_settings(v, ui);
            }
            _ => {}
        }
    }
}

impl Select {
    pub fn new() -> Self {
        Self {
            follow: Follow::No,
            handle: WhichHandle::Neither,
            corner_one: None,
            corner_two: None,
            show_sel_box: false,
        }
    }
    
    fn mouse_moved(&mut self, v: &mut Editor, meta: MouseInfo) {
        if !meta.is_down { return; }
    
        let x = calc_x(meta.position.0 as f32);
        let y = calc_y(meta.position.1 as f32);

        match (v.contour_idx, v.point_idx, self.handle) {
            // Point itself is being moved.
            (Some(ci), Some(pi), WhichHandle::Neither) => {
                if !v.is_modifying() { 
                    v.begin_layer_modification("Move point.");
                }

                let reference_point = v.with_active_layer(|layer| get_point!(layer, ci, pi).clone());
                let selected = v.selected.clone();
                let ctrl_mod = meta.modifiers.ctrl;
                v.with_active_layer_mut(|layer| {
                    if !ctrl_mod {
                        for (ci, pi) in &selected {
                            let (ci, pi) = (*ci, *pi);
                            let point = &get_point!(layer, ci, pi);                          
                            let offset_x = point.x - reference_point.x;
                            let offset_y = point.y - reference_point.y;
                            move_point(&mut layer.outline, ci, pi, x + offset_x, y + offset_y);
                        }
                    }

                    move_point(&mut layer.outline, ci, pi, x, y);

                });
    
            }
            // A control point (A or B) is being moved.
            (Some(ci), Some(pi), wh) => {
                if !v.is_modifying() { 
                    v.begin_layer_modification("Move handle.");
                }
                
                v.with_active_layer_mut(|layer| {
                    let handle = match wh {
                        WhichHandle::A => get_point!(layer, ci, pi).a,
                        WhichHandle::B => get_point!(layer, ci, pi).b,
                        WhichHandle::Neither => unreachable!("Should've been matched by above?!"),
                    };
        
                    // Current x, current y
                    let (cx, cy) = match handle {
                        Handle::At(cx, cy) => (cx, cy),
                        _ => panic!("Clicked non-existent handle A! Cidx {} pidx {}", ci, pi),
                    };
        
                    // Difference in x, difference in y
                    let (dx, dy) = (cx - x, cy - y);
        
                    // If Follow::Mirror (left mouse button), other control point (handle) will do mirror
                    // image action of currently selected control point. Perhaps pivoting around central
                    // point is better?
                    macro_rules! move_mirror {
                        ($cur:ident, $mirror:ident) => {
                            get_point!(layer, ci, pi).$cur = Handle::At(x, y);
                            let h = get_point!(layer, ci, pi).$mirror;
                            match h {
                                Handle::At(hx, hy) => {
                                    if self.follow == Follow::Mirror {
                                        get_point!(layer, ci, pi).$mirror = Handle::At(hx + dx, hy + dy);
                                    } else if self.follow == Follow::ForceLine {
                                        let (px, py) =
                                            (get_point!(layer, ci, pi).x, get_point!(layer, ci, pi).y);
                                        let (dx, dy) = (px - x, py - y);
        
                                        get_point!(layer, ci, pi).$mirror = Handle::At(px + dx, py + dy);
                                    }
                                }
                                Handle::Colocated => (),
                            }
                        };
                    }
        
                    #[rustfmt::skip]
                    match wh {
                        WhichHandle::A => { move_mirror!(a, b); },
                        WhichHandle::B => { move_mirror!(b, a); },
                        WhichHandle::Neither => unreachable!("Should've been matched by above?!"),
                    }
                });
            }
            _ => {
                if !meta.modifiers.shift {
                    v.selected = HashSet::new();
                }
                self.corner_two = Some(meta.position);
                let last_selected = v.selected.clone();
                let selected = v.with_active_layer(|layer| {
                    let c1 = self.corner_one.unwrap_or((0., 0.));
                    let c2 = self.corner_two.unwrap_or((0., 0.));
                    let rect = Rect::from_point_and_size(
                        (c1.0 as f32, c1.1 as f32),
                        ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
                    );
                    
                    build_box_selection(
                        last_selected.clone(),
                        rect,
                        &layer.outline,
                    )
                });
                v.selected = selected
            },
        };
    }

    fn mouse_pressed(&mut self, v: &mut Editor, meta: MouseInfo) {

        // if we found a point or handle we're going to start a drag operation
        match clicked_point_or_handle(v, meta.position, None) {
            Some((ci, pi, wh)) => {
                if meta.modifiers.shift || meta.modifiers.ctrl {
                    if let Some(point_idx) = v.point_idx {
                        v.selected.insert((v.contour_idx.unwrap(), point_idx));
                    }
                } else if !v.selected.contains(&(ci, pi)) {
                    v.selected = HashSet::new();
                }

                v.contour_idx = Some(ci);
                v.point_idx = Some(pi);
                self.follow = meta.into();
                self.handle = wh;
            },
            None => {
                if !meta.modifiers.ctrl {
                    v.selected = HashSet::new();
                } else {
                    if let Some(point_idx) = v.point_idx {
                        v.selected.insert((v.contour_idx.unwrap(), point_idx));
                    }
                }
                v.contour_idx = None;
                v.point_idx = None;
                self.handle = WhichHandle::Neither;
                self.show_sel_box = true;
                self.corner_one = Some(meta.position);
                self.corner_two = Some(meta.position);
            },
        };
    }

    fn mouse_double_pressed(&mut self, v: &mut Editor, meta: MouseInfo) {
        let ci = if let Some((ci, _pi, _wh)) = clicked_point_or_handle(v, meta.position, None) {
            ci
        } else {
            return
        };

        let contour_len = v.with_active_layer(|layer| get_contour_len!(layer, ci));

        if !meta.modifiers.shift {
            v.selected = HashSet::new();
        }

        for pi in 0..contour_len {
            v.selected.insert((ci, pi));
        }
    }

    fn mouse_released(&mut self, v: &mut Editor, meta: MouseInfo) {
        // we are going to check if we're dropping this point onto another and if this is the end, and that the 
        // start or vice versa if so we're going to merge but first we have to check we're dragging a point
        if self.handle == WhichHandle::Neither && v.is_modifying() {
            let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

            // are we overlapping a point?
            if let Some((ci, pi, WhichHandle::Neither)) = clicked_point_or_handle(v, meta.position, Some((vci, vpi))) {
                // if that point the start or end of it's contour?
                if let Some(info) = get_contour_start_or_end(v, vci, vpi) {
                    // is our current point the start or end of it's contour?
                    if let Some(target_info) = get_contour_start_or_end(v, ci, pi) {
                        let info_type = v.with_active_layer(|layer| {get_contour_type!(layer, vci)});
                        let target_type = v.with_active_layer(|layer| {get_contour_type!(layer, ci)});

                        // do we have two starts or two ends?
                        if info_type == PointType::Move && target_type == PointType::Move && target_info != info {
                            let start = if info == SelectPointInfo::Start { vci } else { ci };
                            let end = if info == SelectPointInfo::End { vci } else { ci };
                            v.merge_contours(start, end);
                        }
                    }
                }
            }
        }


        v.end_layer_modification();
        self.show_sel_box = false;
        self.corner_one = None;
        self.corner_two = None;
    }

    // This draws a preview to show if we're overlapping a point we can merge with or not.
    // Note that all tool draw events draw over the glyph view.
    fn draw_merge_preview(&self, v: &Editor, canvas: &mut Canvas) {
        if self.handle == WhichHandle::Neither && v.is_modifying() {
            let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

            // are we overlapping a point?
            if let Some((ci, pi, WhichHandle::Neither)) = clicked_point_or_handle(v, v.mouse_info.position, Some((vci, vpi))) {
                // if that point the start or end of it's contour?
                if let Some(info) = get_contour_start_or_end(v, vci, vpi) {
                    // is our current point the start or end of it's contour?
                    if let Some(target_info) = get_contour_start_or_end(v, ci, pi) {
                        let info_type = v.with_active_layer(|layer| {get_contour_type!(layer, vci)});
                        let target_type = v.with_active_layer(|layer| {get_contour_type!(layer, ci)});

                        // do we have two starts or two ends?
                        if info_type == PointType::Move && target_type == PointType::Move && target_info != info {
                            // start and end seem flipped because we're talking about contours now the contour with the end point
                            // is actually the start
                            let merge =  v.with_active_layer(|layer| {get_contour!(layer, ci)[pi].clone()});
                            draw_point(
                                v,
                                (calc_x(merge.x), calc_y(merge.y)),
                                (merge.x, merge.y),
                                None,
                                UIPointType::Point((merge.a, merge.b)),
                                true,
                                canvas
                            );
                        }
                    }
                }
            }
        }
    }

    // This renders the dashed path of the selection box. 
    fn draw_selbox(&self, v: &Editor, canvas: &mut Canvas) {
        let c1 = self.corner_one.unwrap_or((0., 0.));
        let c2 = self.corner_two.unwrap_or((0., 0.));
    
        let mut path = Path::new();
        let mut paint = Paint::default();
        let rect = Rect::from_point_and_size(
            (c1.0 as f32, c1.1 as f32),
            ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
        );
        path.add_rect(rect, None);
        path.close();
        paint.set_color(OUTLINE_STROKE);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / v.viewport.factor));
        let dash_offset = (1. / v.viewport.factor) * 2.;
        paint.set_path_effect(dash_path_effect::new(&[dash_offset, dash_offset], 0.0));
        canvas.draw_path(&path, &paint);
    }
}


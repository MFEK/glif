use std::collections::HashSet;

// Select
use super::{EditorEvent, Tool, prelude::*};
use crate::{renderer::{UIPointType, points::draw_point}, state::{Follow, Editor}, util::math::FlipIfRequired};
use glifparser::{Handle, Outline, PointType, WhichHandle};
use skulpin::skia_safe::dash_path_effect;
use skulpin::skia_safe::{Canvas, Contains, Paint, PaintStyle, Path, Rect};

#[derive(Clone)]
pub struct Select {
    follow: Follow,
    handle: WhichHandle,
    corner_one: Option<(f64, f64)>,
    corner_two: Option<(f64, f64)>,
    show_sel_box: bool,
    modifying: bool,
}

impl Tool for Select {
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, position, meta } => {
                match event_type {
                    super::MouseEventType::Pressed => { self.mouse_pressed(v, position, meta) }
                    super::MouseEventType::Released => {self.mouse_released(v, position, meta)}
                    super::MouseEventType::Moved => { self.mouse_moved(v, position, meta) }
                }
            }
            EditorEvent::Draw { skia_canvas } => {
                 self.draw_selbox(v, skia_canvas);
                 self.draw_merge_preview(v, skia_canvas);
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
            modifying: false,
        }
    }
    
    fn mouse_moved(&mut self, v: &mut Editor, position: (f64, f64), meta: MouseMeta) {
        if !v.mousedown { return; }
    
        let x = calc_x(v.mousepos.0 as f32);
        let y = calc_y(v.mousepos.1 as f32);
    
        let single_point = match (v.contour_idx, v.point_idx, self.handle) {
            // Point itself is being moved.
            (Some(ci), Some(pi), WhichHandle::Neither) => {
                if !self.modifying { 
                    v.begin_layer_modification("Move point.");
                    self.modifying = true;
                }

                let reference_point = v.with_active_layer(|layer| get_outline!(layer)[ci][pi].clone());
                let selected = v.selected.clone();
                let ctrl_mod = meta.modifiers.ctrl;
                v.with_active_layer_mut(|layer| {
                    let outline = get_outline_mut!(layer);
                    if ctrl_mod {
                        for (ci, pi) in &selected {
                            let (ci, pi) = (*ci, *pi);
                            let point = &outline[ci][pi];                          
                            let offset_x = point.x - reference_point.x;
                            let offset_y = point.y - reference_point.y;
                            move_point(outline, ci, pi, x + offset_x, y + offset_y, self.follow);
                        }
                    }

                    move_point(outline, ci, pi, x, y, self.follow);

                });
    
                true
            }
            // A control point (A or B) is being moved.
            (Some(ci), Some(pi), wh) => {
                if !self.modifying { 
                    v.begin_layer_modification("Move handle.");
                    self.modifying = true;
                }
                
                v.with_active_layer_mut(|layer| {
                    let outline = get_outline_mut!(layer);

                    let handle = match wh {
                        WhichHandle::A => outline[ci][pi].a,
                        WhichHandle::B => outline[ci][pi].b,
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
                            outline[ci][pi].$cur = Handle::At(x, y);
                            let h = outline[ci][pi].$mirror;
                            match h {
                                Handle::At(hx, hy) => {
                                    if self.follow == Follow::Mirror {
                                        outline[ci][pi].$mirror = Handle::At(hx + dx, hy + dy);
                                    } else if self.follow == Follow::ForceLine {
                                        let (px, py) =
                                            (outline[ci][pi].x, outline[ci][pi].y);
                                        let (dx, dy) = (px - x, py - y);
        
                                        outline[ci][pi].$mirror = Handle::At(px + dx, py + dy);
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
                true
            }
            _ => false,
        };
    
        if !single_point {
            self.corner_two = Some(v.mousepos);
            let last_selected = v.selected.clone();
            let selected = v.with_active_layer(|layer| {
                let c1 = self.corner_one.unwrap_or((0., 0.));
                let c2 = self.corner_two.unwrap_or((0., 0.));
                let rect = Rect::from_point_and_size(
                    (c1.0 as f32, c1.1 as f32),
                    ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
                );
                
                build_sel_vec_from_rect(
                    last_selected.clone(),
                    rect,
                    layer.outline.as_ref(),
                )
            });
            v.selected = selected
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, position: (f64, f64), meta: MouseMeta) {

        let single_point = match v.clicked_point_or_handle(None) {
            Some((ci, pi, wh)) => {
                if meta.modifiers.shift || meta.modifiers.ctrl {
                    if let Some(point_idx) = v.point_idx {
                        v.selected.insert((v.contour_idx.unwrap(), point_idx));
                    }
                } else {
                    v.selected = HashSet::new();
                }

                v.contour_idx = Some(ci);
                v.point_idx = Some(pi);
                self.follow = meta.into();
                self.handle = wh;
                true
            },
            None => {
                v.contour_idx = None;
                v.point_idx = None;
                self.handle = WhichHandle::Neither;
                false
            },
        };
    
        if !single_point {
            if !meta.modifiers.shift {
                v.selected = HashSet::new();
            }
            self.show_sel_box = true;
            self.corner_one = Some(v.mousepos);
            self.corner_two = Some(v.mousepos);
        }
    }

    fn mouse_released(&mut self, v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        // we are going to check if we're dropping this point onto another and if this is the end, and that the 
        // start or vice versa if so we're going to merge but first we have to check we're dragging a point
        if self.handle == WhichHandle::Neither && self.modifying {
            let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

            // are we overlapping a point?
            if let Some((ci, pi, WhichHandle::Neither)) = v.clicked_point_or_handle(Some((vci, vpi))) {
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
        self.modifying = false;
        self.show_sel_box = false;
        self.corner_one = None;
        self.corner_two = None;
    }

    fn draw_merge_preview(&self, v: &Editor, canvas: &mut Canvas) {
        if self.handle == WhichHandle::Neither && self.modifying {
            let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

            // are we overlapping a point?
            if let Some((ci, pi, WhichHandle::Neither)) = v.clicked_point_or_handle(Some((vci, vpi))) {
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
        paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / v.factor));
        let dash_offset = (1. / v.factor) * 2.;
        paint.set_path_effect(dash_path_effect::new(&[dash_offset, dash_offset], 0.0));
        canvas.draw_path(&path, &paint);
    }
}

#[derive(PartialEq, Clone, Copy)]
enum SelectPointInfo {
    Start,
    End
}

fn get_contour_start_or_end(v: &Editor, contour_idx: usize, point_idx: usize) -> Option<SelectPointInfo>
{
    let contour_len = v.with_active_layer(|layer| {get_contour_len!(layer, contour_idx)} ) - 1;
    match point_idx {
        0 => Some(SelectPointInfo::Start),
        contour_len => Some(SelectPointInfo::End),
        _ => None
    }
}

pub fn build_sel_vec_from_rect(
    selected: HashSet<(usize, usize)>,
    mut rect: Rect,
    outline: Option<&Vec<glifparser::Contour<PointData>>>,
) -> HashSet<(usize, usize)> {
    rect.flip_if_required();

    let mut selected = selected.clone();
    for o in outline {
        for (cidx, contour) in o.iter().enumerate() {
            for (pidx, point) in contour.iter().enumerate() {
                if Rect::from(rect).contains(skulpin::skia_safe::Point::from((calc_x(point.x), calc_y(point.y)))) {
                    selected.insert((cidx, pidx));
                }
            }
        }
    }

    selected
}

pub fn move_point(outline: &mut Outline<PointData>, ci: usize, pi: usize, x: f32, y: f32, follow: Follow) {
    let (cx, cy) = (outline[ci][pi].x, outline[ci][pi].y);
    let (dx, dy) = (cx - x, cy - y);

    outline[ci][pi].x = x;
    outline[ci][pi].y = y;

    let a = outline[ci][pi].a;
    let b = outline[ci][pi].b;
    match a {
        Handle::At(hx, hy) => {
            outline[ci][pi].a = Handle::At(hx - dx, hy - dy)
        }
        Handle::Colocated => (),
    }
    match b {
        Handle::At(hx, hy) => {
            outline[ci][pi].b = Handle::At(hx - dx, hy - dy)
        }
        Handle::Colocated => (),
    }

}
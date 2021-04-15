// Select
use super::{EditorEvent, Tool, prelude::*};
use crate::{state::{Follow, Editor}, util::math::FlipIfRequired};
use glifparser::{Handle, WhichHandle};
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
            EditorEvent::Draw { skia_canvas } => { self.draw_selbox(v, skia_canvas) }
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

                v.with_active_layer_mut(|layer| {
                    let outline = get_outline_mut!(layer);
                    let (cx, cy) = (outline[ci][pi].x, outline[ci][pi].y);
                    let (dx, dy) = (cx - x, cy - y);

                    outline[ci][pi].x = x;
                    outline[ci][pi].y = y;

                    match self.follow {
                        // ForceLine makes no sense in this context, but putting it here prevents us from
                        // falling back to the No branch.
                        Follow::Mirror | Follow::ForceLine => {
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
                        _ => (),
                    }
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
            let selected = v.with_active_layer(|layer| {
                let c1 = self.corner_one.unwrap_or((0., 0.));
                let c2 = self.corner_two.unwrap_or((0., 0.));
                let rect = Rect::from_point_and_size(
                    (c1.0 as f32, c1.1 as f32),
                    ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
                );
                
                build_sel_vec_from_rect(
                    rect,
                    layer.outline.as_ref(),
                )
            });
            v.selected = selected
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, position: (f64, f64), meta: MouseMeta) {
        let single_point = match clicked_point_or_handle(v, position) {
            Some((ci, pi, wh)) => {
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
            self.show_sel_box = true;
            self.corner_one = Some(v.mousepos);
            self.corner_two = Some(v.mousepos);
        }
    }

    fn mouse_released(&mut self, v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        v.end_layer_modification();
        self.modifying = false;
        self.show_sel_box = false;
        self.corner_one = None;
        self.corner_two = None;
    }

    fn draw_selbox(&self, v: &mut Editor, canvas: &mut Canvas) {
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


/// Transform mouse click position into indexes into STATE.glyph.glif.outline and the handle if
/// applicable, and store it in TOOL_DATA.
fn clicked_point_or_handle(v: &mut state::Editor, position: (f64, f64)) -> Option<(usize, usize, WhichHandle)> {
    let factor = v.factor;
    let _contour_idx = 0;
    let _point_idx = 0;

    // How we do this is quite naïve. For each click, we just iterate all points and check the
    // point and both handles. It's just a bunch of floating point comparisons in a compiled
    // language, so I'm not too concerned about it, and even in the TT2020 case doesn't seem to
    // slow anything down.
    v.with_active_layer(|layer| {
        for (contour_idx, contour) in get_outline!(layer).iter().enumerate() {
            for (point_idx, point) in contour.iter().enumerate() {
                let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
                // Topleft corner of point
                let point_tl = SkPoint::new(
                    calc_x(point.x as f32) - (size / 2.),
                    calc_y(point.y as f32) - (size / 2.),
                );
                let point_rect = SkRect::from_point_and_size(point_tl, (size, size));
                // Topleft corner of handle a
                let a = point.handle_or_colocated(WhichHandle::A, |f| f, |f| f);
                let a_tl = SkPoint::new(calc_x(a.0) - (size / 2.), calc_y(a.1) - (size / 2.));
                let a_rect = SkRect::from_point_and_size(a_tl, (size, size));
                // Topleft corner of handle b
                let b = point.handle_or_colocated(WhichHandle::B, |f| f, |f| f);
                let b_tl = SkPoint::new(calc_x(b.0) - (size / 2.), calc_y(b.1) - (size / 2.));
                let b_rect = SkRect::from_point_and_size(b_tl, (size, size));
    
                // winit::PhysicalPosition as an SkPoint
                let sk_mpos = SkPoint::new(v.mousepos.0 as f32, v.mousepos.1 as f32);
    
                if point_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::Neither));
                } else if a_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::A));
                } else if b_rect.contains(sk_mpos) {
                    return Some((contour_idx, point_idx, WhichHandle::B));
                }
            }
        }
        None
    })
}

pub fn build_sel_vec_from_rect(
    mut rect: Rect,
    outline: Option<&Vec<glifparser::Contour<PointData>>>,
) -> Vec<glifparser::Point<PointData>> {
    rect.flip_if_required();

    let mut selected = Vec::new();
    for o in outline {
        for contour in o {
            for point in contour {
                if Rect::from(rect).contains(skulpin::skia_safe::Point::from((calc_x(point.x), calc_y(point.y)))) {
                    selected.push(point.clone());
                }
            }
        }
    }

    selected
}

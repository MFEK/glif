
use crate::{renderer::{self, UIPointType, points}, state::{Editor, SelectPointInfo}};
use super::{EditorEvent, Tool, prelude::*};
use MFEKmath::{Bezier, Evaluate, Piecewise, Vector, evaluate::Primitive};
use flo_curves::bezier::solve_curve_for_t;
use glifparser::{self, Contour, Handle, Point, PointType, WhichHandle};
use renderer::points::draw_point;
use skulpin::skia_safe::{Paint, Path, Rect};
#[derive(Clone)]
pub struct Pen {}

impl Tool for Pen {
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, position, meta } => {
                match event_type {
                    super::MouseEventType::Pressed => { self.mouse_pressed(v, position, meta) }
                    super::MouseEventType::Released => { self.mouse_released(v, position, meta)}
                    super::MouseEventType::Moved => { self.mouse_moved(v, position, meta) }
                }
            }
            EditorEvent::Draw { skia_canvas } => { 
                self.draw_nearest_point(v, skia_canvas);
                self.draw_merge_preview(v, skia_canvas);
             }
            _ => {}
        }
    }
}

impl Pen {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_moved(&self, v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        if !v.mousedown { return };

        if let Some(idx) = v.contour_idx {
            let mousepos = v.mousepos;
            v.with_active_layer_mut(|layer| {
                let outline = get_outline_mut!(layer);
                let last_point = outline[idx].last().unwrap().clone();

                let pos = (calc_x(mousepos.0 as f32), calc_y(mousepos.1 as f32));
                let offset = (last_point.x - pos.0, last_point.y - pos.1);
                let handle_b = (last_point.x + offset.0, last_point.y + offset.1);

                outline[idx].last_mut().unwrap().a = Handle::At(calc_x(mousepos.0 as f32), calc_y(mousepos.1 as f32));
                outline[idx].last_mut().unwrap().b = Handle::At(handle_b.0, handle_b.1);
            });
        }
    }

    fn mouse_pressed(&self, v: &mut Editor, _position: (f64, f64), meta: MouseMeta) {
        v.begin_layer_modification("Add point.");


        // we've got a point selected?
        if v.contour_idx.is_some() && v.point_idx.is_some() {
            // we've clicked a handle?
            if let Some(info) = v.clicked_point_or_handle(None) {
                let c_idx = v.contour_idx.unwrap();
                let p_idx = v.contour_idx.unwrap();

                // we have the end of one contour active and clicked the start of another?
                let end_is_active = v.get_contour_start_or_end(c_idx, p_idx) == Some(SelectPointInfo::End);
                let start_is_clicked = v.get_contour_start_or_end(info.0, info.1) == Some(SelectPointInfo::Start);

                // make sure these contours are open
                let selected_open = v.with_active_layer(|layer| get_contour_type!(layer, c_idx)) == PointType::Move;
                let target_open = v.with_active_layer(|layer| get_contour_type!(layer, info.0)) == PointType::Move;
                if end_is_active && start_is_clicked && selected_open && target_open {
                    v.with_active_layer_mut(|layer| {
                        get_contour_mut!(layer, c_idx).push(Point::from_x_y_type(
                        (calc_x(_position.0 as f32), calc_y(_position.1 as f32)),
                        PointType::Curve
                        ));
                    });
                    v.merge_contours(info.0, c_idx);
                    return;
                }
            }
    
        }

        // if we clicked on an existing curve we add a point there and return
        if let Some(info) = nearest_point_on_curve(v) {
            v.with_active_layer_mut(|layer| {
                let mut second_idx_zero = false;
                let contour = &mut layer.outline.as_mut().unwrap()[info.contour_idx];
                let mut point = contour.remove(info.seg_idx);
                let mut next_point = if info.seg_idx == contour.len() {
                    second_idx_zero = true;
                    contour.remove(0)
                } else { 
                    contour.remove(info.seg_idx) 
                };

                let bez = Bezier::from(&point, &next_point);
                let subdivisions = bez.subdivide(info.t);

                if let Some(subdivisions) = subdivisions {
                    let (sub_a, sub_b) = (subdivisions.0.to_control_points(), subdivisions.1.to_control_points());
                    point.a = sub_a[1].to_handle();
                    next_point.b = sub_b[2].to_handle();

                    if second_idx_zero { 
                        contour.insert(0, next_point);
                    } else {
                        contour.insert(info.seg_idx, next_point);
                    }

                    let (x, y) = (sub_a[3].x, sub_a[3].y);
                    contour.insert(info.seg_idx, Point{
                        x: x as f32,
                        y: y as f32,
                        a: sub_b[1].to_handle(),
                        b: sub_a[2].to_handle(),
                        name: None,
                        ptype: PointType::Curve,
                        data: None,

                    });

                    contour.insert(info.seg_idx, point);
                }
            });
            return
        }

        // if we click somewhere else and we have the last point of a contour selected continue that contour
        if let Some(contour_idx) = v.contour_idx {
            let mouse_pos = v.mousepos;
            let contour_len = v.with_active_layer(|layer| {get_outline!(layer)[contour_idx].len()});

            if v.point_idx.unwrap() == contour_len - 1 {
                v.point_idx = v.with_active_layer_mut(|layer| {
                    let outline = get_outline_mut!(layer);
                    outline[contour_idx].push(Point::from_x_y_type(
                    (calc_x(mouse_pos.0 as f32), calc_y(mouse_pos.1 as f32)),
                    PointType::Curve,
                    ));
    
                    Some(outline[contour_idx].len() - 1)
                });
                return
            }
        }

        // if not let's create a new contour
        let mouse_pos = v.mousepos;
        v.contour_idx = v.with_active_layer_mut(|layer| {
            let outline = get_outline_mut!(layer);
            let mut new_contour: Contour<PointData> = Vec::new();
            new_contour.push(Point::from_x_y_type(
                (calc_x(mouse_pos.0 as f32), calc_y(mouse_pos.1 as f32)),
                if meta.modifiers.shift {
                    PointType::Move
                } else {
                    PointType::Curve
                },
            ));
            outline.push(new_contour);

            Some(outline.len() - 1)
        });
        v.point_idx = Some(0);
    }

    fn mouse_released(&self, v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        if let Some(idx) = v.contour_idx {
            v.with_active_layer_mut(|layer| {
                get_outline_mut!(layer)[idx].last_mut().map(|point| {
                    if point.a != Handle::Colocated && point.ptype != PointType::Move {
                        point.ptype = PointType::Curve;
                    }
                });
            });
        }

        v.end_layer_modification();
    }

    fn draw_nearest_point(&self, v: &mut Editor, canvas: &mut Canvas) {
        if v.mousedown { return };
        let info = nearest_point_on_curve(v);

        if let Some(info) = info {
            draw_point(
                v,
                (calc_x(info.point.0), calc_y(info.point.1)),
                info.point,
                None,
                UIPointType::Point((Handle::At(info.a.0, info.a.1), Handle::At(info.b.0, info.b.1))),
                true,
                canvas
            )
        }
    }

    fn draw_merge_preview(&self, v: &Editor, canvas: &mut Canvas) {
        // we've got a point selected?
        if v.contour_idx.is_some() && v.point_idx.is_some() {
            // we've clicked a handle?
            if let Some(info) = v.clicked_point_or_handle(None) {
                let c_idx = v.contour_idx.unwrap();
                let p_idx = v.contour_idx.unwrap();

                // we have the end of one contour active and clicked the start of another?
                let end_is_active = v.get_contour_start_or_end(c_idx, p_idx) == Some(SelectPointInfo::End);
                let start_is_clicked = v.get_contour_start_or_end(info.0, info.1) == Some(SelectPointInfo::Start);

                // make sure these contours are open
                let selected_open = v.with_active_layer(|layer| get_contour_type!(layer, c_idx)) == PointType::Move;
                let target_open = v.with_active_layer(|layer| get_contour_type!(layer, info.0)) == PointType::Move;
                if end_is_active && start_is_clicked && selected_open && target_open {
                    let point = v.with_active_layer(|layer| get_contour!(layer, info.0)[info.1].clone());
                    draw_point(
                        v,
                        (calc_x(point.x), calc_y(point.y)),
                        (point.x, point.y),
                        None,
                        UIPointType::Point((point.a, point.b)),
                        true,
                        canvas
                    );
                }
            }
    
        }
    }
}


struct PenPointInfo {
    t: f64,
    contour_idx: usize,
    seg_idx: usize,
    point: (f32, f32),
    a: (f32, f32),
    b: (f32, f32),
}
fn nearest_point_on_curve(v: &Editor) -> Option<PenPointInfo>
{
    v.with_active_layer(|layer| {
        let pw: Piecewise<Piecewise<Bezier>> = layer.outline.as_ref().unwrap().into();
        
        let mut distance = f64::INFINITY;
        let mut current = None;
        let mut h1 = None;
        let mut h2 = None;

        let mut t = None;
        let mut contour_idx = None;
        let mut seg_idx = None;

        for (cx, contour) in pw.segs.iter().enumerate() {
            for (bx, bezier) in contour.segs.iter().enumerate() {
                let mouse_vec = Vector::from_components(calc_x(v.mousepos.0 as f32) as f64, calc_y(v.mousepos.1 as f32) as f64);
                let ct = solve_curve_for_t(bezier, &mouse_vec, 3.5 / v.factor as f64);
                
                if let Some(ct) = ct {
                    let new_distance = bezier.at(ct).distance(mouse_vec);
                    if new_distance < distance {
                        distance = new_distance;
                        current = Some(bezier.at(ct));
                        t = Some(ct);
                        contour_idx = Some(cx);
                        seg_idx = Some(bx);

                        let subdivisions = bezier.subdivide(ct);
                        if let Some(subdivisions) = subdivisions {
                            h1 = Some(subdivisions.0.to_control_points()[2]);
                            h2 = Some(subdivisions.1.to_control_points()[1]);
                        }
                        else
                        {
                            return None
                        }
                    }
                }
            }
        }

        if let Some(current) = current { 
            let (h1, h2) = (h1.unwrap(), h2.unwrap());
            Some(PenPointInfo {
                t: t.unwrap(),
                contour_idx: contour_idx.unwrap(),
                seg_idx: seg_idx.unwrap(),
                point: (current.x as f32, current.y as f32),
                a: (h1.x as f32, h1.y as f32),
                b: (h2.x as f32, h2.y as f32),
            })
        } else { None }
    })
}
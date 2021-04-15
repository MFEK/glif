use crate::state::Editor;
use super::{EditorEvent, Tool, prelude::*};
use glifparser::{self, Contour, Handle, Point, PointType};
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
            _ => {}
        }
    }
}

impl Pen {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_moved(&self, v: &mut Editor, position: (f64, f64), _meta: MouseMeta) {
        if !v.mousedown { return };

        if let Some(idx) = v.contour_idx {
            v.with_active_layer_mut(|layer| {
                let outline = get_outline_mut!(layer);
                let last_point = outline[idx].last().unwrap().clone();

                let pos = (calc_x(position.0 as f32), calc_y(position.1 as f32));
                let offset = (last_point.x - pos.0, last_point.y - pos.1);
                let handle_b = (last_point.x + offset.0, last_point.y + offset.1);

                outline[idx].last_mut().unwrap().a = Handle::At(calc_x(position.0 as f32), calc_y(position.1 as f32));
                outline[idx].last_mut().unwrap().b = Handle::At(handle_b.0, handle_b.1);
            });
        }
    }

    fn mouse_pressed(&self, v: &mut Editor, _position: (f64, f64), meta: MouseMeta) {
        v.begin_layer_modification("Add point.");

        match v.contour_idx {
            Some(idx) => {
                let mouse_pos = v.mousepos;
                v.with_active_layer_mut(|layer| {
                    let outline = get_outline_mut!(layer);
                    outline[idx].push(Point::from_x_y_type(
                    (calc_x(mouse_pos.0 as f32), calc_y(mouse_pos.1 as f32)),
                    PointType::Curve,
                    ));
                });
            }
            None => {
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
                })
            }
        }
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
}
use super::prelude::*;
use crate::contour_operations;
use crate::tool_behaviors::{move_handle::MoveHandle, pan::PanBehavior, zoom_scroll::ZoomScroll};
use crate::user_interface::Interface;
use glifrenderer::points::draw_point;

use editor::util::get_contour_start_or_end;
use glifrenderer::points::UIPointType;
use sdl2::mouse::MouseButton;
use MFEKmath::{Bezier, Primitive};
#[derive(Clone, Debug)]
pub struct Pen {}

impl Tool for Pen {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { mouse_info, event_type } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => (),
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {}
        }
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_merge_preview(v, i, canvas);
        self.draw_nearest_point(v, i, canvas);
    }
}

impl Pen {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_pressed(&self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
        if mouse_info.button != MouseButton::Left {
            v.set_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info)));
            return;
        };

        v.begin_modification("Add point.");

        // We check if we have a point selected and are clicking on the beginning of another contour.
        // If that is the case we merge them
        if let (Some(c_idx), Some(p_idx)) = (v.contour_idx, v.point_idx) {
            // we've clicked a point?
            if let Some((info_ci, info_pi, _)) =
                clicked_point_or_handle(v, i, mouse_info.raw_position, None)
            {
                // we have the end of one contour active and clicked the start of another?
                let end_is_active =
                    get_contour_start_or_end(v, c_idx, p_idx) == Some(SelectPointInfo::End);
                let start_is_clicked =
                    get_contour_start_or_end(v, info_ci, info_pi) == Some(SelectPointInfo::Start);

                // make sure these contours are open
                let selected_open =
                    v.with_active_layer(|layer| get_contour_type!(layer, c_idx)) == PointType::Move;
                let target_open = v.with_active_layer(|layer| get_contour_type!(layer, info_ci))
                    == PointType::Move;
                if end_is_active && start_is_clicked && selected_open && target_open {
                    v.with_active_layer_mut(|layer| {
                        let new_point = get_point!(layer, info_ci, info_pi).clone();
                        get_contour_mut!(layer, c_idx).push(new_point);
                    });
                    v.merge_contours(info_ci, c_idx);
                    v.end_modification();
                    return;
                }
            }
        }

        // Next we check if our mouse is over an existing curve. If so we add a point to the curve.
        if let Some(info) = nearest_point_on_curve(v, i, mouse_info.position) {
            v.with_active_layer_mut(|layer| {
                let mut second_idx_zero = false;
                let contour = &mut layer.outline[info.contour_idx];
                let mut point = contour.inner.remove(info.seg_idx);
                let mut next_point = if info.seg_idx == contour.inner.len() {
                    second_idx_zero = true;
                    contour.inner.remove(0)
                } else {
                    contour.inner.remove(info.seg_idx)
                };

                let bez = Bezier::from(&point, &next_point);
                let subdivisions = bez.subdivide(info.t);

                if let Some(subdivisions) = subdivisions {
                    let (sub_a, sub_b) = (
                        subdivisions.0.to_control_points(),
                        subdivisions.1.to_control_points(),
                    );
                    point.a = sub_a[1].to_handle();
                    next_point.b = sub_b[2].to_handle();

                    if second_idx_zero {
                        contour.inner.insert(0, next_point);
                    } else {
                        contour.inner.insert(info.seg_idx, next_point);
                    }

                    let (x, y) = (sub_a[3].x, sub_a[3].y);
                    contour.inner.insert(
                        info.seg_idx,
                        Point::from_x_y_a_b_type(
                            (x as f32, y as f32),
                            (sub_b[1].to_handle(), sub_a[2].to_handle()),
                            PointType::Curve,
                        ),
                    );
                    contour.operation = contour_operations::insert(contour, info.seg_idx);
                    contour.inner.insert(info.seg_idx, point);
                }
            });
            v.end_modification();
        }
        // If we've got the end of a contour selected we'll continue drawing that contour.
        else if let Some(contour_idx) = v.contour_idx {
            let mouse_pos = mouse_info.position;
            let contour_len = v.with_active_layer(|layer| get_contour_len!(layer, contour_idx));

            if v.point_idx.unwrap() == contour_len - 1 {
                v.point_idx = v.with_active_layer_mut(|layer| {
                    get_contour!(layer, contour_idx).push(Point::from_x_y_type(
                        (mouse_pos.0 as f32, mouse_pos.1 as f32),
                        PointType::Curve,
                    ));

                    layer.outline[contour_idx].operation =
                        contour_operations::insert(&layer.outline[contour_idx], contour_len);
                    Some(get_contour_len!(layer, contour_idx) - 1)
                });
            } else if v.point_idx.unwrap() == 0 {
                v.with_active_layer_mut(|layer| {
                    let point_type = get_point!(layer, contour_idx, 0).ptype;

                    if get_point!(layer, contour_idx, 0).ptype == PointType::Move {
                        get_point!(layer, contour_idx, 0).ptype = PointType::Curve;
                    }
                    get_contour!(layer, contour_idx).insert(
                        0,
                        Point::from_x_y_type((mouse_pos.0 as f32, mouse_pos.1 as f32), point_type),
                    );

                    layer.outline[contour_idx].operation =
                        contour_operations::insert(&layer.outline[contour_idx], 0);
                });
                v.end_modification();
            }
        } else {
            // Lastly if we get here we create a new contour.
            let mouse_pos = mouse_info.position;
            v.contour_idx = v.with_active_layer_mut(|layer| {
                let mut new_contour: Contour<MFEKGlifPointData> = Vec::new();
                new_contour.push(Point::from_x_y_type(
                    (mouse_pos.0 as f32, mouse_pos.1 as f32),
                    if mouse_info.modifiers.shift {
                        PointType::Curve
                    } else {
                        PointType::Move
                    },
                ));

                layer.outline.push(new_contour.into());
                Some(layer.outline.len() - 1)
            });
            v.end_modification();
            v.point_idx = Some(0);
        }

        // No matter how you move the point we want you to be able to manipulate it so we push the MoveHandle
        // vehavior onto the editor's behavior stack.
        v.push_behavior(Box::new(MoveHandle::new(WhichHandle::A, mouse_info, true)));
    }

    fn draw_nearest_point(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        if i.mouse_info.is_down {
            return;
        };
        let info = nearest_point_on_curve(v, i, i.mouse_info.position);

        if let Some(info) = info {
            draw_point::<()>(
                &i.viewport,
                &Point::from_x_y_type(info.point, PointType::Curve),
                None,
                UIPointType::Point((
                    Handle::At(info.a.0, info.a.1),
                    Handle::At(info.b.0, info.b.1),
                )),
                true,
                canvas,
            )
        }
    }

    fn draw_merge_preview(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        // we've got a point selected?
        if let (Some(c_idx), Some(p_idx)) = (v.contour_idx, v.point_idx) {
            // we've clicked a handle?
            if let Some((info_ci, info_pi, _)) =
                clicked_point_or_handle(v, i, i.mouse_info.raw_position, None)
            {
                // we have the end of one contour active and clicked the start of another?
                let end_is_active =
                    get_contour_start_or_end(v, c_idx, p_idx) == Some(SelectPointInfo::End);
                let start_is_clicked =
                    get_contour_start_or_end(v, info_ci, info_pi) == Some(SelectPointInfo::Start);

                // make sure these contours are open
                let selected_open =
                    v.with_active_layer(|layer| get_contour_type!(layer, c_idx)) == PointType::Move;
                let target_open = v.with_active_layer(|layer| get_contour_type!(layer, info_ci))
                    == PointType::Move;
                if end_is_active && start_is_clicked && selected_open && target_open {
                    let point =
                        v.with_active_layer(|layer| get_contour!(layer, info_ci)[info_pi].clone());
                    draw_point(
                        &i.viewport,
                        &point,
                        None,
                        UIPointType::Point((point.a, point.b)),
                        true,
                        canvas,
                    );
                }
            }
        }
    }
}

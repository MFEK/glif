use std::collections::HashSet;

// Select
use super::{prelude::*, EditorEvent, MouseEventType, Tool};
use crate::command::{Command, CommandType};
use crate::tool_behaviors::rotate_selection::RotateSelection;
use crate::util::math::ReverseContours as _;

use MFEKmath::Vector;

use crate::tool_behaviors::move_handle::MoveHandle;
use crate::tool_behaviors::move_point::MovePoint;
use crate::tool_behaviors::pan::PanBehavior;
use crate::tool_behaviors::selection_box::SelectionBox;

mod dialog;

// Select is a good example of a more complicated tool that keeps lots of state.
// It has state for which handle it's selected, follow rules, selection box, and to track if it's currently
// moving a point.
#[derive(Clone, Debug)]
pub struct Select {
    pivot_point: Option<(f32, f32)>,
}

impl Tool for Select {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                MouseEventType::DoubleClick => self.mouse_double_pressed(v, i, mouse_info),
                _ => {}
            },
            EditorEvent::ToolCommand {
                command: Command::SelectAll,
                stop_after,
                ..
            } => {
                *stop_after = true;
                self.select_all(v);
            }
            EditorEvent::ToolCommand {
                command: Command::ReverseContour,
                stop_after,
                ..
            } => {
                *stop_after = true;
                self.reverse_selected(v);
            }
            EditorEvent::ToolCommand {
                command,
                stop_after,
                ..
            } => {
                if command.type_() == CommandType::Nudge {
                    *stop_after = true;
                    self.nudge_selected(v, command);
                }
            }
        }
    }

    fn draw(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_pivot_point(v, i, canvas);
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.select_settings(v, i, ui);
    }
}

impl Select {
    pub fn new() -> Self {
        Self { pivot_point: None }
    }

    fn select_all(&mut self, v: &mut Editor) {
        let points = v.with_active_layer(|layer| {
            let mut points = HashSet::new();
            for (ci, contour) in layer.outline.iter().enumerate() {
                for (pi, _point) in contour.inner.iter().enumerate() {
                    points.insert((ci, pi));
                }
            }
            points
        });
        v.selected = points;
    }

    fn nudge_selected(&mut self, v: &mut Editor, command: Command) {
        for (ci, pi) in v.selected.clone() {
            v.begin_modification("Nudge selected points.");
            v.with_active_layer_mut(|layer| {
                let point = &get_point!(layer, ci, pi);
                let factor = PanBehavior::nudge_factor(command);
                let offset = PanBehavior::nudge_offset(command, factor);
                let x = point.x;
                let y = point.y;
                editor::util::move_point(&mut layer.outline, ci, pi, x - offset.0, y + offset.1);
            });
            v.end_modification();
        }
    }

    fn reverse_selected(&mut self, v: &mut Editor) {
        let ci = if let Some((ci, _)) = v.selected() {
            ci
        } else {
            return;
        };

        v.begin_modification("Reversing contours.");
        let point_idx = v.point_idx;
        v.point_idx = v.with_active_layer_mut(|layer| {
            let contour_len = layer.outline[ci].inner.len();
            layer.outline[ci].inner = layer.outline[ci].inner.clone().reverse_contours();
            if let Some(pi) = point_idx {
                if layer.outline[ci].inner[0].ptype != PointType::Move {
                    if pi == 0 {
                        Some(0)
                    } else {
                        Some(contour_len - pi)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        });
        if !v.point_idx.is_some() {
            v.contour_idx = None;
        }
        v.end_modification();
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
        // if the user clicked middle mouse we initiate a pan behavior
        if mouse_info.button == MouseButton::Middle {
            v.set_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info)));
            return;
        }

        // if the user holds control we initiate a rotation of the current selection, either around the pivot point
        // or around the selection's bounding box's center
        if mouse_info.modifiers.ctrl && !v.selected.is_empty() {
            let pivot = self
                .pivot_point
                .unwrap_or_else(|| v.get_selection_bounding_box_center());
            let pivot_calc = (calc_x(pivot.0), calc_y(pivot.1));
            let pivot_vector = Vector::from_components(pivot_calc.0 as f64, pivot_calc.1 as f64);
            let mouse_vector =
                Vector::from_components(mouse_info.position.0 as f64, mouse_info.position.1 as f64);
            let normal_from_pivot = (pivot_vector - mouse_vector).normalize();

            v.set_behavior(Box::new(RotateSelection::new(
                pivot,
                normal_from_pivot.into(),
                mouse_info,
            )));
            return;
        }

        // if we found a point or handle we're going to start a drag operation
        match clicked_point_or_handle(v, i, mouse_info.position, None) {
            Some((ci, pi, wh)) => {
                // first we check if shift is  held, if they are we put the current selection
                // into the editor's selected HashSet
                if mouse_info.modifiers.shift {
                    if let Some(point_idx) = v.point_idx {
                        v.selected.insert((v.contour_idx.unwrap(), point_idx));
                    }
                } else if !v.selected.contains(&(ci, pi)) {
                    // if the user isn't holding shift or control, and the point they're clicking is not in the current
                    // selection we clear the selection
                    v.selected = HashSet::new();
                }

                // Set the editor's selected point to the most recently clicked one.
                v.contour_idx = Some(ci);
                v.point_idx = Some(pi);

                if wh == WhichHandle::Neither {
                    // the user clicked niether handle so that's our cue to push a move_point behavior on the stack
                    let move_selected = !mouse_info.modifiers.ctrl;
                    v.set_behavior(Box::new(MovePoint::new(move_selected, mouse_info)));
                } else {
                    // the user clicked a handle so we push a move_handle behavior
                    let follow = mouse_info.into();
                    v.set_behavior(Box::new(MoveHandle::new(wh, follow, mouse_info)));
                }
            }
            None => {
                // if the user isn't holding shift we clear the current selection and the currently selected
                // point
                if !mouse_info.modifiers.shift {
                    v.selected = HashSet::new();
                    v.contour_idx = None;
                    v.point_idx = None;
                }

                // if they clicked right mouse we set the pivot point that will be used by rotate_points behavior.
                if mouse_info.button == MouseButton::Right {
                    self.pivot_point =
                        Some((mouse_info.position.0, rcalc_y(mouse_info.position.1)));
                } else if mouse_info.button == MouseButton::Left {
                    v.set_behavior(Box::new(SelectionBox::new(mouse_info)));
                }
            }
        };
    }

    fn mouse_double_pressed(&mut self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
        let ci = if let Some((ci, _pi, _wh)) =
            clicked_point_or_handle(v, i, mouse_info.raw_position, None)
        {
            ci
        } else {
            return;
        };

        let contour_len = v.with_active_layer(|layer| get_contour_len!(layer, ci));

        if !mouse_info.modifiers.shift {
            v.selected = HashSet::new();
        }

        for pi in 0..contour_len {
            v.selected.insert((ci, pi));
        }
    }

    fn draw_pivot_point(&self, _v: &Editor, i: &Interface, canvas: &mut Canvas) {
        if let Some(pivot) = self.pivot_point {
            let pivot = (calc_x(pivot.0), calc_y(pivot.1));
            let mut paint = Paint::default();

            paint.set_color(OUTLINE_STROKE);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / i.viewport.factor));
            canvas.draw_circle(pivot, 5. * (1. / i.viewport.factor), &paint);
        }
    }
}

use std::collections::HashSet;

// Select
use super::{prelude::*, EditorEvent, MouseEventType, Tool};
use crate::command::{Command, CommandType};
use crate::editor::tunni::{get_closest_tunni_line, get_tunni_line_from_info, get_tunni_point_from_info, clicked_tunni_point_or_line};
use crate::get_point_mut;
use crate::tool_behaviors::move_tunni_point::MoveTunniPoint;
use crate::tool_behaviors::rotate_selection::RotateSelection;
use crate::tool_behaviors::move_tunni_line::MoveTunniLine;
use glifparser::glif::mfek::contour::MFEKContourCommon;

use MFEKmath::Vector;

use crate::tool_behaviors::{
    draw_pivot::DrawPivot, move_handle::MoveHandle, move_point::MovePoint, pan::PanBehavior,
    selection_box::SelectionBox, zoom_scroll::ZoomScroll,
};

// Select is a good example of a more complicated tool that keeps lots of state.
// It has state for which handle it's selected, follow rules, selection box, and to track if it's currently
// moving a point.
#[derive(Clone, Debug, Default)]
pub struct Select {
    pivot_point: Option<(f32, f32)>,
    draw_pivot: DrawPivot,
}

impl Tool for Select {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        self.draw_pivot.event(v, i, event.clone());
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
                *stop_after.borrow_mut() = true;
                self.select_all(v);
            }
            EditorEvent::ToolCommand {
                command: Command::ReverseContour,
                stop_after,
                ..
            } => {
                *stop_after.borrow_mut() = true;
                self.reverse_selected(v);
            }
            EditorEvent::ToolCommand {
                command,
                stop_after,
                ..
            } => {
                if command.type_() == CommandType::Nudge {
                    *stop_after.borrow_mut() = true;
                    self.nudge_selected(v, command);
                }
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            #[allow(unreachable_patterns)] // more events likely to be added.
            _ => {}
        }
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &Canvas) {
        self.draw_tunni_line(v, i, canvas);
        self.draw_pivot.draw(v, i, canvas);
    }
}

impl Select {
    pub fn new() -> Self {
        Self::default()
    }

    fn select_all(&mut self, v: &mut Editor) {
        let mut points = HashSet::new();
        for (ci, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
            for (pi, _) in contour.inner().iter().enumerate() {
                points.insert((ci, pi));
            }
        }
        v.selected = points;
    }

    fn draw_tunni_line(&mut self, v: &Editor, i: &Interface, canvas: &Canvas) {
        let mut paint = Paint::default();
    
        let closest_tunni = get_closest_tunni_line(v, i);

        if let Some(tunni_info) = closest_tunni {
            let tunni_line = get_tunni_line_from_info(v, &tunni_info).unwrap();
            let p1 = (tunni_line.0.x as f32, tunni_line.0.y as f32);
            let p2: (f32, f32) = (tunni_line.1.x as f32, tunni_line.1.y as f32);

            paint.set_anti_alias(true);
            paint.set_color(0xFF0000FF);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / i.viewport.factor));
            canvas.draw_line(p1, p2, &paint);

            let tunni_point = get_tunni_point_from_info(v, &tunni_info).unwrap();
            paint.set_style(PaintStyle::Fill);
            canvas.draw_circle((tunni_point.x as f32, tunni_point.y as f32), 5. * (1. / i.viewport.factor), &paint);
        }
    }

    fn nudge_selected(&mut self, v: &mut Editor, command: Command) {
        let mut selected = v.selected.clone();
        if let (Some(ci), Some(pi)) = (v.contour_idx, v.point_idx) {
            selected.insert((ci, pi));
        }
        if selected.len() == 0 {
            return;
        }
        v.begin_modification("Nudge selected points.", false);
        for (ci, pi) in selected {
            let layer = v.get_active_layer_mut();
            let point = get_point_mut!(layer, ci, pi).unwrap();
            let factor = PanBehavior::nudge_factor(command);
            let offset = PanBehavior::nudge_offset(command, factor);
            
            point.set_position(point.x() - offset.0, point.y() + offset.1);
        }
        v.end_modification();
    }

    fn reverse_selected(&mut self, v: &mut Editor) {
        let ci = if let Some((ci, _)) = v.selected_point() {
            ci
        } else {
            return;
        };

        v.begin_modification("Reversing contours.", false);
        let point_idx = v.point_idx;
        v.point_idx = {
            let layer = v.get_active_layer_mut();
            let contour_len = layer.outline[ci].len();
            layer.outline[ci].reverse_points();
            if let Some(pi) = point_idx {
                if !get_contour!(layer, ci).is_open() {
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
        };
        if !v.point_idx.is_some() {
            v.contour_idx = None;
        }
        v.end_modification();
    }

    pub fn mouse_pressed(&mut self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
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
            let pivot_calc = (pivot.0, pivot.1);
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
        if let Some((ci, pi, wh)) = clicked_point_or_handle(v, i, mouse_info.raw_position, None) {
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
                v.set_behavior(Box::new(MoveHandle::new(wh, mouse_info, false)));
            }

            return
        }

        if let Some((tunni_info, tunni_type)) = clicked_tunni_point_or_line(v, i) {
            match tunni_type {
                editor::tunni::Tunni::Point => {
                    v.set_selected(tunni_info.a.0, tunni_info.a.1);
                    v.set_behavior(Box::new(MoveTunniPoint::new(mouse_info, tunni_info)));
                    return
                },
                editor::tunni::Tunni::Line => {
                    v.set_selected(tunni_info.a.0, tunni_info.a.1);
                    let line = get_tunni_line_from_info(v, &tunni_info).unwrap();
                    v.set_behavior(Box::new(MoveTunniLine::new(mouse_info, tunni_info, line)));
                    return
                },
            }
        }


        // the user clicked an empty location
        // if the user isn't holding shift we clear the current selection and the currently selected
        // point
        if !mouse_info.modifiers.shift {
            v.selected = HashSet::new();
            v.contour_idx = None;
            v.point_idx = None;
        }

        // if they clicked right mouse we set the pivot point that will be used by rotate_points behavior.
        if mouse_info.button == MouseButton::Right {
            self.pivot_point = Some((mouse_info.position.0, mouse_info.position.1));
        } else if mouse_info.button == MouseButton::Left {
            v.set_behavior(Box::new(SelectionBox::new(mouse_info)));
        }
    }

    pub fn mouse_double_pressed(&mut self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
        let ci = if let Some((ci, _pi, _wh)) =
            clicked_point_or_handle(v, i, mouse_info.raw_position, None)
        {
            ci
        } else {
            return;
        };

        let contour_len = get_contour_len!(v.get_active_layer_ref(), ci);

        if !mouse_info.modifiers.shift {
            v.selected = HashSet::new();
        }

        for pi in 0..contour_len {
            v.selected.insert((ci, pi));
        }
    }
}

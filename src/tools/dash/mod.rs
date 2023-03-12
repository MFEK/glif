mod dialog;

use glifparser::glif::contour_operations::ContourOperations;
use glifparser::glif::contour_operations::dash::DashContour;
use skia_safe::{PaintCap, PaintJoin};

use super::prelude::*;
use crate::editor::Editor;
use crate::tool_behaviors::zoom_scroll::ZoomScroll;

#[derive(Clone, Debug, Default)]
pub struct Dash {}

impl Tool for Dash {
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

    fn dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) -> bool {
        let show_dialog = match v.contour_idx {
            Some(ci) => match v.get_active_layer_ref().outline[ci].operation() {
                Some(ContourOperations::DashAlongPath { .. }) => true,
                _ => false,
            },
            _ => false,
        };

        if show_dialog {
            self.tool_dialog(v, i, ui);
            return true;
        }

        return false;
    }
}

impl Dash {
    pub fn new() -> Self {
        Self::default()
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        if let Some((ci, pi, _wh)) = clicked_point_or_handle(v, i, mouse_info.raw_position, None) {
            let layer_op = v.get_active_layer_ref().outline[ci].operation().clone();
            v.contour_idx = Some(ci);
            v.point_idx = Some(pi);

            match layer_op {
                Some(ContourOperations::DashAlongPath { .. }) => (),
                None | Some(_) => {
                    v.begin_modification("Added dash contour.", false);
                    v.get_active_layer_mut().outline[ci].set_operation(Some(ContourOperations::DashAlongPath {
                        data: DashContour {
                            stroke_width: 10.,
                            cull: None,
                            dash_desc: vec![10., 10.],
                            include_last_path: false,
                            paint_cap: PaintCap::Butt as u8,
                            paint_join: PaintJoin::Miter as u8,
                        },
                    }));
                    v.end_modification();
                }
            }
        }
    }
}

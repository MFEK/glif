use super::prelude::*;
use crate::tool_behaviors::{pan::PanBehavior, zoom_scroll::ZoomScroll};
use crate::user_interface;

#[derive(Clone, Debug, Default)]
pub struct Grid;

impl Tool for Grid {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => {
                    v.set_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info)));
                }
                _ => {}
            },
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {}
        }
    }

    fn ui(&mut self, _v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.grid_settings(i, ui);
    }
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grid_settings(&mut self, i: &mut Interface, ui: &egui::Ui) {

    }
}

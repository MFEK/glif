use glifrenderer::constants;
use skia_safe::{Point, Color4f, Typeface, Font};

use crate::editor::Editor;
use crate::tool_behaviors::zoom_scroll::ZoomScroll;
use crate::tools::cut::{Cut, Intersection};
use crate::user_interface::Interface;

use super::prelude::*;

#[derive(Clone, Debug)]
pub struct Measure {
    dragging: bool,
}

impl Tool for Measure {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Pressed => self.mouse_pressed(i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(i, mouse_info),
                MouseEventType::Released => self.mouse_released(),
                _ => (),
            }
        }

        match event {
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {},
        }
    }
}

impl Measure {
    pub fn new() -> Self {
        Self { dragging: false }
    }

    fn mouse_released(&mut self) {
        self.dragging = false
    }

    fn mouse_pressed(&mut self, i: &mut Interface, mouse_info: MouseInfo) {
        i.measure.enabled = true;
        self.dragging = true;
        i.measure.start_point = Some(mouse_info.position);
        i.measure.end_point = None;
    }

    fn mouse_moved(&mut self, i: &mut Interface, mouse_info: MouseInfo) {
        if !self.dragging { return }
        i.measure.end_point = Some(mouse_info.position);
    }
}
use crate::state::Editor;
use super::{EditorEvent, MouseEventType, Tool, prelude::*};

#[derive(Clone)]
pub struct Pan {
    last_position: Option<(f64, f64)>
}

impl Tool for Pan {
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, position, meta } => {
                match event_type {
                    MouseEventType::Moved => { self.mouse_moved(v, position, meta) }
                    MouseEventType::Pressed => { self.mouse_pressed(v, position, meta) }
                    MouseEventType::Released => { self.mouse_released(v, position, meta) }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl Pan {
    pub fn new() -> Self {
        Self {
            last_position: None
        }
    }

    fn mouse_moved(&mut self, v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        if !v.mousedown { return }
        if let Some(pivot_point) = self.last_position {
            let mut offset = v.offset;

            offset.0 += (v.absolute_mousepos.0 - pivot_point.0).floor() as f32;
            offset.1 += (v.absolute_mousepos.1 - pivot_point.1).floor() as f32;
           
            v.offset = offset;

            self.last_position = Some(v.absolute_mousepos);
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        self.last_position = Some(v.absolute_mousepos);
    }

    fn mouse_released(&mut self, _v: &mut Editor, _position: (f64, f64), _meta: MouseMeta) {
        self.last_position = None;
    }
}

use super::prelude::*;
#[derive(Clone)]
pub struct Anchors {}

impl Anchors {
    pub fn new() -> Self {
        Anchors{}
    }
}

impl Tool for Anchors {
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                match event_type {
                    //MouseEventType::Moved => { self.mouse_moved(v, meta) }
                    //MouseEventType::Pressed => { self.mouse_pressed(v, meta) }
                    //MouseEventType::Released => { self.mouse_released(v, meta) }
                    
                    _ => {}
                }
            },
            _ => {}
        }
    }
}

use skulpin::skia_safe::{Paint, PaintStyle};
impl Anchors {

}

use crate::editor::Editor;
use crate::user_interface::Interface;
use super::prelude::*;

#[derive(Clone)]
pub struct Grid {
}


impl Tool for Grid {
    fn handle_event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                match event_type {
                    MouseEventType::Moved => { self.mouse_moved(i, meta) }
                    MouseEventType::Pressed => { self.mouse_pressed(meta) }
                    MouseEventType::Released => { self.mouse_released() }
                    _ => {}
                }
            }
            EditorEvent::Draw { skia_canvas } => { 
                self.draw_grid(v, i, skia_canvas);
            }
            _ => {}
        }
    }
}
 
impl Grid {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_moved(&mut self, i: &mut Interface, meta: MouseInfo) {
    }

    fn mouse_pressed(&mut self, meta: MouseInfo) {
    }

    fn mouse_released(&mut self) {
    }

    fn draw_grid(&mut self, v: &mut Editor, i: &mut Interface, canvas: &mut Canvas) {

    }
}

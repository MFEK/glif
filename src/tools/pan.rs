use crate::editor::Editor;
use crate::tool_behaviors::pan::PanBehavior;
use crate::user_interface::Interface;
use super::prelude::*;

// Pan is a good example of a simple tool. It holds only a little bit of state, an optional position.
// If the optional position is set the tool will calculate the delta between that position and the current
// mouse position and set the camera accordingly.
#[derive(Clone)]
pub struct Pan {
    last_position: Option<(f32, f32)>
}


// We implement Tool for our tool. Here you can route events to functions or implement logic directly in the
// match statement.
impl Tool for Pan {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, mouse_info } => {
                match event_type {
                    MouseEventType::Pressed => { 
                        v.push_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info)))
                     }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

// Here you can implement behaviors for events. 
impl Pan {
    pub fn new() -> Self {
        Self {
            last_position: None
        }
    }
}
